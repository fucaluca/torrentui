use std::fs;

use async_trait::async_trait;
use futures::future::try_join_all;
use reqwest::Client;
use serde::{Deserialize, de::DeserializeOwned};
use snafu::{ResultExt, Snafu};

use super::{
    Api, ApiError, ParseResponseSnafu, ReadTorrentSnafu, RequestSnafu, TorrentInfoRaw,
    endpoints::Endpoints,
};
use crate::torrent::{InfoHash, Source};

pub mod dto;

#[derive(Default)]
pub struct ApiBuilder {
    base_url: String,
    client: Client,
}

#[derive(Debug, Snafu)]
pub enum ApiBuilderError {
    #[snafu(display(r#"Invalid base url: "{}""#, base_url))]
    InvalidUrl {
        source: url::ParseError,
        base_url: String,
    },
}

impl ApiBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    pub fn client(mut self, client: Client) -> Self {
        self.client = client;
        self
    }

    pub fn build(self) -> Result<RqbitHttpApi, ApiBuilderError> {
        let _ = url::Url::parse(&self.base_url).context(InvalidUrlSnafu {
            base_url: &self.base_url,
        })?;
        Ok(RqbitHttpApi {
            base_url: self.base_url,
            client: self.client,
        })
    }
}

pub struct RqbitHttpApi {
    base_url: String,
    client: Client,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct HttpError {
    error_kind: String,
    human_readable: String,
    status: u16,
    status_text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
}

impl RqbitHttpApi {
    pub fn builder() -> ApiBuilder {
        ApiBuilder::new()
    }

    async fn send_get_request<T: DeserializeOwned>(&self, endpoint: String) -> Result<T, ApiError> {
        let url = format!("{}{}", self.base_url, endpoint);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context(RequestSnafu { url: &url })?;

        if response.status().is_success() {
            response
                .json::<T>()
                .await
                .context(ParseResponseSnafu { url: &url })
        } else {
            let error = response
                .json::<HttpError>()
                .await
                .context(ParseResponseSnafu { url: &url })?;
            Err(ApiError::Api {
                status: error.status,
                url,
                message: error.human_readable,
            })
        }
    }

    async fn send_post_request(&self, endpoint: String) -> Result<(), ApiError> {
        let url = format!("{}{}", self.base_url, endpoint);
        let response = self
            .client
            .post(&url)
            .send()
            .await
            .context(RequestSnafu { url: &url })?;
        self.handle_response_status(response, url).await
    }

    async fn handle_response_status(
        &self,
        response: reqwest::Response,
        url: String,
    ) -> Result<(), ApiError> {
        if !response.status().is_success() {
            let error = response
                .json::<HttpError>()
                .await
                .context(ParseResponseSnafu { url: &url })?;
            Err(ApiError::Api {
                status: error.status,
                url,
                message: error.human_readable,
            })
        } else {
            Ok(())
        }
    }

    async fn get_torrent_info(
        &self,
        info_hash: &InfoHash,
    ) -> Result<dto::TorrentInfoResponse, ApiError> {
        self.send_get_request::<dto::TorrentInfoResponse>(Endpoints::get_torrent_info(info_hash))
            .await
    }

    async fn get_torrent_list(&self) -> Result<Vec<dto::TorrentItemResponse>, ApiError> {
        Ok(self
            .send_get_request::<dto::TorrentListResponse>(Endpoints::get_torrents())
            .await?
            .torrents)
    }

