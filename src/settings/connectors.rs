use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
    sync::Arc,
    time::Duration,
};

use indexmap::IndexMap;
use reqwest::Client;
use serde::Deserialize;
use snafu::{ResultExt, Snafu};

use crate::connectors::{
    Connector, ConnectorName,
    rqbit::{
        Rqbit, RqbitBuilderError,
        api::{ApiBuilderError, RqbitHttpApiV8},
    },
};

type ConnectorBox = Box<dyn Connector + Send + Sync + 'static>;

#[derive(Debug)]
pub struct ConfiguredConnector {
    pub connector: ConnectorBox,
    pub update_interval_secs: Duration,
}

#[derive(Debug, Default)]
pub struct Connectors(pub IndexMap<ConnectorName, Arc<ConfiguredConnector>>);

impl Deref for Connectors {
    type Target = IndexMap<ConnectorName, Arc<ConfiguredConnector>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Connectors {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

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
                name: ConnectorName,
            },
        }

        #[derive(Deserialize, Debug)]
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
                #[cfg_attr(test, serde(default))]
                update_interval_secs: u64,
                #[cfg_attr(test, serde(default))]
                selected_by_default: bool,
            },
            Transmission,
        }

        type ConnectorsMap = IndexMap<String, ConnectorConfig>;

        let connectors_map = ConnectorsMap::deserialize(deserializer)?;

        let connectors = connectors_map
            .into_iter()
            .map(|(name, connector_raw)| match connector_raw {
                ConnectorConfig::Rqbit {
                    url,
                    api_version,
                    update_interval_secs,
                    selected_by_default,
                } => {
                    let api = match api_version {
                        ApiVersion::V8 => RqbitHttpApiV8::builder()
                            .base_url(url)
                            .client(Client::default())
                            .build()
                            .context(FailedCreateApiSnafu { api_version })?,
                        ApiVersion::V9 => todo!("API version 9 is not implemented yet"),
                    };

                    let name = ConnectorName::new(name);
                    let rqbit: ConnectorBox = Box::new(
                        Rqbit::builder()
                            .name(ConnectorName::clone(&name))
                            .api(api)
                            .selected(selected_by_default)
                            .build()
                            .context(FailedCreateConnectorSnafu { name: name.clone() })?,
                    );
                    let configured_connector = ConfiguredConnector {
                        connector: rqbit,
                        update_interval_secs: Duration::from_secs(update_interval_secs),
                    };
                    Ok((name, Arc::new(configured_connector)))
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
    use std::{sync::Arc, time::Duration};

    use crate::settings::Settings;

    use pretty_assertions::assert_eq;

    #[test]
    fn create_rqbit_connector() -> color_eyre::Result<()> {
        let connector_name = Arc::new("localhost".into());
        let config_str = format!(
            r#"
            [connectors.{connector_name}]
            kind = "rqbit"
            url = "http://localhost:3030"
            api_version = "v8"
            update_interval_secs = 5
        "#
        );
        let settings = Settings::test_settings(config_str)?;

        assert_eq!(settings.connectors.contains_key(&connector_name), true);
        let connector = settings
            .connectors
            .get(&connector_name)
            .expect(r#"Connector {connector_name} not found"#);
        assert_eq!(connector.update_interval_secs, Duration::from_secs(5));
        Ok(())
    }
}
