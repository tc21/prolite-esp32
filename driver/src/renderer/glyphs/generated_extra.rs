use std::collections::HashMap;

use lazy_static::lazy_static;

use super::Glyph;

lazy_static! {
    pub static ref CHARS_EXTRA: HashMap<char, Glyph> = chars();
}

pub fn chars() -> HashMap<char, Glyph> {
    let mut m = HashMap::new();

    m.insert('🦊', 0b0110000000000000100010011011001111101101011111111100111000001000.into());

    m
}
