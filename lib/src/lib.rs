use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

pub mod api;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pixel {
    pub red: Level,
    pub green: Level,
}

impl Pixel {
    const OFF: Pixel = Self {
        red: Level::Off,
        green: Level::Off,
    };

    fn serialize(&self) -> u8 {
        (if self.red == Level::On { 1 } else { 0 }) + (if self.green == Level::On { 2 } else { 0 })
    }

    fn deserialize(x: u8) -> Self {
        Self {
            red: if x & 1 == 1 { Level::On } else { Level::Off },
            green: if (x << 1) & 1 == 1 {
                Level::On
            } else {
                Level::Off
            },
        }
    }
}

impl Default for Pixel {
    fn default() -> Self {
        Pixel::OFF
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Off,
    On,
}

const DISPLAY_WIDTH: usize = 80;
const DISPLAY_HEIGHT: usize = 7;

#[derive(Debug)]
pub struct ScreenBuffer(pub [[Pixel; DISPLAY_WIDTH]; DISPLAY_HEIGHT]);

impl ScreenBuffer {
    pub const WIDTH: usize = DISPLAY_WIDTH;
    pub const HEIGHT: usize = DISPLAY_HEIGHT;

    pub fn new() -> Self {
        Self([[Pixel::default(); DISPLAY_WIDTH]; DISPLAY_HEIGHT])
    }

    pub fn set_if_in_bounds(&mut self, row: i32, col: i32, pixel: Pixel) {
        if row < 0 || col < 0 || row >= DISPLAY_HEIGHT as i32 || col >= DISPLAY_WIDTH as i32 {
            return;
        }

        self[row as usize][col as usize] = pixel;
    }

    pub fn serialize(&self) -> [u8; DISPLAY_HEIGHT * DISPLAY_WIDTH] {
        let mut result = [0u8; DISPLAY_HEIGHT * DISPLAY_WIDTH];

        for i in 0..(DISPLAY_HEIGHT * DISPLAY_WIDTH) {
            let row = i / DISPLAY_WIDTH;
            let col = i % DISPLAY_WIDTH;
            result[i] = self[row][col].serialize();
        }

        result
    }

    pub fn deserialize(s: [u8; DISPLAY_HEIGHT * DISPLAY_WIDTH]) -> Self {
        let mut result = Self::new();

        for i in 0..(DISPLAY_HEIGHT * DISPLAY_WIDTH) {
            let row = i / DISPLAY_WIDTH;
            let col = i % DISPLAY_WIDTH;
            result[row][col] = Pixel::deserialize(s[i]);
        }

        result
    }
}

impl Display for ScreenBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let red = Pixel {
            red: Level::On,
            green: Level::Off,
        };
        let green = Pixel {
            red: Level::Off,
            green: Level::On,
        };
        let orange = Pixel {
            red: Level::On,
            green: Level::On,
        };

        for row in 0..DISPLAY_HEIGHT {
            for col in 0..DISPLAY_WIDTH {
                if self[row][col] == Pixel::OFF {
                    write!(f, ".")?;
                } else if self[row][col] == red {
                    write!(f, "R")?;
                } else if self[row][col] == green {
                    write!(f, "G")?;
                } else if self[row][col] == orange {
                    write!(f, "O")?;
                } else {
                    write!(f, "?")?;
                }
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}

impl Deref for ScreenBuffer {
    type Target = [[Pixel; DISPLAY_WIDTH]; DISPLAY_HEIGHT];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ScreenBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
