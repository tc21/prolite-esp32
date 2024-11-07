use std::{
    collections::VecDeque,
    sync::mpsc::{self, Receiver, Sender, TryRecvError},
    thread,
    time::Instant,
};

use config::RENDER_FRAMERATE;
use esp_idf_svc::{
    hal::{
        self,
        delay::Delay,
        gpio::{AnyInputPin, AnyOutputPin, PinDriver},
        uart::{config::Config, UartDriver},
    },
    sys,
};
use gpio::ControlPins;
use log::info;
use prolite::{api::{Content, Repeat}, ScreenBuffer};
use renderer::render_result::{ContentState, CurrentContent, ScreenBufferState};

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

    info!("{:?}", hal::cpu::core());

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
        peripherals.uart0,
        peripherals.pins.gpio43,
        peripherals.pins.gpio44,
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
    let content = Content {
        text: "Now Playing: 初音ミクの消失 - cosMo".to_owned(),
        color: prolite::api::Color::Green,
        animation: prolite::api::Animation::Slide(
            prolite::api::SlideType::InOut,
            prolite::api::AnimationDirection::RightToLeft,
            prolite::api::Interval::DPS(8),
        ),
        repeat: prolite::api::Repeat::Forever
    };

    let mut current_content = None;
    let mut content_queue = Box::new(VecDeque::new());
    let mut now = Instant::now();

    content_queue.push_back(content);


    // Loop:
    // 1. Receive new commands
    // 2. Render command and update screen
    // 3. Sleep until next frame
    loop {
        if let Some(command) = try_recv(&command_rx) {
            info!("[render] received new command {:?}", &command);

            match command {
                prolite::api::Command::AddToQueue(command) => content_queue.push_back(command),
                prolite::api::Command::ShowNow(command) => {
                    content_queue.clear();
                    current_content = Some(CurrentContent {
                        content: command,
                        start_time: now,
                    });
                }
                prolite::api::Command::Clear => {
                    content_queue.clear();
                    current_content = None;
                }
            }
        }

        if current_content.is_none() {
            if let Some(next_command) = content_queue.pop_front() {
                info!("[render] starting render of command {:?}", &next_command);
                current_content = Some(CurrentContent {
                    content: next_command,
                    start_time: now,
                })
            }
        }

        if let Some(command) = current_content.as_ref() {
            let render = renderer::render(
                &command.content,
                command.start_time,
                now,
                renderer::UnknownGlyphBehavior::ReplaceWithPlaceholder,
            );

            // todo move to better location
            if render.content_state == ContentState::Complete {
                match command.content.repeat {
                    prolite::api::Repeat::None => {
                        info!("[render] finished rendering previous command");
                        current_content = None
                    },

                    prolite::api::Repeat::Forever => {
                        current_content = Some(CurrentContent {
                            content: command.content.clone(),
                            start_time: Instant::now(),
                        })
                    },

                    prolite::api::Repeat::Times(times) => {
                        let mut content = command.content.clone();
                        content.repeat = match times {
                            ..1 => Repeat::None,
                            _ => Repeat::Times(times - 1)
                        };

                        current_content = Some(CurrentContent {
                            content,
                            start_time: Instant::now(),
                        })
                    },
                }

            }

            if render.buffer_state != ScreenBufferState::NotUpdated {
                // info!("rendered: \n{}", &render.buffer);

                send(&screen_buffer_tx, render.buffer);
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

fn initialize_uart_thread(uart_receiver: UartDriver, buffer_sender: Sender<prolite::api::Command>) {
    let mut buffer = [0u8; 256];
    info!("uart init");

    loop {
        let read = uart_receiver.read(&mut buffer, UART_READ_DELAY_TICKS);
        match read {
            Ok(0) => { /* do nothing */ }
            Ok(_) => match prolite::api::Command::deserialize(&buffer) {
                Ok(command) => send(&buffer_sender, command),
                Err(e) => info!("[uart] failed to deserialize command: {:?}", e),
            },
            Err(_) => { /* do nothing */ }
        }
    }
}

const UART_READ_DELAY_TICKS: u32 = 1000;

fn initial_buffer() -> Box<ScreenBuffer> {
    let content = Content {
        text: "Initializing".to_string(),
        color: prolite::api::Color::Orange,
        animation: prolite::api::Animation::None(prolite::api::ContentDuration::Forever),
        repeat: prolite::api::Repeat::None
    };

    renderer::render(
        &content,
        Instant::now(),
        Instant::now(),
        renderer::UnknownGlyphBehavior::Ignore,
    )
    .buffer
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
