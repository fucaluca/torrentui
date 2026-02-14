use std::fmt::Display;

use serde::Deserialize;
use snafu::Snafu;
use tokio::sync::mpsc::error::SendError;

use crate::{
    app::CurrentScreen, connectors::ConnectorCommands, settings::keybindings::KeyBindingsError,
    torrent::source::MagnetError,
};

#[derive(Debug, Default, Clone)]
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
    Help(CurrentScreen),
    Escape,
    Input,
    Backspace,
    Switch,
    Toggle,
    Send,
    NoOp,
    #[default]
    Next,
}

impl<'de> Deserialize<'de> for Action {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let parsed_action = String::deserialize(deserializer)?;
        use Action::*;
        match parsed_action.as_str() {
            "Quit" => Ok(Quit),
            "AddTorrent" => Ok(AddTorrent),
            "Help" => Ok(Help(CurrentScreen::default())),
            "Up" => Ok(Up),
            "Down" => Ok(Down),
            "Left" => Ok(Left),
            "Right" => Ok(Right),
            "GotoTop" => Ok(GotoTop),
            "GotoBottom" => Ok(GotoBottom),
            "Select" => Ok(Select),
            "SelectAll" => Ok(SelectAll),
            "Pause" => Ok(Pause),
            "Start" => Ok(Start),
            "PauseToggle" => Ok(PauseToggle),
            "Forget" => Ok(Forget),
            "Delete" => Ok(Delete),
            "Escape" => Ok(Escape),
            "Input" => Ok(Input),
            "Backspace" => Ok(Backspace),
            "Switch" => Ok(Switch),
            "Toggle" => Ok(Toggle),
            "Send" => Ok(Send),
            "NoOp" => Ok(NoOp),
            "Next" => Ok(Next),
            variant => Err(serde::de::Error::unknown_variant(
                variant,
                &[
                    "Quit",
                    "AddTorrent",
                    "Help",
                    "Up",
                    "Down",
                    "Left",
                    "Right",
                    "GotoTop",
                    "GotoBottom",
                    "Select",
                    "SelectAll",
                    "Pause",
                    "Start",
                    "PauseToggle",
                    "Forget",
                    "Delete",
                    "Escape",
                    "Input",
                    "Backspace",
                    "Switch",
                    "Toggle",
                    "Send",
                    "NoOp",
                ],
            )),
        }
    }
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
            Self::Help(_) => write!(f, "Help"),
            Self::Escape => write!(f, "Escape"),
            Self::Next => write!(f, "Next"),
            Self::Switch => write!(f, "Switch"),
            Self::NoOp => write!(f, "NoOp"),
            Self::Input => write!(f, "Input"),
            Self::Backspace => write!(f, "Backspace"),
            Self::Toggle => write!(f, "Toggle"),
            Self::Send => write!(f, "Send"),
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
    #[snafu(display(r#"Failed to send command"#))]
    SendError {
        source: SendError<ConnectorCommands>,
    },
    #[snafu(display("Failed to create magnet link"))]
    CreateMagnetError { source: MagnetError },
}
