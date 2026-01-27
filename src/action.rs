use std::fmt::Display;

use serde::Deserialize;

#[derive(Debug, Default, Deserialize, Clone, Copy)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub enum Action {
    Quit,
    AddTorrent,
    Up,
    Down,
    GotoTop,
    GotoBottom,
    Pause,
    Start,
    PauseToggle,
    Forget,
    Delete,
    Help,
    Escape,
    Input,
    Backspace,
    #[default]
    NoOp,
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Quit => write!(f, "Quit"),
            Self::AddTorrent => write!(f, "AddTorrent"),
            Self::Up => write!(f, "Up"),
            Self::Down => write!(f, "Down"),
            Self::GotoTop => write!(f, "GotoTop"),
            Self::GotoBottom => write!(f, "GotoBottom"),
            Self::Pause => write!(f, "Pause"),
            Self::Start => write!(f, "Start"),
            Self::PauseToggle => write!(f, "PauseToggle"),
            Self::Forget => write!(f, "Forget"),
            Self::Delete => write!(f, "Delete"),
            Self::Help => write!(f, "Help"),
            Self::Escape => write!(f, "Escape"),
            Self::NoOp => write!(f, "NoOp"),
            Self::Input => write!(f, "Input"),
            Self::Backspace => write!(f, "Backspace"),
        }
    }
}
