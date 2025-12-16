use std::path::PathBuf;

use directories::ProjectDirs;

use color_eyre::eyre::Result;
use config::{Config, File};
use serde::Deserialize;

use crate::settings::keybindings::KeyBindings;

mod defaults;
pub mod keybindings;

#[derive(Debug, Deserialize, Default)]
pub struct Settings {
    #[serde(default = "defaults::update_interval")]
    #[allow(dead_code)] // TODO: remove
    update_torrent_list_interval: u8,
    #[serde(default)]
    pub keybindings: KeyBindings,
}

#[derive(Debug)]
pub enum ConfigSource {
    File(PathBuf),
    #[cfg(test)]
    String(&'static str),
}

impl ConfigSource {
    fn get(&self) -> Box<dyn config::Source> {
        match self {
            Self::File(path) => Box::new(File::from(path.clone())),
            #[cfg(test)]
            Self::String(s) => Box::new(File::from_str(s, config::FileFormat::Toml)),
        }
    }
}

impl config::Source for ConfigSource {
    fn clone_into_box(&self) -> Box<dyn config::Source + Send + Sync> {
        self.get().clone_into_box()
    }

    fn collect(&self) -> Result<config::Map<String, config::Value>, config::ConfigError> {
        self.get().collect()
    }
}

impl Settings {
    pub fn new(config_source: ConfigSource) -> Result<Self> {
        let settings = Config::builder().add_source(config_source).build()?;
        Ok(settings.try_deserialize()?)
    }
}

pub fn get_config_dir() -> PathBuf {
    if let Some(project_dir) = get_project_dir() {
        project_dir.config_local_dir().to_path_buf()
    } else {
        PathBuf::from(".").join(".config")
    }
}

fn get_project_dir() -> Option<ProjectDirs> {
    ProjectDirs::from("com", "fucaluca", env!("CARGO_PKG_NAME"))
}

#[cfg(test)]
mod tests {
    use super::{ConfigSource, Result, Settings, defaults};
    use pretty_assertions::assert_eq;

    #[test]
    fn get_update_torrent_list_interval() -> Result<()> {
        let config_toml = r#"
            update_torrent_list_interval = 20
        "#;
        let config_source = ConfigSource::String(config_toml);
        let settings = Settings::new(config_source)?;

        assert_eq!(settings.update_torrent_list_interval, 20);

        Ok(())
    }

    #[test]
    fn default_update_torrent_list_interval() -> Result<()> {
        let default_interval = defaults::update_interval();
        let config_toml = "";
        let config_source = ConfigSource::String(config_toml);
        let settings = Settings::new(config_source)?;

        assert_eq!(settings.update_torrent_list_interval, default_interval);

        Ok(())
    }
}
