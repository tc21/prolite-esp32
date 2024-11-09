use std::time::Duration;

use prolite::ScreenBuffer;

use prolite::api::{Alignment, Animation, SlideDirection, SlideInBoundsDirection, SlideType};

#[derive(Debug)]
pub struct Offset {
    pub x: i32,
    pub y: i32,
}

pub fn get_global_offset(
    animation: &Animation,
    default_alignment: Alignment,
    rendered_width: usize,
    duration: Option<Duration>,
    time_elapsed: Duration,
) -> Offset {
    let default_offset = get_default_offset(default_alignment, rendered_width);

    match animation {
        Animation::None { .. } => default_offset,
        Animation::Slide {
            slide_type,
            direction,
            ..
        } => {
            // not sure if making these into functions is even worth it,
            // it may be too complicated to optimize away
            let top_position = || Offset {
                x: default_offset.x,
                y: -(ScreenBuffer::HEIGHT as i32),
            };

            let bottom_position = || Offset {
                x: default_offset.x,
                y: ScreenBuffer::HEIGHT as i32,
            };

            let left_position = || Offset {
                x: -(rendered_width as i32),
                y: default_offset.y,
            };

            let right_position = || Offset {
                x: ScreenBuffer::WIDTH as i32,
                y: default_offset.y,
            };

            let (altered_start_offset, altered_end_offset) = match direction {
                SlideDirection::TopToBottom => (top_position(), bottom_position()),
                SlideDirection::BottomToTop => (bottom_position(), top_position()),
                SlideDirection::LeftToRight => (left_position(), right_position()),
                SlideDirection::RightToLeft => (right_position(), left_position()),
            };

            let (start_offset, end_offset) = match slide_type {
                SlideType::In => (altered_start_offset, default_offset),
                SlideType::Out => (default_offset, altered_end_offset),
                SlideType::InOut => (altered_start_offset, altered_end_offset),
            };

            let duration = duration.unwrap_or_default();

            get_offset_for_linear_movement(start_offset, end_offset, duration, time_elapsed)
        }
        Animation::SlideInBounds { direction, .. } => {
            let left_aligned = get_default_offset(Alignment::Left, rendered_width);
            let right_aligned = get_default_offset(Alignment::Right, rendered_width);

            let (start_offset, end_offset) = match direction {
                SlideInBoundsDirection::Forward => (left_aligned, right_aligned),
                SlideInBoundsDirection::Reverse => (right_aligned, left_aligned),
            };

            let duration = duration.unwrap_or_default();

            get_offset_for_linear_movement(start_offset, end_offset, duration, time_elapsed)
        }
    }
}

pub fn get_default_offset(alignment: Alignment, rendered_width: usize) -> Offset {
    let x = match alignment {
        Alignment::Left => 0,
        Alignment::Center => {
            let mut w = rendered_width as i32;
            // Place items in center, prefer left side if cannot center:
            // screen width = 8, width = 4 -> x = 2  W/2-w/2
            // screen width = 8, width = 5 -> x = 1  W/2-(w+1)/2
            // screen width = 7, width = 4 -> x = 1  W/2-w/2
            // screen width = 7, width = 5 -> x = 1  W/2-w/2
            let midpoint = (ScreenBuffer::WIDTH / 2) as i32;
            if (midpoint & 1) ^ (w & 1) == 1 {
                w += 1;
            }

            midpoint - w / 2
        }
        Alignment::Right => ScreenBuffer::WIDTH as i32 - rendered_width as i32,
    };

    Offset { x, y: 0 }
}

fn get_offset_for_linear_movement(
    start: Offset,
    end: Offset,
    duration: Duration,
    time_elapsed: Duration,
) -> Offset {
    let progress = time_elapsed.div_duration_f32(duration);
    let x = start.x + ((end.x - start.x) as f32 * progress).round() as i32;
    let y = start.y + ((end.y - start.y) as f32 * progress).round() as i32;

    Offset { x, y }
}
