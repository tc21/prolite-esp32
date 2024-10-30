use std::time::Instant;

use crate::protocol::{
    drive::{Level, Pixel, Screen},
    CommandState, DisplayCommand, RenderResult, ScreenState,
};

pub fn render(
    command: &DisplayCommand,
    start_time: Instant,
    current_time: Instant,
) -> RenderResult {
    // todo!()
    let on = Pixel {
        red: Level::On,
        green: Level::On,
    };
    let off = Pixel {
        red: Level::Off,
        green: Level::Off,
    };
    let mut screen = Box::new(Screen([[off; 80]; 7]));

    for i in 0..80 {
        let row = ((i as i32 % 12) - 6).abs() as usize;
        screen[row][i] = on;
    }

    RenderResult {
        screen,
        command_state: CommandState::InAction,
        screen_state: ScreenState::Updated,
    }
}
