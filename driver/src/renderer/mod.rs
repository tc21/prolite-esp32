use std::time::Duration;

use glyphs::RenderedGlyphs;
pub use glyphs::UnknownGlyphBehavior;

use prolite::{api::Content, Pixel, ScreenBuffer};

mod animations;
pub mod current_content;
pub mod glyphs;

pub fn render(
    content: &Content,
    rendered_glyphs: &RenderedGlyphs,
    duration: Option<Duration>,
    time_elapsed: Duration,
) -> Box<ScreenBuffer> {
    let offset = animations::get_global_offset(
        &content.animation,
        content.align,
        rendered_glyphs.width,
        duration,
        time_elapsed,
    );

    let pixel = content.color.to_pixel();

    let mut buffer = Box::new(ScreenBuffer([[Pixel::default(); 80]; 7]));

    for rendered_glyph in &rendered_glyphs.glyphs {
        let glyph = rendered_glyph.glyph;

        let start_col = rendered_glyph.x_offset as i32 + offset.x;
        let start_row = offset.y;

        glyph.copy_to_buffer(&mut buffer, pixel, start_col, start_row);
    }

    buffer
}
