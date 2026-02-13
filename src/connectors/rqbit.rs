use std::{path::PathBuf, sync::Arc};

use async_trait::async_trait;
use snafu::{ResultExt, Snafu, ensure};

use crate::{
    connectors::{
        AddTorrentFailedSnafu, Connector, ConnectorError, ConnectorName, DeleteTorrentSnafu,
        ForgetTorrentSnafu, GetListFailedSnafu, PauseTorrentSnafu, StartTorrentSnafu,
    },
    torrent::{InfoHash, Source, TorrentInfo},
};

pub mod api;
mod endpoints;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum ApiError {
    #[snafu(display(r#"Failed to send request to "{}""#, url))]
    Request { source: reqwest::Error, url: String },
    #[snafu(display(r#"Failed to parse response from "{}""#, url))]
    ParseResponse { source: reqwest::Error, url: String },
    #[snafu(display("Failed to read torrent file {}", path.display()))]
    ReadTorrent {
        source: std::io::Error,
        path: PathBuf,
    },
    #[snafu(display(r#"API error from "{}" ({}): "{}""#, url, status, message))]
    Api {
        status: u16,
        url: String,
        message: String,
    },
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Api: std::fmt::Debug + Sync + Send + 'static {
    async fn start_torrent(&self, info_hash: &InfoHash) -> Result<(), ApiError>;
    async fn pause_torrent(&self, info_hash: &InfoHash) -> Result<(), ApiError>;
    async fn delete_torrent(&self, info_hash: &InfoHash) -> Result<(), ApiError>;
    async fn forget_torrent(&self, info_hash: &InfoHash) -> Result<(), ApiError>;
    async fn add_torrent(&self, source: Source) -> Result<(), ApiError>;
    async fn get_torrents(&self) -> Result<Vec<TorrentInfo>, ApiError>;
}

#[derive(Debug)]
pub struct Rqbit<T: Api> {
    name: Arc<String>,
    api: T,
    selected: bool,
}

#[async_trait]
impl<T: Api + Send + Sync + 'static> Connector for Rqbit<T> {
    fn name(&self) -> Arc<String> {
        Arc::clone(&self.name)
    }
    fn selected(&self) -> &bool {
        &self.selected
    }
    fn selected_mut(&mut self) -> &mut bool {
        &mut self.selected
    }
    fn toggle_selected(&mut self) {
        self.selected = !self.selected;
    }

    async fn get_torrent_list(&self) -> Result<Vec<TorrentInfo>, ConnectorError> {
        self.api
            .get_torrents()
            .await
            .boxed()
            .context(GetListFailedSnafu {
                connector_name: Arc::clone(&self.name),
                operation: "Get torrent list",
            })
    }

    async fn add_torrent(&self, torrent_source: Source) -> Result<(), ConnectorError> {
        self.api
            .add_torrent(torrent_source)
            .await
            .boxed()
            .context(AddTorrentFailedSnafu {
                connector_name: Arc::clone(&self.name),
                operation: "Add torrent",
            })?;
        Ok(())
    }

    async fn forget_torrent(&self, info_hash: InfoHash) -> Result<(), ConnectorError> {
        self.api
            .forget_torrent(&info_hash)
            .await
            .boxed()
            .context(ForgetTorrentSnafu {
                connector_name: Arc::clone(&self.name),
                operation: "Forget torrent",
                info_hash,
            })
    }

    async fn delete_torrent(&self, info_hash: InfoHash) -> Result<(), ConnectorError> {
        self.api
            .delete_torrent(&info_hash)
            .await
            .boxed()
            .context(DeleteTorrentSnafu {
                connector_name: Arc::clone(&self.name),
                operation: "Delete torrent",
                info_hash,
            })?;
        Ok(())
    }

    async fn pause_torrent(&self, info_hash: InfoHash) -> Result<(), ConnectorError> {
        self.api
            .pause_torrent(&info_hash)
            .await
            .boxed()
            .context(PauseTorrentSnafu {
                connector_name: Arc::clone(&self.name),
                operation: "Pause torrent",
                info_hash,
            })?;
        Ok(())
    }

    async fn start_torrent(&self, info_hash: InfoHash) -> Result<(), ConnectorError> {
        self.api
            .start_torrent(&info_hash)
            .await
            .boxed()
            .context(StartTorrentSnafu {
                connector_name: Arc::clone(&self.name),
                operation: "Start torrent",
                info_hash,
            })?;
        Ok(())
    }
}

