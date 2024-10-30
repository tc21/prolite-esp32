use std::time::Instant;

use crate::protocol::{
    drive::{Level, Pixel, ScreenBuffer},
    Content, ContentState, RenderResult, ScreenBufferState,
};

pub fn render(content: &Content, start_time: Instant, current_time: Instant) -> RenderResult {
    // todo!()
    let on = Pixel {
        red: Level::On,
        green: Level::On,
    };
    let off = Pixel {
        red: Level::Off,
        green: Level::Off,
    };
    let mut buffer = Box::new(ScreenBuffer([[off; 80]; 7]));

    for i in 0..80 {
        let row = ((i as i32 % 12) - 6).abs() as usize;
        buffer[row][i] = on;
    }

    RenderResult {
        buffer,
        command_state: ContentState::Incomplete,
        buffer_state: ScreenBufferState::Updated,
    }
}
