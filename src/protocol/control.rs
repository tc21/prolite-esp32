use std::time::{Duration, Instant};

use super::drive::ScreenBuffer;

#[derive(Debug)]
pub enum Command {
    AddToQueue(Content),
    ShowNow(Content),
    Clear,
}

#[derive(Debug)]
pub struct Content {
    pub text: String,
    pub animation: Animation,
    pub duration: ContentDuration,
}

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
    pub command_state: ContentState,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Animation {
    None,
    SlideIn(AnimationDirection, AnimationSpeed),
    SlideOut(AnimationDirection, AnimationSpeed),
    SlideThrough(AnimationDirection, AnimationSpeed),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationDirection {
    TopToBottom,
    BottomToTop,
    LeftToRight,
    RightToLeft,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationSpeed {
    // Clamps to duration if duration != UntilAnimationEnd
    // otherwise equivalent to TODO(temporarily set at 80dps)
    Natural,
    DPS(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentDuration {
    Finite(Duration),
    UntilAnimationEnd,
    Forever,
}
