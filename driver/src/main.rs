use std::{
    collections::VecDeque,
    sync::mpsc::{self, Receiver, Sender, TryRecvError},
    thread,
    time::{Duration, Instant},
};

use config::RENDER_FRAMERATE;
use esp_idf_svc::{
    hal::{
        self,
        delay::Delay,
        gpio::{AnyInputPin, AnyOutputPin, PinDriver},
        uart::{config::Config, UartDriver},
    },
    io::Read,
    sys::{self},
};
use gpio::ControlPins;
use log::info;
use prolite::{
    api::{Animation, Color, Command, ContentDuration},
    ScreenBuffer,
};
use renderer::{
    current_content::{ContentGroupState, CurrentContent},
    glyphs::get_glyph_placement,
    UnknownGlyphBehavior,
};

mod config;
mod driver;
mod gpio;
mod renderer;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    // Disable IDLE task WDT on this CPU.
    unsafe {
        sys::esp_task_wdt_delete(sys::xTaskGetIdleTaskHandleForCore(hal::cpu::core() as i32))
    };
    // Enable WDT on the main task (this task).
    unsafe { sys::esp_task_wdt_add(sys::xTaskGetCurrentTaskHandle()) };

    let peripherals = hal::prelude::Peripherals::take().unwrap();

    let mut control_pins = ControlPins {
        r: PinDriver::output(peripherals.pins.gpio4).unwrap(),
        g: PinDriver::output(peripherals.pins.gpio5).unwrap(),

        row_0: PinDriver::output(peripherals.pins.gpio11).unwrap(),
        row_1: PinDriver::output(peripherals.pins.gpio10).unwrap(),
        row_2: PinDriver::output(peripherals.pins.gpio9).unwrap(),

        clk: PinDriver::output(peripherals.pins.gpio18).unwrap(),
        screen: PinDriver::output(peripherals.pins.gpio8).unwrap(),
    };

    let uart_rx = UartDriver::new(
        peripherals.uart1,
        peripherals.pins.gpio14,
        peripherals.pins.gpio13,
        Option::<AnyInputPin>::None,
        Option::<AnyOutputPin>::None,
        &Config::default(),
    )
    .unwrap();

    let (command_tx, command_rx) = mpsc::channel();
    let (buffer_tx, buffer_rx) = mpsc::channel();

    let delay_driver = Delay::new_default();

    thread::Builder::new()
        .stack_size(8 * 1024)
        .spawn(move || initialize_uart_thread(uart_rx, command_tx))
        .unwrap();

    thread::Builder::new()
        .stack_size(8 * 1024)
        .spawn(move || initialize_renderer_thread(command_rx, buffer_tx))
        .unwrap();
    // thread::Builder::new()
    // .stack_size(8 * 1024)
    // .spawn(move || {

    let mut buffer = initial_buffer();
    loop {
        match buffer_rx.try_recv() {
            Ok(b) => buffer = b,
            Err(_) => { /* do nothing */ }
        }

        match driver::display_screen(&buffer, &delay_driver, &mut control_pins) {
            Ok(_) => { /* do nothing */ }
            Err(e) => info!("[driver] error: {:?}", e),
        }

        unsafe {
            sys::esp_task_wdt_reset();
        }
    }

    // }).unwrap();
}

