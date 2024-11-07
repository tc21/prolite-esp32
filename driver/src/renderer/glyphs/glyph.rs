// For optimization, we store glyphs up to 9 wide in a u64:

use prolite::{Pixel, ScreenBuffer};

// For characters 9 wide, the leftmost bit is 1
// For all other widths, the leftmost 4 bits is a 4-bit integer storing (width - 1)
// (so we're using three bits to store 0-7)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Glyph {
    data: u64,
}

pub const PLACEHOLDER_GLYPH: Glyph = Glyph {
    data: 0b0110_00000000000_1100011_1011101_1111101_1111011_1110111_1111111_1110111
};

pub const EMPTY_GLYPH: Glyph = Glyph { data: 0 };

impl Glyph {
    pub const fn width(&self) -> usize {
        if self.data >> 63 == 1 {
            9
        } else {
            ((self.data >> 60) + 1) as usize
        }
    }

    pub const fn height(&self) -> usize {
        ScreenBuffer::HEIGHT
    }

    pub const fn new(data: u64) -> Self {
        Self { data }
    }

    pub fn copy_to_buffer(&self, buffer: &mut ScreenBuffer, pixel: Pixel, x: i32, y: i32) {
        let width = self.width();
        let mut data = self.data;

        // info!("{}, {}, {}", data & 1, x, y);

        for row in (0..7).rev() {
            for col in (0..(width as i32)).rev() {
                let x = x + col;
                let y = y + row;

                if data & 1 == 1 {
                    buffer.set_if_in_bounds(y, x, pixel);
                }

                data >>= 1;
            }
        }
    }
}

impl Into<Glyph> for u64 {
    fn into(self) -> Glyph {
        Glyph { data: self }
    }
}