    async fn map_torrent_info(
        &self,
        torrent_item: dto::TorrentItemResponse,
    ) -> Result<TorrentInfoRaw, ApiError> {
        let info_hash = torrent_item.info_hash;
        let torrent_info = self.get_torrent_info(&info_hash).await?;
        let dto::TorrentLiveResponse {
            download_speed,
            upload_speed,
            time_remaining,
            snapshot,
        } = torrent_info.live.unwrap_or_default();

        Ok(TorrentInfoRaw {
            name: torrent_item.name,
            info_hash,
            output_folder: torrent_item.output_folder,
            finished: torrent_info.finished,
            state: torrent_info.state.into(),
            downloaded_bytes: torrent_info.progress_bytes,
            uploaded_bytes: torrent_info.uploaded_bytes,
            total_bytes: torrent_info.total_bytes,
            download_speed_mpbs: download_speed.mbps,
            upload_speed_mpbs: upload_speed.mbps,
            time_remaining_secs: time_remaining.map(|tr| tr.duration.secs),
            peer_queued: snapshot.peer_stats.queued,
            peer_live: snapshot.peer_stats.live,
        })
    }
}

#[async_trait]
impl Api for RqbitHttpApi {
    async fn get_torrents(&self) -> Result<Vec<TorrentInfoRaw>, ApiError> {
        let torrents = self.get_torrent_list().await?;
        let futures = torrents
            .into_iter()
            .map(|torrent_item| self.map_torrent_info(torrent_item))
            .collect::<Vec<_>>();
        try_join_all(futures).await
    }

    async fn start_torrent(&self, info_hash: &InfoHash) -> Result<(), ApiError> {
        self.send_post_request(Endpoints::start_torrent(info_hash))
            .await
    }

    async fn pause_torrent(&self, info_hash: &InfoHash) -> Result<(), ApiError> {
        self.send_post_request(Endpoints::pause_torrent(info_hash))
            .await
    }

    async fn delete_torrent(&self, info_hash: &InfoHash) -> Result<(), ApiError> {
        self.send_post_request(Endpoints::delete_torrent(info_hash))
            .await
    }

    async fn forget_torrent(&self, info_hash: &InfoHash) -> Result<(), ApiError> {
        self.send_post_request(Endpoints::forget_torrent(info_hash))
            .await
    }

    async fn add_torrent(&self, torrent_source: Source) -> Result<(), ApiError> {
        let url = format!("{}{}", self.base_url, Endpoints::add_torrent());
        let body = match torrent_source {
            Source::Magnet(magnet) => magnet.as_bytes().to_vec(),
            Source::FilePath(path) => fs::read(&path).context(ReadTorrentSnafu { path })?,
        };

        let response = self
            .client
            .post(&url)
            .body(body)
            .send()
            .await
            .context(RequestSnafu { url: &url })?;

        self.handle_response_status(response, url).await
    }
}

#[cfg(test)]
mod test {
    use std::fs;

    use crate::torrent::source::Magnet;
    use assert_matches::assert_matches;
    use color_eyre::eyre::OptionExt;
    use fake::{Fake, Faker};
    use pretty_assertions::assert_eq;
    use reqwest::{Client, StatusCode};
    use tempdir::TempDir;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{body_bytes, method, path},
    };

    use super::{Api, ApiError, Endpoints, HttpError, InfoHash, RqbitHttpApi, Source, dto};

    async fn setup() -> color_eyre::Result<(RqbitHttpApi, MockServer)> {
        let _result = color_eyre::config::HookBuilder::default()
            .theme(color_eyre::config::Theme::dark())
            .install();

        let mock_server = MockServer::start().await;
        let api = RqbitHttpApi::builder()
            .base_url(mock_server.uri())
            .client(Client::default())
            .build()?;

        Ok((api, mock_server))
    }

    #[tokio::test]
    async fn start_torrent() -> color_eyre::Result<()> {
        let (api, mock_server) = setup().await?;
        let info_hash = Faker.fake::<InfoHash>();
        let endpoint = Endpoints::start_torrent(&info_hash);
        let responder = ResponseTemplate::new(StatusCode::OK);

        Mock::given(method("POST"))
            .and(path(&endpoint))
            .respond_with(responder)
            .expect(1)
            .named(format!("POST {}", &endpoint))
            .mount(&mock_server)
            .await;

        api.start_torrent(&info_hash).await?;

        Ok(())
    }

