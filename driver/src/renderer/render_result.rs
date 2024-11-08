use std::time::Instant;

use prolite::{api::Content, ScreenBuffer};

use super::{
    glyphs::{get_glyph_placement, PlacedGlyphs},
    UnknownGlyphBehavior,
};

#[derive(Debug)]
pub struct CurrentContent {
    pub content: Content,
    pub start_time: Instant,
    pub rendered_glyphs: PlacedGlyphs,
}

impl CurrentContent {
    pub fn new(content: Content, behavior: UnknownGlyphBehavior) -> Self {
        let rendered_glyphs = get_glyph_placement(&content.text, behavior);

        Self {
            content,
            start_time: Instant::now(),
            rendered_glyphs,
        }
    }
}

#[derive(Debug)]
pub struct RenderResult {
    pub buffer: Box<ScreenBuffer>,
    pub content_state: ContentState,
    pub buffer_state: ScreenBufferState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentState {
    Incomplete,
    Complete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenBufferState {
    Updated,
    NotUpdated,
}
