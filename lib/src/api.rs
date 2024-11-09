use crate::{Level, Pixel};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::DurationSecondsWithFrac;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "snake_case")]
pub enum Command {
    AddToQueue { content: ContentGroup },
    ShowNow { content: ContentGroup },
    Clear,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContentGroup {
    pub contents: Vec<Content>,
    #[serde(default)]
    pub repeat: Repeat,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Content {
    pub text: String,
    #[serde(default)]
    pub color: Color,
    #[serde(default)]
    pub animation: Animation,
    #[serde(default)]
    pub align: Alignment,
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
#[serde(rename_all = "snake_case")]
pub enum Alignment {
    Left,
    Center,
    Right,
}

impl Default for Alignment {
    fn default() -> Self {
        Alignment::Center
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
        direction: SlideDirection,
        #[serde(default)]
        speed: SlideSpeed,
    },
    SlideInBounds {
        #[serde(default)]
        direction: SlideInBoundsDirection,
        #[serde(default)]
        speed: SlideSpeed,
    }
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
pub enum SlideDirection {
    TopToBottom,
    BottomToTop,
    LeftToRight,
    RightToLeft,
}

impl Default for SlideDirection {
    fn default() -> Self {
        SlideDirection::RightToLeft
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SlideInBoundsDirection {
    Forward,
    Reverse,
}

impl Default for SlideInBoundsDirection {
    fn default() -> Self {
        SlideInBoundsDirection::Forward
    }
}

#[serde_as]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SlideSpeed {
    Duration(#[serde_as(as = "DurationSecondsWithFrac<f64>")] Duration),
    Dps(usize),
}

impl Default for SlideSpeed {
    fn default() -> Self {
        SlideSpeed::Dps(12)
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
