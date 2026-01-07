use std::collections::BTreeMap;

use reqwest::Client;
use serde::Deserialize;

use crate::connectors::{
    Connector,
    rqbit::{Rqbit, TorrentError, api::RqbitHttpApi},
};

#[derive(Debug, Default)]
pub struct Connectors(
    BTreeMap<String, Box<dyn Connector<Error = TorrentError> + Sync + Send + 'static>>,
);

impl<'de> Deserialize<'de> for Connectors {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(tag = "api_version", rename_all = "lowercase")]
        enum RqbitApiVersion {
            V8,
            V9,
        }

        #[derive(Deserialize)]
        #[serde(tag = "kind", content = "config", rename_all = "lowercase")]
        enum ConnectorConfig {
            Rqbit {
                url: String,
                api_version: RqbitApiVersion,
            },
            Transmission,
        }

        type ConnectorsMap = BTreeMap<String, ConnectorConfig>;

        let connectors_map = ConnectorsMap::deserialize(deserializer)?;

        let connectors = connectors_map
            .into_iter()
            .map(|(name, connector_raw)| match connector_raw {
                ConnectorConfig::Rqbit { url, api_version } => {
                    let api = match api_version {
                        RqbitApiVersion::V8 => RqbitHttpApi::builder()
                            .base_url(url)
                            .client(Client::default())
                            .build()
                            .unwrap(),
                        RqbitApiVersion::V9 => todo!(),
                    };
                    let rqbit: Box<dyn Connector<Error = TorrentError> + Send + Sync + 'static> =
                        Box::new(Rqbit::builder().name(name.clone()).api(api).build()?);
                    Ok((name, rqbit))
                }
                ConnectorConfig::Transmission => todo!(),
            })
            // TODO: Replace color_eyre with structured error handling (snafu)
            // TODO: Add proper error display in UI
            .collect::<color_eyre::Result<_>>()
            .map_err(serde::de::Error::custom)?;
        Ok(Connectors(connectors))
    }
}