impl<T: Api> Rqbit<T> {
    pub fn builder() -> RqbitBuilder<T> {
        RqbitBuilder::new()
    }
}

pub struct RqbitBuilder<T: Api> {
    name: Option<ConnectorName>,
    api: Option<T>,
    selected: bool,
}

impl<T: Api> RqbitBuilder<T> {
    pub fn new() -> Self {
        Self {
            name: None,
            api: None,
            selected: false,
        }
    }
    pub fn name(mut self, name: ConnectorName) -> Self {
        self.name = Some(name);
        self
    }
    pub fn api(mut self, api: T) -> Self {
        self.api = Some(api);
        self
    }
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
    pub fn build(self) -> Result<Rqbit<T>, RqbitBuilderError> {
        ensure!(self.name.is_some(), MissingNameSnafu);
        ensure!(!self.name.as_ref().unwrap().is_empty(), InvalidNameSnafu);
        ensure!(self.api.is_some(), MissingApiSnafu);

        Ok(Rqbit {
            name: self.name.unwrap(),
            api: self.api.unwrap(),
            selected: self.selected,
        })
    }
}

#[derive(Debug, Snafu)]
pub enum RqbitBuilderError {
    #[snafu(display("Api cannot be None"))]
    MissingApi,
    #[snafu(display("Connector name cannot be None"))]
    MissingName,
    #[snafu(display("Connector name cannot be empty"))]
    InvalidName,
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use super::{
        ApiError, ConnectorError, MockApi, Rqbit, RqbitBuilderError, endpoints::Endpoints,
    };
    use crate::{
        connectors::{Connector, ConnectorName},
        torrent::{InfoHash, Source, TorrentInfo, source::Magnet},
    };
    use assert_matches::assert_matches;
    use color_eyre::eyre::OptionExt;
    use fake::{Fake, Faker};
    use mockall::predicate;
    use pretty_assertions::assert_eq;

    fn setup() {
        let _result = color_eyre::config::HookBuilder::default()
            .theme(color_eyre::config::Theme::dark())
            .install();
    }

    fn create_test_rqbit(mock_api: MockApi) -> color_eyre::Result<Rqbit<MockApi>> {
        Ok(Rqbit::builder()
            .name(ConnectorName::new("test_rqbit_connector".into()))
            .api(mock_api)
            .build()?)
    }

    #[test]
    fn builder_missing_name() {
        let rqbit = Rqbit::<MockApi>::builder().build();
        assert_matches!(rqbit, Err(RqbitBuilderError::MissingName));
    }

    #[test]
    fn builder_missing_api() {
        let rqbit = Rqbit::<MockApi>::builder()
            .name(ConnectorName::new("test_rqbit_connector".into()))
            .build();
        assert_matches!(rqbit, Err(RqbitBuilderError::MissingApi));
    }

    #[tokio::test]
    async fn get_torrent_list() -> color_eyre::Result<()> {
        setup();

        let torrent_list = (0..10)
            .map(|_| Faker.fake::<TorrentInfo>())
            .collect::<Vec<_>>();
        let torrent_list_length = torrent_list.len();

        let mut mock_api = MockApi::new();
        mock_api
            .expect_get_torrents()
            .times(1)
            .with()
            .returning(move || Ok(torrent_list.clone()));

        let rqbit = create_test_rqbit(mock_api)?;

        let response = rqbit.get_torrent_list().await?;
        assert_eq!(response.len(), torrent_list_length);

        Ok(())
    }

    #[tokio::test]
    async fn add_torrent() -> color_eyre::Result<()> {
        setup();

        let magnet = Faker.fake::<Magnet>();
        let torrent_source = Source::Magnet(magnet);

        let mut mock_api = MockApi::new();

        mock_api
            .expect_add_torrent()
            .times(1)
            .with(predicate::eq(torrent_source.clone()))
            .returning(|_| Ok(()));

        let rqbit = create_test_rqbit(mock_api)?;

        rqbit.add_torrent(torrent_source).await?;

        Ok(())
    }