    #[tokio::test]
    async fn start_torrent_not_found() -> color_eyre::Result<()> {
        let (api, mock_server) = setup().await?;
        let info_hash = Faker.fake::<InfoHash>();
        let endpoint = Endpoints::start_torrent(&info_hash);
        let body = HttpError {
            error_kind: "torrent_not_found".into(),
            human_readable: format!("torrent {} not found", &info_hash),
            status: 404,
            status_text: "404 Not Found".into(),
            id: Some(info_hash.to_string()),
        };
        let responder = ResponseTemplate::new(StatusCode::NOT_FOUND).set_body_json(&body);

        Mock::given(method("POST"))
            .and(path(&endpoint))
            .respond_with(responder)
            .expect(1)
            .named(format!("POST {}", &endpoint))
            .mount(&mock_server)
            .await;

        let response = api.start_torrent(&info_hash).await;

        assert_matches!(
            response,
            Err(ApiError::Api { status, url, message }) => {
                let expected_url = format!("{}{}", api.base_url, &endpoint);
                assert_eq!(status, body.status);
                assert_eq!(url, expected_url);
                assert_eq!(message, body.human_readable);
            },
            "Expected HttpError::Api with specific fields, but got {:?}",
            response
        );

        Ok(())
    }

    #[tokio::test]
    async fn pause_torrent() -> color_eyre::Result<()> {
        let (api, mock_server) = setup().await?;
        let info_hash = Faker.fake::<InfoHash>();
        let responder = ResponseTemplate::new(StatusCode::OK);
        let endpoint = Endpoints::pause_torrent(&info_hash);

        Mock::given(method("POST"))
            .and(path(&endpoint))
            .respond_with(responder)
            .expect(1)
            .named(format!("POST {}", &endpoint))
            .mount(&mock_server)
            .await;

        api.pause_torrent(&info_hash).await?;
        Ok(())
    }

    #[tokio::test]
    async fn pause_torrent_not_found() -> color_eyre::Result<()> {
        let (api, mock_server) = setup().await?;
        let info_hash = Faker.fake::<InfoHash>();
        let endpoint = Endpoints::pause_torrent(&info_hash);
        let body = HttpError {
            error_kind: "torrent_not_found".into(),
            human_readable: format!("torrent {} not found", &info_hash),
            status: 404,
            status_text: "404 Not Found".into(),
            id: Some(info_hash.to_string()),
        };

        let responder = ResponseTemplate::new(StatusCode::NOT_FOUND).set_body_json(&body);

        Mock::given(method("POST"))
            .and(path(&endpoint))
            .respond_with(responder)
            .expect(1)
            .named(format!("POST {}", &endpoint))
            .mount(&mock_server)
            .await;
        let response = api.pause_torrent(&info_hash).await;

        assert_matches!(
            response,
            Err(ApiError::Api { status, url, message }) => {
                let expected_url = format!("{}{}", api.base_url, &endpoint);
                assert_eq!(status, body.status);
                assert_eq!(url, expected_url);
                assert_eq!(message, body.human_readable);
            },
            "Expected HttpError::Api with specific fields, but got {:?}",
            response
        );

        Ok(())
    }

    #[tokio::test]
    async fn delete_torrent() -> color_eyre::Result<()> {
        let (api, mock_server) = setup().await?;
        let info_hash = Faker.fake::<InfoHash>();
        let endpoint = Endpoints::delete_torrent(&info_hash);
        let responder = ResponseTemplate::new(StatusCode::OK);

        Mock::given(method("POST"))
            .and(path(&endpoint))
            .respond_with(responder)
            .expect(1)
            .named(format!("POST {}", &endpoint))
            .mount(&mock_server)
            .await;

        api.delete_torrent(&info_hash).await?;

        Ok(())
    }

