use std::time::Duration;
use serde::{Serialize, Deserialize};
use crate::{Level, Pixel};

#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    AddToQueue(Content),
    ShowNow(Content),
    Clear,
}

impl Command {
    pub fn serialize(&self) -> bincode::Result<Vec<u8>> {
        bincode::serialize(self)
    }

    pub fn deserialize(bytes: &[u8]) -> bincode::Result<Self> {
        bincode::deserialize(bytes)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Content {
    pub text: String,
    pub color: Color,
    pub animation: Animation,
    pub repeat: Repeat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Repeat {
    None,
    Forever,
    Times(usize)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Color {
    Red,
    Green,
    Orange
}

impl Color {
    pub fn to_pixel(self) -> Pixel {
        match self {
            Color::Red => Pixel { red: Level::On, green: Level::Off },
            Color::Green => Pixel { red: Level::Off, green: Level::On },
            Color::Orange => Pixel { red: Level::On, green: Level::On },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Animation {
    None(ContentDuration),
    Slide(SlideType, AnimationDirection, Interval)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlideType {
    In,
    Out,
    InOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationDirection {
    TopToBottom,
    BottomToTop,
    LeftToRight,
    RightToLeft,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Interval {
    Duration(Duration),
    DPS(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentDuration {
    Finite(Duration),
    Forever,
}
