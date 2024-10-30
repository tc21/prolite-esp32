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
use protocol::{drive::Screen, Command, CommandInAction, CommandState, ScreenState};

mod config;
mod control;
mod display;
mod driver;
mod network;
mod protocol;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello, world!");

    let peripherals = hal::prelude::Peripherals::take().unwrap();
    let (command_tx, command_rx) = mpsc::channel();
    let (screen_tx, screen_rx) = mpsc::channel();

    let (continuation_tx, continuation_rx) = mpsc::channel();

    let wifi_config = WifiConfig {
        ssid: env!("WIFI_SSID"),
        password: env!("WIFI_PASSWORD"),
    };

    // numerous unwraps occur here:
    // if the initialize process fails, simply reboot and try again

    // networking is heavy
    thread::Builder::new()
        .stack_size(1024 * 4)
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
        .stack_size(1024 * 96)
        .spawn(move || {
            init_driver_thread(screen_rx, control_pins);
        })
        .unwrap();

    thread::Builder::new()
        .stack_size(1024 * 8)
        .spawn(move || {
            init_display_thread(command_rx, screen_tx);
        })
        .unwrap();
}

const MAX_RETRY_ATTEMPTS: usize = 3;

fn init_networking_thread(
    config: WifiConfig,
    modem: impl hal::peripheral::Peripheral<P = hal::modem::Modem> + 'static,
    command_tx: Sender<protocol::Command>,
    continuation_tx: Sender<i32>,
) -> Result<(), EspError> {
    let mut connection = network::establish_wifi_connection(config.ssid, config.password, modem)?;
    let mut _server = control::establish_control_server(command_tx)?;

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

const SCREEN_UPDATE_DELAY: Duration = Duration::from_micros(200);
const DISPLAY_PROCESSING_RATE: Duration = Duration::from_millis(20);

fn init_display_thread(command_rx: Receiver<Command>, screen_tx: Sender<Box<Screen>>) {
    let mut current_command = None;
    let mut command_queue = VecDeque::new();
    let mut now = Instant::now();

    // Loop:
    // 1. Receive new commands
    // 2. Render command and update screen
    // 3. Sleep until next frame
    loop {
        if let Some(command) = try_recv(&command_rx) {
            info!("[display] received new command {:?}", &command);

            match command {
                Command::DisplayInQueue(command) => command_queue.push_back(command),
                Command::DisplayNow(command) => {
                    command_queue.clear();
                    current_command = Some(CommandInAction {
                        command,
                        start_time: now,
                    });
                }
                Command::Clear => {
                    command_queue.clear();
                    current_command = None;
                }
            }
        }

        if current_command.is_none() {
            if let Some(next_command) = command_queue.pop_front() {
                info!("[display] starting render of command {:?}", &next_command);
                current_command = Some(CommandInAction {
                    command: next_command,
                    start_time: now,
                })
            }
        }

        if let Some(command) = current_command.as_ref() {
            let render = display::render(&command.command, command.start_time, now);

            if render.command_state == CommandState::Finished {
                info!("[display] finished rendering previous command");
                current_command = None
            }

            if render.screen_state != ScreenState::Unchanged {
                send(&screen_tx, render.screen);
            }
        }

        let elapsed = now.elapsed();

        if elapsed < DISPLAY_PROCESSING_RATE {
            thread::sleep(DISPLAY_PROCESSING_RATE - elapsed);
            now = now.checked_add(DISPLAY_PROCESSING_RATE).unwrap();
        } else {
            now = Instant::now();
        }
    }
}

fn init_driver_thread(screen_rx: Receiver<Box<Screen>>, mut control_pins: ControlPins) {
    let mut screen = Box::new(protocol::drive::Screen::new());

    loop {
        if let Some(new_screen) = try_recv(&screen_rx) {
            screen = new_screen
        }

        match driver::display_screen(&screen, &mut control_pins) {
            Ok(_) => { /* do nothing */ }
            Err(e) => info!("failed to drive screen: {:?}", e),
        };

        thread::sleep(SCREEN_UPDATE_DELAY);
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