    #[tokio::test]
    async fn delete_torrent_not_found() -> color_eyre::Result<()> {
        let (api, mock_server) = setup().await?;
        let info_hash = Faker.fake::<InfoHash>();
        let body = HttpError {
            error_kind: "torrent_not_found".into(),
            human_readable: format!("torrent {} not found", &info_hash),
            status: 404,
            status_text: "404 Not Found".into(),
            id: Some(info_hash.to_string()),
        };
        let responder = ResponseTemplate::new(StatusCode::NOT_FOUND).set_body_json(&body);

        let endpoint = Endpoints::delete_torrent(&info_hash);
        Mock::given(method("POST"))
            .and(path(&endpoint))
            .respond_with(responder)
            .expect(1)
            .named(format!("POST {}", &endpoint))
            .mount(&mock_server)
            .await;

        let response = api.delete_torrent(&info_hash).await;

        assert_matches!(
            response,
            Err(ApiError::Api { status, url, message }) => {
                let expected_url = format!("{}{}", api.base_url, endpoint);
                assert_eq!(status, body.status);
                assert_eq!(url, expected_url);
                assert_eq!(message, body.human_readable);
            },
            "Expected HttpError::Api with specific parameters, but got {:?}",
            response
        );
        Ok(())
    }

    #[tokio::test]
    async fn forget_torrent_not_found() -> color_eyre::Result<()> {
        let (api, mock_server) = setup().await?;
        let info_hash = Faker.fake::<InfoHash>();
        let body = HttpError {
            error_kind: "torrent_not_found".into(),
            human_readable: format!("torrent {} not found", &info_hash),
            status: 404,
            status_text: "404 Not Found".into(),
            id: Some(info_hash.to_string()),
        };
        let responder = ResponseTemplate::new(StatusCode::NOT_FOUND).set_body_json(&body);
        let endpoint = Endpoints::forget_torrent(&info_hash);

        Mock::given(method("POST"))
            .and(path(&endpoint))
            .respond_with(responder)
            .expect(1)
            .named(format!("POST {}", &endpoint))
            .mount(&mock_server)
            .await;

        let result = api.forget_torrent(&info_hash).await;

        assert_matches!(
            result,
            Err(ApiError::Api { status, url, message }) => {
                let expected_url = format!("{}{}", api.base_url, endpoint);
                assert_eq!(status, body.status);
                assert_eq!(url, expected_url);
                assert_eq!(message, body.human_readable);
            },
            "Expected HttpError::Api with specific fields, but got {:?}",
            result
        );

        Ok(())
    }

    #[tokio::test]
    async fn forget_torrent() -> color_eyre::Result<()> {
        let (api, mock_server) = setup().await?;
        let info_hash = Faker.fake::<InfoHash>();
        let responder = ResponseTemplate::new(StatusCode::OK);
        let endpoint = Endpoints::forget_torrent(&info_hash);

        Mock::given(method("POST"))
            .and(path(&endpoint))
            .respond_with(responder)
            .expect(1)
            .named(format!("POST {}", &endpoint))
            .mount(&mock_server)
            .await;

        api.forget_torrent(&info_hash).await?;

        Ok(())
    }

