use std::fmt::Display;

use serde::Deserialize;
use snafu::Snafu;
use tokio::sync::mpsc::error::SendError;

use crate::{connectors::ConnectorCommands, settings::keybindings::KeyBindingsError};

#[derive(Debug, Default, Deserialize, Clone, Copy)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub enum Action {
    Quit,
    AddTorrent,
    Up,
    Down,
    Left,
    Right,
    GotoTop,
    GotoBottom,
    Select,
    SelectAll,
    Pause,
    Start,
    PauseToggle,
    Forget,
    Delete,
    Help,
    Escape,
    Input,
    Backspace,
    NoOp,
    #[default]
    Next,
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Quit => write!(f, "Quit"),
            Self::AddTorrent => write!(f, "AddTorrent"),
            Self::Up => write!(f, "Up"),
            Self::Down => write!(f, "Down"),
            Self::Left => write!(f, "Left"),
            Self::Right => write!(f, "Right"),
            Self::GotoTop => write!(f, "GotoTop"),
            Self::GotoBottom => write!(f, "GotoBottom"),
            Self::Pause => write!(f, "Pause"),
            Self::Select => write!(f, "Select"),
            Self::SelectAll => write!(f, "SelectAll"),
            Self::Start => write!(f, "Start"),
            Self::PauseToggle => write!(f, "PauseToggle"),
            Self::Forget => write!(f, "Forget"),
            Self::Delete => write!(f, "Delete"),
            Self::Help => write!(f, "Help"),
            Self::Escape => write!(f, "Escape"),
            Self::Next => write!(f, "Next"),
            Self::NoOp => write!(f, "NoOp"),
            Self::Input => write!(f, "Input"),
            Self::Backspace => write!(f, "Backspace"),
        }
    }
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum ActionError {
    #[snafu(display("Action failed"))]
    CommandSendFailed {
        source: SendError<ConnectorCommands>,
    },
    #[snafu(display("Connector not found"))]
    ConnectorNotFound,
    #[snafu(display(r#"Failed to get Action"#))]
    GetActionFailed { source: KeyBindingsError },
}
