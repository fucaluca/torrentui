use color_eyre::Result;

use crate::app::App;

mod actions;
mod app;
mod errors;
mod mode;
mod settings;
mod tui;

#[tokio::main]
async fn main() -> Result<()> {
    crate::errors::init()?;
    let mut app = App::new()?;
    app.run().await?;
    Ok(())
}