    #[tokio::test]
    async fn add_torrent_error() -> color_eyre::Result<()> {
        setup();
        let magnet = Faker.fake::<Magnet>();
        let torrent_source = Source::Magnet(magnet);
        let mut mock_api = MockApi::new();

        mock_api
            .expect_add_torrent()
            .times(1)
            .with(predicate::eq(torrent_source.clone()))
            .returning(|_| {
                Err(ApiError::Api {
                    status: 500,
                    url: Endpoints::add_torrent(),
                    message: "internal server error".into(),
                })
            });

        let expected_connector_name = "test_rqbit_connector";

        let rqbit = create_test_rqbit(mock_api)?;

        let result = rqbit.add_torrent(torrent_source).await;

        assert_matches!(
            result,
            Err(ConnectorError::AddTorrentFailed { source, connector_name, operation }) => {
                let api_error = source.downcast_ref::<ApiError>()
                    .ok_or_eyre("Expected ApiError inside Box")?;
                assert_matches!(api_error, ApiError::Api { .. });
                assert_eq!(connector_name, Arc::new(expected_connector_name.into()));
                assert_eq!(operation, "Add torrent");
            }
        );

        Ok(())
    }

    #[tokio::test]
    async fn delete_torrent() -> color_eyre::Result<()> {
        setup();

        let info_hash = Faker.fake::<InfoHash>();
        let mut mock_api = MockApi::new();
        mock_api
            .expect_delete_torrent()
            .times(1)
            .with(predicate::eq(info_hash.clone()))
            .returning(|_| Ok(()));

        let rqbit = create_test_rqbit(mock_api)?;

        rqbit.delete_torrent(info_hash).await?;

        Ok(())
    }

    #[tokio::test]
    async fn delete_torrent_not_found() -> color_eyre::Result<()> {
        setup();

        let expected_info_hash = Faker.fake::<InfoHash>();
        let mut mock_api = MockApi::new();

        mock_api
            .expect_delete_torrent()
            .times(1)
            .with(predicate::eq(expected_info_hash.clone()))
            .returning(|info_hash| {
                Err(ApiError::Api {
                    status: 404,
                    url: Endpoints::delete_torrent(info_hash),
                    message: format!("torrent {} not found", info_hash),
                })
            });

        let expected_connector_name = "test_rqbit_connector";

        let rqbit = create_test_rqbit(mock_api)?;

        let response = rqbit.delete_torrent(expected_info_hash.clone()).await;

        assert_matches!(
            response,
            Err(ConnectorError::DeleteTorrent { source, connector_name, operation, info_hash }) => {
                let api_err = source.downcast_ref::<ApiError>().ok_or_eyre("Expected ApiError inside Box")?;
                assert_matches!(api_err, ApiError::Api { .. });
                assert_eq!(connector_name, Arc::new(expected_connector_name.into()));
                assert_eq!(operation, "Delete torrent");
                assert_eq!(info_hash, expected_info_hash);
            }
        );

        Ok(())
    }

    #[tokio::test]
    async fn forget_torrent() -> color_eyre::Result<()> {
        setup();

        let info_hash = Faker.fake::<InfoHash>();
        let mut mock_api = MockApi::new();

        mock_api
            .expect_forget_torrent()
            .times(1)
            .with(predicate::eq(info_hash.clone()))
            .returning(|_| Ok(()));

        let rqbit = create_test_rqbit(mock_api)?;

        rqbit.forget_torrent(info_hash).await?;

        Ok(())
    }

    #[tokio::test]
    async fn forget_torrent_not_found() -> color_eyre::Result<()> {
        setup();

        let expected_info_hash = Faker.fake::<InfoHash>();
        let mut mock_api = MockApi::new();

        mock_api
            .expect_forget_torrent()
            .times(1)
            .with(predicate::eq(expected_info_hash.clone()))
            .returning(|info_hash| {
                Err(ApiError::Api {
                    status: 404,
                    url: Endpoints::forget_torrent(info_hash),
                    message: format!("torrent {} not found", info_hash),
                })
            });

        let expected_connector_name = "test_rqbit_connector";

        let rqbit = create_test_rqbit(mock_api)?;

        let result = rqbit.forget_torrent(expected_info_hash.clone()).await;

        assert_matches!(
            result,
            Err(ConnectorError::ForgetTorrent { source, connector_name, operation, info_hash }) => {
                let api_error = source.downcast_ref::<ApiError>().ok_or_eyre("Expected ApiError inside Box")?;
                assert_matches!(api_error, ApiError::Api { .. });
                assert_eq!(connector_name, Arc::new(expected_connector_name.into()));
                assert_eq!(operation, "Forget torrent");
                assert_eq!(info_hash, expected_info_hash);
            }
        );

        Ok(())
    }

