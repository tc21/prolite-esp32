use std::time::Duration;

use esp_idf_svc::hal::{
    self,
    gpio::{Level, PinDriver},
};

pub type RedPin = PinDriver<'static, hal::gpio::Gpio4, hal::gpio::Output>;
pub type GreenPin = PinDriver<'static, hal::gpio::Gpio5, hal::gpio::Output>;
pub type Row0Pin = PinDriver<'static, hal::gpio::Gpio11, hal::gpio::Output>;
pub type Row1Pin = PinDriver<'static, hal::gpio::Gpio10, hal::gpio::Output>;
pub type Row2Pin = PinDriver<'static, hal::gpio::Gpio9, hal::gpio::Output>;
pub type ClockPin = PinDriver<'static, hal::gpio::Gpio18, hal::gpio::Output>;
pub type ScreenPin = PinDriver<'static, hal::gpio::Gpio8, hal::gpio::Output>;

pub const CLOCK_DELAY_US: u32 = 1;
pub const ROW_DELAY_US: u32 = 200;
pub const RENDER_FRAMERATE: Duration = Duration::from_micros(41667);

pub const CONTROL_SIGNALS_BY_ROW: [(Level, Level, Level); 7] = [
    (Level::High, Level::High, Level::High),
    (Level::High, Level::High, Level::Low),
    (Level::High, Level::Low, Level::High),
    (Level::High, Level::Low, Level::Low),
    (Level::Low, Level::High, Level::High),
    (Level::Low, Level::High, Level::Low),
    (Level::Low, Level::Low, Level::High),
];