    #[tokio::test]
    async fn add_torrent_unknown_error() -> color_eyre::Result<()> {
        let (api, mock_server) = setup().await?;
        let magnet = "magnet:?xt=urn:btih:3E408407EB36C13F7C1CA999E686B1AD8C49A4CF&tr=http%3A%2F%2Fbt4.t-ru.org%2Fann%3Fmagnet&dn=%D0%94%D0%B6%D0%B0%D0%BD%D0%B3%D0%BE%20%D0%BE%D1%81%D0%B2%D0%BE%D0%B1%D0%BE%D0%B6%D0%B4%D0%B5%D0%BD%D0%BD%D1%8B%D0%B9%20%2F%20Django%20Unchained%20(%D0%9A%D0%B2%D0%B5%D0%BD%D1%82%D0%B8%D0%BD%20%D0%A2%D0%B0%D1%80%D0%B0%D0%BD%D1%82%D0%B8%D0%BD%D0%BE%20%2F%20Quentin%20Tarantino)%20%5B2012%2C%20%D0%A1%D0%A8%D0%90%2C%20%D0%B4%D1%80%D0%B0%D0%BC%D0%B0%2C%20%D0%B2%D0%B5%D1%81%D1%82%D0%B5%D1%80%D0%BD%2C%20%D0%BA%D0%BE%D0%BC%D0%B5%D0%B4%D0%B8%D1%8F%2C%20%D0%BF%D1%80%D0%B8%D0%BA%D0%BB%D1%8E%D1%87%D0%B5%D0%BD%D0%B8%D1%8F%2C%20BDRip%201080p%5D";
        let body = HttpError {
            error_kind: "internal_server_error".into(),
            human_readable: "Internal Server Error".into(),
            status: 500,
            status_text: "500 Internal Server Error".into(),
            id: None,
        };
        let responder =
            ResponseTemplate::new(StatusCode::INTERNAL_SERVER_ERROR).set_body_json(&body);

        let endpoint = Endpoints::add_torrent();
        Mock::given(method("POST"))
            .and(path(&endpoint))
            .and(body_bytes(magnet))
            .respond_with(responder)
            .expect(1)
            .named(format!("POST {}", &endpoint))
            .mount(&mock_server)
            .await;

        let magnet = Magnet::new(magnet)?;
        let torrent_source = Source::Magnet(magnet);
        let response = api.add_torrent(torrent_source).await;

        assert_matches!(
            response,
            Err(ApiError::Api { status, url, message }) => {
                let expected_url = format!("{}{}", api.base_url, &endpoint);
                assert_eq!(status, body.status);
                assert_eq!(url, expected_url);
                assert_eq!(message, body.human_readable);
            },
            "Expected HttpError::Api with specific fields, but got: {:?}",
            response,
        );

        Ok(())
    }

    #[tokio::test]
    async fn add_magnet() -> color_eyre::Result<()> {
        let (api, mock_server) = setup().await?;
        let magnet = "magnet:?xt=urn:btih:3E408407EB36C13F7C1CA999E686B1AD8C49A4CF&tr=http%3A%2F%2Fbt4.t-ru.org%2Fann%3Fmagnet&dn=%D0%94%D0%B6%D0%B0%D0%BD%D0%B3%D0%BE%20%D0%BE%D1%81%D0%B2%D0%BE%D0%B1%D0%BE%D0%B6%D0%B4%D0%B5%D0%BD%D0%BD%D1%8B%D0%B9%20%2F%20Django%20Unchained%20(%D0%9A%D0%B2%D0%B5%D0%BD%D1%82%D0%B8%D0%BD%20%D0%A2%D0%B0%D1%80%D0%B0%D0%BD%D1%82%D0%B8%D0%BD%D0%BE%20%2F%20Quentin%20Tarantino)%20%5B2012%2C%20%D0%A1%D0%A8%D0%90%2C%20%D0%B4%D1%80%D0%B0%D0%BC%D0%B0%2C%20%D0%B2%D0%B5%D1%81%D1%82%D0%B5%D1%80%D0%BD%2C%20%D0%BA%D0%BE%D0%BC%D0%B5%D0%B4%D0%B8%D1%8F%2C%20%D0%BF%D1%80%D0%B8%D0%BA%D0%BB%D1%8E%D1%87%D0%B5%D0%BD%D0%B8%D1%8F%2C%20BDRip%201080p%5D";
        let responder = ResponseTemplate::new(StatusCode::OK);
        let endpoint = Endpoints::add_torrent();

        Mock::given(method("POST"))
            .and(path(&endpoint))
            .and(body_bytes(magnet))
            .respond_with(responder)
            .expect(1)
            .named(format!("POST {}", &endpoint))
            .mount(&mock_server)
            .await;

        let magnet = Magnet::new(magnet)?;
        let torrent_source = Source::Magnet(magnet);

        api.add_torrent(torrent_source).await?;

        Ok(())
    }

