use std::time::{Duration, Instant};

use super::drive::Screen;

#[derive(Debug)]
pub enum Command {
    DisplayInQueue(DisplayCommand),
    DisplayNow(DisplayCommand),
    Clear,
}

#[derive(Debug)]
pub struct DisplayCommand {
    pub text: String,
    pub animation: Animation,
    pub duration: DisplayDuration,
}

#[derive(Debug)]
pub struct CommandInAction {
    pub command: Command,
    pub start_time: Instant,
}

impl CommandInAction {
    pub fn new(command: Command) -> Self {
        Self {
            command,
            start_time: Instant::now(),
        }
    }
}

#[derive(Debug)]
pub struct RenderResult {
    pub screen: Screen,
    pub command_state: CommandState,
    pub screen_state: ScreenState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandState {
    InAction,
    Finished,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenState {
    Updated,
    Unchanged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Animation {
    None,
    SlideIn(Direction, Speed),
    SlideOut(Direction, Speed),
    SlideThrough(Direction, Speed),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    TopToBottom,
    BottomToTop,
    LeftToRight,
    RightToLeft,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Speed {
    // Clamps to duration if duration != UntilAnimationEnd
    // otherwise equivalent to TODO(temporarily set at 80dps)
    Natural,
    DPS(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayDuration {
    Finite(Duration),
    UntilAnimationEnd,
    Forever,
}
