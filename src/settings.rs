use std::path::PathBuf;

use directories::ProjectDirs;

use color_eyre::eyre::Result;
use config::{Config, File};
use serde::Deserialize;

use crate::settings::{connectors::Connectors, keybindings::KeyBindings, styles::Styles};

pub mod connectors;
mod defaults;
pub mod keybindings;
pub mod styles;

#[derive(Debug, Deserialize, Default)]
pub struct Settings {
    #[serde(default)]
    pub keybindings: KeyBindings,
    #[serde(default)]
    pub connectors: Connectors,
    #[serde(default)]
    pub styles: Styles,
    #[serde(default = "defaults::notification_timeout_millis")]
    pub notification_timeout_millis: u64,
}

#[derive(Debug)]
pub enum ConfigSource {
    File(PathBuf),
    #[cfg(test)]
    String(String),
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
