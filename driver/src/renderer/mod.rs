use std::time::Duration;

use glyphs::RenderedGlyphs;
// use current_content::CurrentContent;
pub use glyphs::UnknownGlyphBehavior;

use prolite::{
    api::{Animation, Color},
    Pixel, ScreenBuffer,
};

mod animations;
pub mod current_content;
pub mod glyphs;

pub fn render(
    rendered_glyphs: &RenderedGlyphs,
    color: &Color,
    animation: &Animation,
    duration: Option<Duration>,
    time_elapsed: Duration,
) -> Box<ScreenBuffer> {
    let offset =
        animations::get_global_offset(animation, rendered_glyphs.width, duration, time_elapsed);

    let pixel = color.to_pixel();

    let mut buffer = Box::new(ScreenBuffer([[Pixel::default(); 80]; 7]));

    for rendered_glyph in &rendered_glyphs.glyphs {
        let glyph = rendered_glyph.glyph;

        let start_col = rendered_glyph.x_offset as i32 + offset.x;
        let start_row = offset.y;

        glyph.copy_to_buffer(&mut buffer, pixel, start_col, start_row);
    }

    buffer
}
