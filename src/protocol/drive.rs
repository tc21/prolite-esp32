use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Copy)]
pub struct Pixel {
    pub red: Level,
    pub green: Level
}

impl Default for Pixel {
    fn default() -> Self {
        Self { red: Level::Off, green: Level::Off }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Off,
    On
}

#[derive(Debug)]
pub struct Screen([[Pixel; 80]; 5]);

impl Screen {
    pub fn new() -> Self {
        Self([[Pixel::default(); 80]; 5])
    }
}

impl Deref for Screen {
    type Target = [[Pixel; 80]; 5];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Screen {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
