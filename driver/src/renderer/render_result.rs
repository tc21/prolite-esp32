use std::time::Instant;

use prolite::{api::Content, ScreenBuffer};


#[derive(Debug)]
pub struct CurrentContent {
    pub content: Content,
    pub start_time: Instant,
}

impl CurrentContent {
    pub fn new(content: Content) -> Self {
        Self {
            content,
            start_time: Instant::now(),
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