fn initialize_renderer_thread(
    command_rx: Receiver<prolite::api::Command>,
    screen_buffer_tx: Sender<Box<ScreenBuffer>>,
) {
    let mut current_content = None;
    let mut content_queue = Box::new(VecDeque::new());
    let mut now = Instant::now();

    let behavior = UnknownGlyphBehavior::ReplaceWithPlaceholder;

    // Loop:
    // 1. Receive new commands
    // 2. Render command and update screen
    // 3. Sleep until next frame
    loop {
        if let Some(command) = try_recv(&command_rx) {
            info!("[render] received new command {:?}", &command);

            match command {
                prolite::api::Command::AddToQueue { content } => content_queue.push_back(content),
                prolite::api::Command::ShowNow { content } => {
                    content_queue.clear();
                    current_content = Some(CurrentContent::new(content, behavior));
                }
                prolite::api::Command::Clear => {
                    content_queue.clear();
                    current_content = None;
                }
            }
        }

        if current_content.is_none() {
            if let Some(next_content) = content_queue.pop_front() {
                info!("[render] starting render of content {:?}", &next_content);
                current_content = Some(CurrentContent::new(next_content, behavior))
            }
        }

        if let Some(cc) = current_content.as_mut() {
            let should_render_current_frame;
            let should_replace_current_content;

            let u = cc.update(now);
            match u {
                ContentGroupState::StepStarted => {
                    should_render_current_frame = true;
                    should_replace_current_content = false;
                }
                ContentGroupState::StepIncomplete => {
                    should_render_current_frame = cc.is_animated();
                    should_replace_current_content = false;
                }
                ContentGroupState::Finished => {
                    should_render_current_frame = cc.is_animated();
                    should_replace_current_content = true;
                }
            }

            if should_render_current_frame {
                let rendered = cc.render(now);
                send(&screen_buffer_tx, rendered);
            }

            if should_replace_current_content {
                info!("[render] finished rendering previous content");
                current_content = None;
            }
        }

        let elapsed = now.elapsed();

        if elapsed < RENDER_FRAMERATE {
            thread::sleep(RENDER_FRAMERATE - elapsed);
            now = now.checked_add(RENDER_FRAMERATE).unwrap();
        } else {
            now = Instant::now();
        }
    }
}

fn initialize_uart_thread(
    mut uart_receiver: UartDriver,
    buffer_sender: Sender<prolite::api::Command>,
) {
    info!("uart init");

    loop {
        let read = read_next_command(&mut uart_receiver);
        match read {
            Ok(Ok(command)) => send(&buffer_sender, command),
            Ok(Err(e)) => info!("[uart] failed to deserialize command: {}", e),
            Err(e) => info!("[uart] failed to receive command: {}", e),
        }
    }
}

fn read_next_command(
    uart_receiver: &mut UartDriver,
) -> Result<serde_json::Result<Command>, String> {
    let mut command = vec![];

    loop {
        let mut remaining_read = uart_receiver.remaining_read().map_err(|e| e.to_string())?;

        if remaining_read == 0 {
            thread::sleep(Duration::from_secs(1));
            continue;
        }

        while remaining_read > 0 {
            let to_read = if remaining_read > 256 {
                256
            } else {
                remaining_read
            };
            let mut buffer = vec![0; to_read];
            uart_receiver
                .read_exact(&mut buffer)
                .map_err(|e| e.to_string())?;
            command.extend_from_slice(&buffer);

            // wait a tiny while to see if more data is coming
            thread::sleep(Duration::from_millis(20));
            remaining_read = uart_receiver.remaining_read().map_err(|e| e.to_string())?;
        }

        return Ok(serde_json::from_slice(&command));
    }
}

fn initial_buffer() -> Box<ScreenBuffer> {
    let rendered_glyphs = get_glyph_placement("booting...", UnknownGlyphBehavior::Ignore);
    let color = Color::Orange;
    let animation = Animation::None {
        duration: ContentDuration::Forever,
    };

    renderer::render(&rendered_glyphs, &color, &animation, None, Duration::ZERO)
}

fn send<T>(sender: &Sender<T>, value: T) {
    // send only errors when the channel is disconnected
    // todo restart threads, but for now just panic
    sender.send(value).unwrap()
}

fn try_recv<T>(receiver: &Receiver<T>) -> Option<T> {
    match receiver.try_recv() {
        Ok(x) => Some(x),
        Err(TryRecvError::Empty) => None,
        Err(TryRecvError::Disconnected) => {
            // todo restart threads, but for now just panic
            panic!("channel disconnected: {:?}", receiver)
            // None
        }
    }
}
