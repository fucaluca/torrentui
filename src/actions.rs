use serde::Deserialize;

#[derive(Debug, Default, Deserialize, Clone)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub enum Action {
    Quit,
    AddTorrent,
    #[default]
    NoOp,
}
