use std::ops::{Deref, DerefMut};

use esp_idf_svc::hal;

#[derive(Debug, Clone, Copy)]
pub struct Pixel {
    pub red: Level,
    pub green: Level,
}

impl Default for Pixel {
    fn default() -> Self {
        Self {
            red: Level::Off,
            green: Level::Off,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Off,
    On,
}

impl Level {
    // we wrote this so we can have multiple levels, not just low and high,
    // but if we end up with just two levels, we can do away with this layer of abstraction
    pub fn to_gpio_level(self) -> hal::gpio::Level {
        match self {
            Level::Off => hal::gpio::Level::Low,
            Level::On => hal::gpio::Level::High,
        }
    }
}

#[derive(Debug)]
pub struct ScreenBuffer(pub [[Pixel; 80]; 7]);

impl ScreenBuffer {
    pub fn new() -> Self {
        Self([[Pixel::default(); 80]; 7])
    }
}

impl Deref for ScreenBuffer {
    type Target = [[Pixel; 80]; 7];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ScreenBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
