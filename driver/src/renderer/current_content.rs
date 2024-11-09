use std::time::{Duration, Instant};

use prolite::{
    api::{
        Animation, AnimationDirection, Content, ContentDuration, ContentGroup, Interval, Repeat,
        SlideType,
    },
    ScreenBuffer,
};

use super::{
    glyphs::{get_glyph_placement, RenderedGlyphs},
    UnknownGlyphBehavior,
};

#[derive(Debug)]
pub struct CurrentContent {
    content_group: ContentGroup,
    step: usize,
    pub step_start_time: Instant,
    pub step_duration: Option<Duration>,
    pub rendered_glyphs: RenderedGlyphs,
    behavior: UnknownGlyphBehavior,

    initialized: bool,
}

impl CurrentContent {
    pub fn new(content_group: ContentGroup, behavior: UnknownGlyphBehavior) -> Self {
        Self {
            content_group,
            step: 0,
            step_start_time: Instant::now(),
            step_duration: None,
            rendered_glyphs: RenderedGlyphs {
                glyphs: vec![],
                width: 0,
            },
            behavior,
            initialized: false,
        }
    }

    pub fn update(&mut self, current_time: Instant) -> ContentState {
        if !self.initialized {
            self.initialize_step();
            self.initialized = true;
            return ContentState::StepStarted;
        }

        if self.step_duration.is_some()
            && current_time - self.step_start_time > self.step_duration.unwrap()
        {
            return self.step();
        }

        return ContentState::StepIncomplete;
    }

    pub fn is_animated(&self) -> bool {
        match self.content().animation {
            Animation::None { .. } => false,
            _ => true,
        }
    }

    pub fn render(&self, current_time: Instant) -> Box<ScreenBuffer> {
        let content = self.content();
        super::render(
            &self.rendered_glyphs,
            &content.color,
            &content.animation,
            self.step_duration,
            current_time - self.step_start_time,
        )
    }

    fn step(&mut self) -> ContentState {
        if self.step + 1 < self.content_group.contents.len() {
            self.step += 1;
            self.initialize_step();
            return ContentState::StepStarted;
        }

        match self.content_group.repeat {
            Repeat::None | Repeat::Times(0) => ContentState::Finished,
            Repeat::Times(n) => {
                self.content_group.repeat = Repeat::Times(n - 1);

                self.step = 0;
                self.initialize_step();
                ContentState::StepStarted
            }
            Repeat::Forever => {
                self.step = 0;
                self.initialize_step();
                ContentState::StepStarted
            }
        }
    }

    fn initialize_step(&mut self) {
        self.rendered_glyphs =
            get_glyph_placement(&self.content_group.contents[self.step].text, self.behavior);
        self.step_start_time = self.step_start_time + self.step_duration.unwrap_or(Duration::ZERO);
        self.step_duration = get_duration(self.content(), self.rendered_glyphs.width);
    }

    pub fn content(&self) -> &Content {
        &self.content_group.contents[self.step]
    }
}

#[derive(Debug)]
pub enum ContentState {
    StepStarted,
    StepIncomplete,
    Finished,
}

fn get_duration(content: &Content, rendered_width: usize) -> Option<Duration> {
    match content.animation {
        Animation::None { duration } => match duration {
            ContentDuration::Duration(duration) => Some(duration),
            ContentDuration::Forever => None,
        },
        Animation::Slide {
            interval,
            direction,
            slide_type,
        } => {
            let animated_length = match (slide_type, direction) {
                (
                    SlideType::In | SlideType::Out,
                    AnimationDirection::TopToBottom | AnimationDirection::BottomToTop,
                ) => ScreenBuffer::HEIGHT,
                (
                    SlideType::In | SlideType::Out,
                    AnimationDirection::LeftToRight | AnimationDirection::RightToLeft,
                ) => (ScreenBuffer::WIDTH + rendered_width) / 2,
                (
                    SlideType::InOut,
                    AnimationDirection::TopToBottom | AnimationDirection::BottomToTop,
                ) => 2 * ScreenBuffer::HEIGHT,
                (
                    SlideType::InOut,
                    AnimationDirection::LeftToRight | AnimationDirection::RightToLeft,
                ) => ScreenBuffer::WIDTH + rendered_width,
            };

            Some(get_finite_animation_duration(&interval, animated_length))
        }
    }
}

fn get_finite_animation_duration(interval: &Interval, animated_length: usize) -> Duration {
    match interval {
        Interval::Duration(duration) => *duration,
        Interval::Dps(dps) => Duration::from_secs_f64((animated_length as f64) / (*dps as f64)),
    }
}
