use std::{collections::BTreeMap, fmt::Display, time::Duration};

use reqwest::Client;
use serde::Deserialize;
use snafu::{ResultExt, Snafu};

use crate::connectors::{
    Connector,
    rqbit::{
        Rqbit, RqbitBuilderError, TorrentError,
        api::{ApiBuilderError, RqbitHttpApiV8},
    },
};

type ConnectorBox = Box<dyn Connector<Error = TorrentError> + Send + Sync + 'static>;

#[derive(Debug)]
struct ConfiguredConnector {
    connector: ConnectorBox,
    update_interval: Duration,
}

#[derive(Debug, Default)]
pub struct Connectors(BTreeMap<String, ConfiguredConnector>);

impl<'de> Deserialize<'de> for Connectors {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Debug, Snafu)]
        enum CreateConnectorError {
            #[snafu(display(r#"Failed to create API version "{api_version}""#))]
            FailedCreateApi {
                source: ApiBuilderError,
                api_version: ApiVersion,
            },

            #[snafu(display(r#"Failed to create connector {name}"#))]
            FailedCreateConnector {
                source: RqbitBuilderError,
                name: String,
            },
        }

        #[derive(Deserialize, Debug)]
        // #[serde(tag = "api_version")]
        #[serde(rename_all = "lowercase")]
        enum ApiVersion {
            V8,
            V9,
        }

        impl Display for ApiVersion {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    ApiVersion::V8 => write!(f, "V8"),
                    ApiVersion::V9 => write!(f, "V9"),
                }
            }
        }

        #[derive(Deserialize)]
        #[serde(tag = "kind", rename_all = "lowercase")]
        enum ConnectorConfig {
            Rqbit {
                url: String,
                api_version: ApiVersion,
                update_interval: u64,
            },
            Transmission,
        }

        type ConnectorsMap = BTreeMap<String, ConnectorConfig>;

        let connectors_map = ConnectorsMap::deserialize(deserializer)?;

        let connectors = connectors_map
            .into_iter()
            .map(|(name, connector_raw)| match connector_raw {
                ConnectorConfig::Rqbit {
                    url,
                    api_version,
                    update_interval,
                } => {
                    let api = match api_version {
                        ApiVersion::V8 => RqbitHttpApiV8::builder()
                            .base_url(url)
                            .client(Client::default())
                            .build()
                            .context(FailedCreateApiSnafu { api_version })?,
                        ApiVersion::V9 => todo!("API version 9 is not implemented yet"),
                    };
                    let rqbit: ConnectorBox = Box::new(
                        Rqbit::builder()
                            .name(name.clone())
                            .api(api)
                            .build()
                            .context(FailedCreateConnectorSnafu { name: name.clone() })?,
                    );
                    let configured_connector = ConfiguredConnector {
                        connector: rqbit,
                        update_interval: Duration::from_secs(update_interval),
                    };
                    Ok((name, configured_connector))
                }
                ConnectorConfig::Transmission => {
                    todo!("Transmission backend is not implemented yet")
                }
            })
            .collect::<Result<_, CreateConnectorError>>()
            .map_err(|e| serde::de::Error::custom(format!("{:?}", color_eyre::Report::new(e))))?;
        Ok(Connectors(connectors))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::settings::{ConfigSource, Settings};

    use pretty_assertions::assert_eq;

    #[test]
    fn create_rqbit_connector() -> color_eyre::Result<()> {
        let connector_name = "localhost";
        let config_toml = format!(
            r#"
            [connectors.{connector_name}]
            kind = "rqbit"
            url = "http://localhost:3030"
            api_version = "v8"
            update_interval = 5
        "#
        );
        let config_source = ConfigSource::String(config_toml);
        let settings = Settings::new(config_source)?;
        let connectors = settings.connectors.0;

        assert_eq!(connectors.contains_key(connector_name), true);
        let connector = connectors
            .get(connector_name)
            .expect(r#"Connector {connector_name} not found"#);
        assert_eq!(connector.update_interval, Duration::from_secs(5));
        Ok(())
    }
}
