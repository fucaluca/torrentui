use serde::Deserialize;

#[derive(Debug, Default, Deserialize, Eq, PartialEq, Hash)]
pub enum Mode {
    #[default]
    TorrentList,
}
