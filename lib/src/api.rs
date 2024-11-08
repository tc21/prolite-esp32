use crate::{Level, Pixel};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::DurationSecondsWithFrac;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "snake_case")]
pub enum Command {
    AddToQueue { content: Content },
    ShowNow { content: Content },
    Clear,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Content {
    pub text: String,
    #[serde(default)]
    pub color: Color,
    #[serde(default)]
    pub animation: Animation,
    #[serde(default)]
    pub repeat: Repeat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Repeat {
    None,
    Forever,
    Times(usize),
}

impl Default for Repeat {
    fn default() -> Self {
        Repeat::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Color {
    Red,
    Green,
    Orange,
}

impl Color {
    pub fn to_pixel(self) -> Pixel {
        match self {
            Color::Red => Pixel {
                red: Level::On,
                green: Level::Off,
            },
            Color::Green => Pixel {
                red: Level::Off,
                green: Level::On,
            },
            Color::Orange => Pixel {
                red: Level::On,
                green: Level::On,
            },
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::Orange
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Animation {
    None {
        #[serde(default)]
        duration: ContentDuration,
    },
    Slide {
        #[serde(default)]
        slide_type: SlideType,
        #[serde(default)]
        direction: AnimationDirection,
        #[serde(default)]
        interval: Interval,
    },
}

impl Default for Animation {
    fn default() -> Self {
        Animation::None {
            duration: ContentDuration::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SlideType {
    In,
    Out,
    InOut,
}

impl Default for SlideType {
    fn default() -> Self {
        SlideType::InOut
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimationDirection {
    TopToBottom,
    BottomToTop,
    LeftToRight,
    RightToLeft,
}

impl Default for AnimationDirection {
    fn default() -> Self {
        AnimationDirection::RightToLeft
    }
}

#[serde_as]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Interval {
    Duration(#[serde_as(as = "DurationSecondsWithFrac<f64>")] Duration),
    Dps(usize),
}

impl Default for Interval {
    fn default() -> Self {
        Interval::Dps(12)
    }
}

#[serde_as]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentDuration {
    Duration(#[serde_as(as = "DurationSecondsWithFrac<f64>")] Duration),
    Forever,
}

impl Default for ContentDuration {
    fn default() -> Self {
        ContentDuration::Forever
    }
}