    #[tokio::test]
    async fn pause_torrent() -> color_eyre::Result<()> {
        setup();

        let info_hash = Faker.fake::<InfoHash>();
        let mut mock_api = MockApi::new();

        mock_api
            .expect_pause_torrent()
            .times(1)
            .with(predicate::eq(info_hash.clone()))
            .returning(|_| Ok(()));

        let rqbit = create_test_rqbit(mock_api)?;

        rqbit.pause_torrent(info_hash).await?;

        Ok(())
    }

    #[tokio::test]
    async fn pause_torrent_not_found() -> color_eyre::Result<()> {
        setup();

        let expected_info_hash = Faker.fake::<InfoHash>();
        let mut mock_api = MockApi::new();

        mock_api
            .expect_pause_torrent()
            .times(1)
            .with(predicate::eq(expected_info_hash.clone()))
            .returning(|info_hash| {
                Err(ApiError::Api {
                    status: 404,
                    url: Endpoints::pause_torrent(info_hash),
                    message: format!("torrent {} not found", info_hash),
                })
            });

        let expected_connector_name = "test_rqbit_connector";

        let rqbit = create_test_rqbit(mock_api)?;

        let result = rqbit.pause_torrent(expected_info_hash.clone()).await;

        assert_matches!(
            result,
            Err(ConnectorError::PauseTorrent { source, connector_name, operation, info_hash }) => {
                let api_error = source.downcast_ref::<ApiError>()
                    .ok_or_eyre("Expected ApiError inside Box")?;
                assert_matches!(api_error, ApiError::Api { .. });
                assert_eq!(connector_name, Arc::new(expected_connector_name.into()));
                assert_eq!(operation, "Pause torrent");
                assert_eq!(info_hash, expected_info_hash);
            }
        );
        Ok(())
    }

    #[tokio::test]
    async fn start_torrent() -> color_eyre::Result<()> {
        setup();

        let info_hash = Faker.fake::<InfoHash>();
        let mut mock_api = MockApi::new();

        mock_api
            .expect_start_torrent()
            .times(1)
            .with(predicate::eq(info_hash.clone()))
            .returning(|_| Ok(()));

        let rqbit = create_test_rqbit(mock_api)?;

        rqbit.start_torrent(info_hash).await?;

        Ok(())
    }
    #[tokio::test]
    async fn start_torrent_not_found() -> color_eyre::Result<()> {
        setup();

        let expected_info_hash = Faker.fake::<InfoHash>();
        let mut mock_api = MockApi::new();

        mock_api
            .expect_start_torrent()
            .times(1)
            .with(predicate::eq(expected_info_hash.clone()))
            .returning(|info_hash| {
                Err(ApiError::Api {
                    status: 404,
                    url: Endpoints::start_torrent(info_hash),
                    message: format!("torrent {} not found", info_hash),
                })
            });

        let expected_connector_name = "test_rqbit_connector";

        let rqbit = create_test_rqbit(mock_api)?;

        let result = rqbit.start_torrent(expected_info_hash.clone()).await;

        assert_matches!(
            result,
            Err(ConnectorError::StartTorrent { source, connector_name, operation, info_hash }) => {
                let api_error = source.downcast_ref::<ApiError>().ok_or_eyre("Expected ApiError inside Box")?;
                assert_matches!(api_error, ApiError::Api { .. });
                assert_eq!(connector_name, Arc::new(expected_connector_name.into()));
                assert_eq!(operation, "Start torrent");
                assert_eq!(info_hash, expected_info_hash);
            }
        );

        Ok(())
    }
}
