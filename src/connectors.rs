pub mod rqbit;

use std::sync::Arc;

use async_trait::async_trait;
use snafu::Snafu;

use crate::torrent::{self, InfoHash, Source};

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
}

pub type BoxedError = Box<dyn snafu::Error + Send + Sync + 'static>;

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

#[derive(Debug)]
#[cfg_attr(test, derive(Clone))]
pub enum ConnectorCommands {
    Add(Source),
    Action {
        kind: ActionKind,
        info_hash: InfoHash,
    },
}

#[derive(Debug)]
#[cfg_attr(test, derive(Clone, Eq, PartialEq))]
pub enum ActionKind {
    Pause,
    Start,
    Forget,
    Delete,
}
