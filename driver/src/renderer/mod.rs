use std::time::{Duration, Instant};

pub use glyphs::UnknownGlyphBehavior;

use prolite::{Pixel, ScreenBuffer};

use prolite::api::{Animation, AnimationDirection, Content, ContentDuration, Interval};
use render_result::{ContentState, CurrentContent, RenderResult, ScreenBufferState};

mod animations;
pub mod glyphs;
pub mod render_result;

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
                    prolite::api::SlideType::In | prolite::api::SlideType::Out,
                    AnimationDirection::TopToBottom | AnimationDirection::BottomToTop,
                ) => ScreenBuffer::HEIGHT,
                (
                    prolite::api::SlideType::In | prolite::api::SlideType::Out,
                    AnimationDirection::LeftToRight | AnimationDirection::RightToLeft,
                ) => (ScreenBuffer::WIDTH + rendered_width) / 2,
                (
                    prolite::api::SlideType::InOut,
                    AnimationDirection::TopToBottom | AnimationDirection::BottomToTop,
                ) => 2 * ScreenBuffer::HEIGHT,
                (
                    prolite::api::SlideType::InOut,
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

pub fn render(
    current_content: &CurrentContent,
    current_time: Instant,
) -> RenderResult {
    let content = current_content.content();
    let duration = get_duration(content, current_content.rendered_glyphs.width);
    let time_elapsed = current_time - current_content.step_start_time;
    let global_offset = animations::get_global_offset(
        &content.animation,
        current_content.rendered_glyphs.width,
        duration,
        time_elapsed,
    );

    let pixel = content.color.to_pixel();

    let mut buffer = Box::new(ScreenBuffer([[Pixel::default(); 80]; 7]));

    for placed_glyph in &current_content.rendered_glyphs.glyphs {
        let glyph = placed_glyph.glyph;

        let start_col = placed_glyph.x_offset as i32 + global_offset.x;
        let start_row = global_offset.y;

        glyph.copy_to_buffer(&mut buffer, pixel, start_col, start_row);
    }

    // for i in 0..80 {
    //     let row = ((i as i32 % 12) - 6).abs() as usize;
    //     buffer[row][i] = on;
    // }

    let mut buffer_state = ScreenBufferState::Updated;

    if let Animation::None { .. } = content.animation {
        // this is nonzero to account for the time between starting the render and now
        if time_elapsed > Duration::from_millis(200) {
            buffer_state = ScreenBufferState::NotUpdated;
        }
    }

    let content_state = if duration.is_some_and(|d| time_elapsed > d) {
        ContentState::Complete
    } else {
        ContentState::Incomplete
    };

    RenderResult {
        buffer,
        content_state,
        buffer_state,
    }
}
