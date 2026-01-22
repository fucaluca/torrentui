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
    #[default]
    NoOp,
}
