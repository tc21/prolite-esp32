use std::{thread, time::Duration};

use config::WifiConfig;
use esp_idf_svc::{
    hal::{
        self,
        gpio::{AnyInputPin, AnyOutputPin},
        uart::{config::Config, UartDriver},
    },
    sys::EspError,
};
use log::info;

mod config;
mod controller;
mod network;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("hello world");

    let peripherals = hal::prelude::Peripherals::take().unwrap();

    let uart_tx = UartDriver::new(
        peripherals.uart1,
        peripherals.pins.gpio4,
        peripherals.pins.gpio5,
        Option::<AnyInputPin>::None,
        Option::<AnyOutputPin>::None,
        &Config::default(),
    )
    .unwrap();

    let config = WifiConfig {
        ssid: env!("WIFI_SSID"),
        password: env!("WIFI_PASSWORD"),
    };

    let mut connection =
        network::establish_wifi_connection(config.ssid, config.password, peripherals.modem)
            .unwrap();

    let ip_address = connection.sta_netif().get_ip_info().unwrap().ip;
    let mut _server = controller::establish_control_server(uart_tx, ip_address).unwrap();

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

const MAX_RETRY_ATTEMPTS: usize = 3;

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
