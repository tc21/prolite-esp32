use std::time::Instant;

use prolite::{api::{Content, ContentGroup, Repeat}, ScreenBuffer};

use super::{
    glyphs::{get_glyph_placement, PlacedGlyphs},
    UnknownGlyphBehavior,
};

#[derive(Debug)]
pub struct CurrentContent {
    content_group: ContentGroup,
    pub step_start_time: Instant,
    pub rendered_glyphs: PlacedGlyphs,
    step: usize,
    behavior: UnknownGlyphBehavior
}

impl CurrentContent {
    pub fn new(content_group: ContentGroup, behavior: UnknownGlyphBehavior) -> Self {
        let rendered_glyphs = get_glyph_placement(&content_group.contents[0].text, behavior);

        Self {
            content_group,
            step: 0,
            step_start_time: Instant::now(),
            rendered_glyphs,
            behavior
        }
    }

    pub fn step(&mut self) -> ContentState {
        if self.content_group.contents.len() == self.step + 1 {
            match self.content_group.repeat {
                prolite::api::Repeat::None => ContentState::Complete,
                prolite::api::Repeat::Forever => {
                    self.step = 0;
                    self.initialize_current_step();
                    ContentState::Incomplete
                },
                prolite::api::Repeat::Times(n) => {
                    if n == 0 {
                        ContentState::Complete
                    } else {
                        self.step = 0;
                        self.initialize_current_step();
                        self.content_group.repeat = Repeat::Times(n - 1);
                        ContentState::Incomplete
                    }
                },
            }
        } else {
            self.step += 1;
            self.initialize_current_step();
            ContentState::Incomplete
        }
    }

    fn initialize_current_step(&mut self) {
        self.step_start_time = Instant::now();
        self.rendered_glyphs = get_glyph_placement(&self.content_group.contents[self.step].text, self.behavior);
    }

    pub fn content(&self) -> &Content {
        &self.content_group.contents[self.step]
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