    #[tokio::test]
    async fn add_torrent_file() -> color_eyre::Result<()> {
        let (api, mock_server) = setup().await?;
        let tmp_dir = TempDir::new(env!("CARGO_PKG_NAME"))?;
        let file_name = "test_torrent_file.torrent";
        let file_path = tmp_dir.path().join(file_name);
        let torrent_file_content = b"Test torrent file content";
        fs::write(&file_path, torrent_file_content)?;

        let responder = ResponseTemplate::new(StatusCode::OK);

        let endpoint = Endpoints::add_torrent();
        Mock::given(method("POST"))
            .and(path(&endpoint))
            .and(body_bytes(torrent_file_content))
            .respond_with(responder)
            .expect(1)
            .named(format!("POST {}", endpoint))
            .mount(&mock_server)
            .await;

        let torrent_source = Source::FilePath(file_path);
        api.add_torrent(torrent_source).await?;

        tmp_dir.close()?;
        Ok(())
    }

    #[tokio::test]
    async fn get_torrents_unknown_error() -> color_eyre::Result<()> {
        let (api, mock_server) = setup().await?;
        let body = HttpError {
            error_kind: "internal_server_error".into(),
            human_readable: "Internal Server Error".into(),
            status: 500,
            status_text: "500 Internal Server Error".into(),
            id: None,
        };
        let responder =
            ResponseTemplate::new(StatusCode::INTERNAL_SERVER_ERROR).set_body_json(&body);

        let endpoint = Endpoints::get_torrents();
        Mock::given(method("GET"))
            .and(path(&endpoint))
            .respond_with(responder)
            .expect(1)
            .named(format!("GET {}", &endpoint))
            .mount(&mock_server)
            .await;

        let response = api.get_torrents().await;
        assert_matches!(
            response,
            Err(ApiError::Api { status, url, message }) => {
                let expected_url = format!("{}{}", api.base_url, endpoint);
                assert_eq!(status, body.status);
                assert_eq!(url, expected_url);
                assert_eq!(message, body.human_readable);
            },
            "Expected HttpError::Api with specific fields, but got: {:?}",
            response
        );
        Ok(())
    }

