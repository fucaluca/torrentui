use std::sync::LazyLock;

use color_eyre::eyre::Result;
use tracing::info;
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::settings;

static LOG_ENV: LazyLock<String> =
    LazyLock::new(|| format!("{}_LOG_LEVEL", env!("CARGO_CRATE_NAME").to_uppercase()));
static LOG_FILE: LazyLock<String> = LazyLock::new(|| format!("{}.log", env!("CARGO_PKG_NAME")));

pub fn init() -> Result<()> {
    let directory = settings::get_data_dir();
    std::fs::create_dir_all(&directory)?;
    let log_path = directory.join(LOG_FILE.clone());

    let env_filter = EnvFilter::builder()
        .with_default_directive(tracing::Level::INFO.into())
        .try_from_env()
        .or_else(|_| {
            EnvFilter::builder()
                .with_default_directive(tracing::Level::INFO.into())
                .with_env_var(LOG_ENV.clone())
                .from_env()
        })?;

    let layer: Box<dyn Layer<tracing_subscriber::Registry> + Send + Sync> =
        match tracing_journald::layer() {
            Ok(journald) => journald.with_filter(env_filter).boxed(),
            Err(e) => {
                eprintln!("Journald logging disabled: {}, falling back to file", e);
                let log_file = std::fs::File::create(log_path)?;
                fmt::layer()
                    .with_file(true)
                    .with_line_number(true)
                    .with_writer(log_file)
                    .with_target(false)
                    .with_ansi(false)
                    .with_filter(env_filter)
                    .boxed()
            }
        };

    tracing_subscriber::registry()
        .with(layer)
        .with(ErrorLayer::default())
        .try_init()?;
    info!(
        "{} v{} started",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    Ok(())
}
