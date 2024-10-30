use std::{
    collections::VecDeque,
    sync::mpsc::{self, Receiver, Sender, TryRecvError},
    thread,
    time::{Duration, Instant},
};

use config::WifiConfig;
use driver::ControlPins;
use esp_idf_svc::{
    hal::{self, gpio::PinDriver},
    sys::EspError,
};
use log::info;
use protocol::{drive::ScreenBuffer, Command, ContentState, CurrentContent, ScreenBufferState};

mod config;
mod controller;
mod driver;
mod network;
mod protocol;
mod renderer;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello, world!");

    let peripherals = hal::prelude::Peripherals::take().unwrap();
    let (command_tx, command_rx) = mpsc::channel();
    let (screen_buffer_tx, screen_buffer_rx) = mpsc::channel();

    let (continuation_tx, continuation_rx) = mpsc::channel();

    let wifi_config = WifiConfig {
        ssid: env!("WIFI_SSID"),
        password: env!("WIFI_PASSWORD"),
    };

    // numerous unwraps occur here:
    // if the initialize process fails, simply reboot and try again

    thread::Builder::new()
        .name("networking".to_owned())
        .stack_size(THREAD_STACK_SIZE)
        .spawn(move || {
            init_networking_thread(wifi_config, peripherals.modem, command_tx, continuation_tx)
                .unwrap();
        })
        .unwrap();

    // wait for networking setup to complete before setting up new threads
    continuation_rx.recv().unwrap();

    let control_pins = ControlPins {
        r: PinDriver::output(peripherals.pins.gpio15).unwrap(),
        g: PinDriver::output(peripherals.pins.gpio2).unwrap(),

        row_0: PinDriver::output(peripherals.pins.gpio4).unwrap(),
        row_1: PinDriver::output(peripherals.pins.gpio16).unwrap(),
        row_2: PinDriver::output(peripherals.pins.gpio17).unwrap(),

        clk: PinDriver::output(peripherals.pins.gpio18).unwrap(),
    };

    thread::Builder::new()
        .name("driver".to_owned())
        .stack_size(THREAD_STACK_SIZE)
        .spawn(move || {
            init_driver_thread(screen_buffer_rx, control_pins);
        })
        .unwrap();

    thread::Builder::new()
        .name("renderer".to_owned())
        .stack_size(THREAD_STACK_SIZE)
        .spawn(move || {
            init_renderer_thread(command_rx, screen_buffer_tx);
        })
        .unwrap();
}

const MAX_RETRY_ATTEMPTS: usize = 3;
const THREAD_STACK_SIZE: usize = 16 * 1024;

fn init_networking_thread(
    config: WifiConfig,
    modem: impl hal::peripheral::Peripheral<P = hal::modem::Modem> + 'static,
    command_tx: Sender<protocol::Command>,
    continuation_tx: Sender<i32>,
) -> Result<(), EspError> {
    let mut connection = network::establish_wifi_connection(config.ssid, config.password, modem)?;
    let mut _server = controller::establish_control_server(command_tx)?;

    send(&continuation_tx, 1);

    loop {
        retry(MAX_RETRY_ATTEMPTS, || {
            if !connection.is_connected()? {
                connection.connect()?;
            }

            Ok(())
        })
        .unwrap();

        thread::sleep(Duration::from_secs(10));
    }
}

const SCREEN_UPDATE_DELAY_MIN: Duration = Duration::from_micros(200);
const RENDER_FRAME_DELAY: Duration = Duration::from_millis(20);

fn init_renderer_thread(
    command_rx: Receiver<Command>,
    screen_buffer_tx: Sender<Box<ScreenBuffer>>,
) {
    let mut current_content = None;
    let mut content_queue = VecDeque::new();
    let mut now = Instant::now();

    // Loop:
    // 1. Receive new commands
    // 2. Render command and update screen
    // 3. Sleep until next frame
    loop {
        if let Some(command) = try_recv(&command_rx) {
            info!("[render] received new command {:?}", &command);

            match command {
                Command::AddToQueue(command) => content_queue.push_back(command),
                Command::ShowNow(command) => {
                    content_queue.clear();
                    current_content = Some(CurrentContent {
                        content: command,
                        start_time: now,
                    });
                }
                Command::Clear => {
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
            let render = renderer::render(&command.content, command.start_time, now);

            if render.command_state == ContentState::Complete {
                info!("[render] finished rendering previous command");
                current_content = None
            }

            if render.buffer_state != ScreenBufferState::NotUpdated {
                send(&screen_buffer_tx, render.buffer);
            }
        }

        let elapsed = now.elapsed();

        if elapsed < RENDER_FRAME_DELAY {
            thread::sleep(RENDER_FRAME_DELAY - elapsed);
            now = now.checked_add(RENDER_FRAME_DELAY).unwrap();
        } else {
            now = Instant::now();
        }
    }
}

fn init_driver_thread(
    screen_buffer_rx: Receiver<Box<ScreenBuffer>>,
    mut control_pins: ControlPins,
) {
    let mut buffer = Box::new(protocol::drive::ScreenBuffer::new());

    loop {
        if let Some(new_buffer) = try_recv(&screen_buffer_rx) {
            buffer = new_buffer
        }

        match driver::display_screen(&buffer, &mut control_pins) {
            Ok(_) => { /* do nothing */ }
            Err(e) => info!("failed to drive screen: {:?}", e),
        };

        thread::sleep(SCREEN_UPDATE_DELAY_MIN);
    }
}

fn retry<T>(
    max_attempts: usize,
    mut f: impl FnMut() -> Result<T, EspError>,
) -> Result<T, EspError> {
    for attempt in 1..=max_attempts {
        let result = f();

        if result.is_ok() || attempt == max_attempts {
            return result;
        }
    }

    panic!("unreachable code")
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
        }
    }
}
