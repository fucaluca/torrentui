pub mod rqbit;

use std::sync::Arc;

use async_trait::async_trait;
use snafu::Snafu;

use crate::torrent::{self, InfoHash, Source, TorrentInfo};

#[derive(Debug)]
pub enum ConnectorEvents {
    AddOk,
    PauseOk,
    StartOk,
    ForgetOk,
    DeleteOk,
    UpdateTorrentList(Arc<String>, Vec<TorrentInfo>),
    Error(ConnectorError),
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Connector: std::fmt::Debug + Sync + Send {
    async fn get_torrent_list(&self) -> Result<Vec<torrent::TorrentInfo>, ConnectorError>;
    async fn add_torrent(&self, torrent_source: torrent::Source) -> Result<(), ConnectorError>;
    async fn forget_torrent(&self, info_hash: torrent::InfoHash) -> Result<(), ConnectorError>;
    async fn delete_torrent(&self, info_hash: torrent::InfoHash) -> Result<(), ConnectorError>;
    async fn pause_torrent(&self, info_hash: torrent::InfoHash) -> Result<(), ConnectorError>;
    async fn start_torrent(&self, info_hash: torrent::InfoHash) -> Result<(), ConnectorError>;
    fn name(&self) -> Arc<String>;
    fn selected(&self) -> &bool;
    #[expect(dead_code)]
    fn selected_mut(&mut self) -> &mut bool;
    #[expect(dead_code)]
    fn toggle_selected(&mut self);
    fn url(&self) -> String;
}

pub type BoxedError = Box<dyn snafu::Error + Send + Sync + 'static>;
pub type ConnectorName = Arc<String>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum ConnectorError {
    #[snafu(display("Failed to fetch torrent list"))]
    GetListFailed {
        source: BoxedError,
        connector_name: Arc<String>,
        operation: String,
    },
    #[snafu(display("Failed to add magnet"))]
    AddTorrentFailed {
        source: BoxedError,
        connector_name: Arc<String>,
        operation: String,
    },
    #[snafu(display("Failed to forget torrent {}", info_hash))]
    ForgetTorrent {
        source: BoxedError,
        connector_name: Arc<String>,
        operation: String,
        info_hash: InfoHash,
    },
    #[snafu(display("Failed to delete torrent {}", info_hash))]
    DeleteTorrent {
        source: BoxedError,
        connector_name: Arc<String>,
        operation: String,
        info_hash: InfoHash,
    },
    #[snafu(display("Failed to pause torrent {}", info_hash))]
    PauseTorrent {
        source: BoxedError,
        connector_name: Arc<String>,
        operation: String,
        info_hash: InfoHash,
    },
    #[snafu(display("Failed to resume torrent {}", info_hash))]
    StartTorrent {
        source: BoxedError,
        connector_name: Arc<String>,
        operation: String,
        info_hash: InfoHash,
    },
}

#[derive(Debug, Clone)]
pub enum ConnectorCommands {
    #[cfg_attr(not(test), expect(dead_code))]
    Add(Source),
    Action {
        kind: ActionKind,
        info_hash: InfoHash,
    },
}

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub enum ActionKind {
    Pause,
    Start,
    Forget,
    Delete,
}
