use esp_idf_svc::hal;
use prolite::Level;

use crate::config::*;

pub struct ControlPins {
    // I haven't been able to figure out how to configure these pins dynamically
    pub r: RedPin,
    pub g: GreenPin,
    pub row_0: Row0Pin,
    pub row_1: Row1Pin,
    pub row_2: Row2Pin,
    pub clk: ClockPin,
    pub screen: ScreenPin,
}

pub trait ToGpioLevel {
    fn to_gpio_level(self) -> hal::gpio::Level;
}

impl ToGpioLevel for Level {
    // we wrote this so we can have multiple levels, not just low and high,
    // but if we end up with just two levels, we can do away with this layer of abstraction
    fn to_gpio_level(self) -> hal::gpio::Level {
        match self {
            Level::Off => hal::gpio::Level::Low,
            Level::On => hal::gpio::Level::High,
        }
    }
}
