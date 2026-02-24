use std::fmt::Display;

use serde::Deserialize;
use snafu::Snafu;
use tokio::sync::mpsc::error::SendError;

use crate::{
    app::CurrentScreen, connectors::ConnectorCommands, domain::torrent::source::MagnetError,
    settings::keybindings::KeyBindingsError,
};

#[derive(Debug, Default, Clone)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub enum Action {
    Quit,
    AddMagnet,
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
    Edit,
    Backspace,
    Switch,
    Toggle,
    Send,
    Enter,
    Play,
    DefaultScreen,
    Paste,
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
            "AddMagnet" => Ok(AddMagnet),
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
            "Edit" => Ok(Edit),
            "Backspace" => Ok(Backspace),
            "Switch" => Ok(Switch),
            "Toggle" => Ok(Toggle),
            "Send" => Ok(Send),
            "Enter" => Ok(Enter),
            "Play" => Ok(Play),
            "Paste" => Ok(Paste),
            "NoOp" => Ok(NoOp),
            "Next" => Ok(Next),
            variant => Err(serde::de::Error::unknown_variant(
                variant,
                &[
                    "Quit",
                    "AddMagnet",
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
                    "Edit",
                    "Backspace",
                    "Switch",
                    "Toggle",
                    "Send",
                    "Enter",
                    "Play",
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
            Self::AddMagnet => write!(f, "AddMagnet"),
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
            Self::Play => write!(f, "Play"),
            Self::Paste => write!(f, "Paste"),
            Self::NoOp => write!(f, "NoOp"),
            Self::Edit => write!(f, "Edit"),
            Self::Backspace => write!(f, "Backspace"),
            Self::Toggle => write!(f, "Toggle"),
            Self::Send => write!(f, "Send"),
            Self::Enter => write!(f, "Enter"),
            Self::DefaultScreen => write!(f, "HideCurrentScreen"),
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
    #[snafu(display("Failed to launch external player"))]
    PlayError { source: std::io::Error },

    #[snafu(display("Failed to initialize clipboard"))]
    ClipboardInitError { source: arboard::Error },

    #[snafu(display("Failed to get text from clipboard"))]
    GetFromClipboardError { source: arboard::Error },
}
