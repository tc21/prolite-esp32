mod generated;
mod generated_extra;
mod glyph;

use generated::CHARS;
use generated::CHARS_MAX;
use generated_extra::CHARS_EXTRA;
use glyph::EMPTY_GLYPH;
pub use glyph::{Glyph, PLACEHOLDER_GLYPH};

fn get_glyph(c: char) -> Option<Glyph> {
    let codepoint = c as usize;

    if codepoint >= CHARS_MAX {
        CHARS_EXTRA.get(&c)
            .map(|glyph| *glyph)
    } else {
        match CHARS[codepoint] {
            EMPTY_GLYPH => None,
            x => Some(x)
        }
    }
}

pub fn get_glyph_placement(text: &str, behavior: UnknownGlyphBehavior) -> PlacedGlyphs {
    let mut width = 0;
    let mut glyphs = vec![];

    for c in text.chars() {
        match get_glyph(c) {
            Some(glyph) => {
                glyphs.push(PlacedGlyph {
                    glyph,
                    x_offset: width,
                });
                width += glyph.width()
            }
            None => {
                match behavior {
                    UnknownGlyphBehavior::ReplaceWithPlaceholder => {
                        glyphs.push(PlacedGlyph {
                            glyph: PLACEHOLDER_GLYPH,
                            x_offset: width,
                        });
                        width +=  PLACEHOLDER_GLYPH.width()
                    }
                    UnknownGlyphBehavior::Ignore => { /* do nothing */ }
                }
            }
        }

        // add spaces between characters
        width += 1
    }

    // remove the extra spaced added at the end
    width -= 1;
    PlacedGlyphs { glyphs, width }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnknownGlyphBehavior {
    ReplaceWithPlaceholder,
    Ignore,
}

#[derive(Debug)]
pub struct PlacedGlyph {
    pub glyph: Glyph,
    pub x_offset: usize,
}

#[derive(Debug)]
pub struct PlacedGlyphs {
    pub glyphs: Vec<PlacedGlyph>,
    pub width: usize,
}