    #[tokio::test]
    async fn get_torrents() -> color_eyre::Result<()> {
        let (api, mock_server) = setup().await?;
        let torrents = (0..3)
            .map(|_| Faker.fake::<dto::TorrentItemResponse>())
            .collect::<Vec<_>>();

        let mut test_data: Vec<dto::TorrentInfoResponse> = vec![];

        for (i, torrent) in torrents.iter().enumerate() {
            test_data.push(dto::TorrentInfoResponse {
                state: dto::TorrentStateResponse::Active,
                live: Some(dto::TorrentLiveResponse {
                    download_speed: dto::TorrentSpeedResponse { mbps: 12.0 },
                    upload_speed: dto::TorrentSpeedResponse { mbps: 2.0 },
                    time_remaining: Some(dto::TorrentTimeRemainingResponse {
                        duration: dto::TorrentTimeRemainingDurationResponse { secs: 22 },
                    }),
                    ..Faker.fake()
                }),
                ..Faker.fake()
            });
            let responder = ResponseTemplate::new(StatusCode::OK).set_body_json(&test_data[i]);
            let endpoint = Endpoints::get_torrent_info(&torrent.info_hash);
            Mock::given(method("GET"))
                .and(path(&endpoint))
                .respond_with(responder)
                .expect(1)
                .named(format!("GET {endpoint}"))
                .mount(&mock_server)
                .await;
        }

        let torrents_length = torrents.len();
        let response = dto::TorrentListResponse {
            torrents: torrents.clone(),
        };
        let responder = ResponseTemplate::new(StatusCode::OK).set_body_json(&response);
        let endpoint = Endpoints::get_torrents();

        Mock::given(method("GET"))
            .and(path(&endpoint))
            .respond_with(responder)
            .expect(1)
            .named(format!("GET {endpoint}"))
            .mount(&mock_server)
            .await;

        let result = api.get_torrents().await?;

        assert_eq!(result.len(), torrents_length);

        for (i, torrent_info_raw) in result.iter().enumerate() {
            assert_eq!(torrent_info_raw.info_hash, torrents[i].info_hash);
            assert_eq!(torrent_info_raw.name, torrents[i].name);
            assert_eq!(torrent_info_raw.output_folder, torrents[i].output_folder);

            assert_eq!(torrent_info_raw.finished, test_data[i].finished);
            assert_eq!(
                torrent_info_raw.downloaded_bytes,
                test_data[i].progress_bytes
            );
            assert_eq!(torrent_info_raw.uploaded_bytes, test_data[i].uploaded_bytes);
            assert_eq!(torrent_info_raw.total_bytes, test_data[i].total_bytes);

            let expected_live = test_data[i]
                .live
                .as_ref()
                .ok_or_eyre(r#"Field "live" cannot be None"#)?;
            assert_eq!(
                torrent_info_raw.download_speed_mpbs,
                expected_live.download_speed.mbps
            );
            assert_eq!(
                torrent_info_raw.upload_speed_mpbs,
                expected_live.upload_speed.mbps
            );
            assert_eq!(
                torrent_info_raw.peer_queued,
                expected_live.snapshot.peer_stats.queued
            );
            assert_eq!(
                torrent_info_raw.peer_live,
                expected_live.snapshot.peer_stats.live
            );
            assert_eq!(
                torrent_info_raw.time_remaining_secs,
                expected_live
                    .time_remaining
                    .as_ref()
                    .map(|tr| tr.duration.secs)
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn get_torrent_info_not_found() -> color_eyre::Result<()> {
        let (api, mock_server) = setup().await?;
        let info_hash = Faker.fake::<InfoHash>();
        let body = HttpError {
            error_kind: "torrent_not_found".into(),
            human_readable: format!("torrent {info_hash} not found"),
            status: 404,
            status_text: "404 Not Found".into(),
            id: Some(info_hash.to_string()),
        };
        let responder = ResponseTemplate::new(StatusCode::NOT_FOUND).set_body_json(&body);

        let endpoint = Endpoints::get_torrent_info(&info_hash);
        Mock::given(method("GET"))
            .and(path(&endpoint))
            .respond_with(responder)
            .expect(1)
            .named(format!("GET {}", &endpoint))
            .mount(&mock_server)
            .await;

        let response = api.get_torrent_info(&info_hash).await;

        assert_matches!(
            response,
            Err(ApiError::Api { status, url, message }) => {
                let expected_url = format!("{}{}", api.base_url, endpoint);
                assert_eq!(status, body.status);
                assert_eq!(url, expected_url);
                assert_eq!(message, body.human_readable);

            },
            "Expected HttpError::Api with specific fields, but got: {:?}",
            response
        );

        Ok(())
    }

    #[tokio::test]
    async fn get_downloading_torrent_info() -> color_eyre::Result<()> {
        let (api, mock_server) = setup().await?;
        let expected = dto::TorrentInfoResponse {
            live: Some(dto::TorrentLiveResponse {
                upload_speed: dto::TorrentSpeedResponse { mbps: 0. },
                download_speed: dto::TorrentSpeedResponse { mbps: 3. },
                time_remaining: Some(dto::TorrentTimeRemainingResponse {
                    duration: dto::TorrentTimeRemainingDurationResponse { secs: 93 },
                }),
                ..Faker.fake()
            }),
            ..Faker.fake()
        };
        let responder = ResponseTemplate::new(StatusCode::OK).set_body_json(&expected);

        let info_hash = Faker.fake::<InfoHash>();
        let endpoint = Endpoints::get_torrent_info(&info_hash);

        Mock::given(method("GET"))
            .and(path(endpoint))
            .respond_with(responder)
            .expect(1)
            .named("GET {endpoint}")
            .mount(&mock_server)
            .await;

        let response = api.get_torrent_info(&info_hash).await?;

        assert_eq!(response.finished, expected.finished);
        assert_eq!(response.state, expected.state);
        assert_eq!(response.progress_bytes, expected.progress_bytes);
        assert_eq!(response.uploaded_bytes, expected.uploaded_bytes);
        assert_eq!(response.total_bytes, expected.total_bytes);

        let expected_live = expected.live.ok_or_eyre("Expected: live cannot be None")?;
        let response_live = response.live.ok_or_eyre("Response: live cannot be None")?;
        let expected_time_remaining = expected_live
            .time_remaining
            .ok_or_eyre("Expected time_remaining cannot be None")?;
        let response_time_remaining = response_live
            .time_remaining
            .ok_or_eyre("Expected time_remaining cannot be None")?;

        assert_eq!(response_time_remaining, expected_time_remaining);
        assert_eq!(
            response_live.upload_speed.mbps,
            expected_live.upload_speed.mbps
        );
        assert_eq!(
            response_live.download_speed.mbps,
            expected_live.download_speed.mbps
        );

        Ok(())
    }

    #[tokio::test]
    async fn get_uploading_torrent_info() -> color_eyre::Result<()> {
        let (api, mock_server) = setup().await?;
        let expected = dto::TorrentInfoResponse {
            live: Some(dto::TorrentLiveResponse {
                time_remaining: None,
                upload_speed: dto::TorrentSpeedResponse { mbps: 2.3 },
                download_speed: dto::TorrentSpeedResponse { mbps: 0. },
                ..Faker.fake()
            }),
            ..Faker.fake()
        };

        let responder = ResponseTemplate::new(StatusCode::OK).set_body_json(&expected);
        let info_hash = Faker.fake::<InfoHash>();
        let endpoint = Endpoints::get_torrent_info(&info_hash);

        Mock::given(method("GET"))
            .and(path(&endpoint))
            .respond_with(responder)
            .expect(1)
            .named(format!("GET {endpoint}"))
            .mount(&mock_server)
            .await;

        let response = api.get_torrent_info(&info_hash).await?;

        assert_eq!(response.finished, expected.finished);
        assert_eq!(response.state, expected.state);
        assert_eq!(response.progress_bytes, expected.progress_bytes);
        assert_eq!(response.uploaded_bytes, expected.uploaded_bytes);
        assert_eq!(response.total_bytes, expected.total_bytes);

        let expected_live = expected.live.ok_or_eyre("Live cannot be None")?;
        let result_live = response.live.ok_or_eyre("Live cannot be None")?;

        assert_eq!(expected_live.time_remaining.is_none(), true);
        assert_eq!(
            expected_live.upload_speed.mbps,
            result_live.upload_speed.mbps
        );
        assert_eq!(
            expected_live.download_speed.mbps,
            result_live.download_speed.mbps
        );
        assert_eq!(expected_live.snapshot, result_live.snapshot);

        Ok(())
    }

    #[tokio::test]
    async fn get_inactive_torrent_info() -> color_eyre::Result<()> {
        let (api, mock_server) = setup().await?;
        let expected = dto::TorrentInfoResponse {
            live: None,
            ..Faker.fake()
        };
        let responder = ResponseTemplate::new(StatusCode::OK).set_body_json(&expected);

        let info_hash = Faker.fake::<InfoHash>();
        let endpoint = Endpoints::get_torrent_info(&info_hash);
        Mock::given(method("GET"))
            .and(path(&endpoint))
            .respond_with(responder)
            .expect(1)
            .named(format!("GET {}", &endpoint))
            .mount(&mock_server)
            .await;

        let response: dto::TorrentInfoResponse = api.get_torrent_info(&info_hash).await?;

        assert_eq!(response.finished, expected.finished);
        assert_eq!(response.state, expected.state);
        assert_eq!(response.progress_bytes, expected.progress_bytes);
        assert_eq!(response.uploaded_bytes, expected.uploaded_bytes);
        assert_eq!(response.total_bytes, expected.total_bytes);
        assert_eq!(response.live.is_none(), true);

        Ok(())
    }
}
