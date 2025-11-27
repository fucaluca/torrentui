use color_eyre::Result;

use crate::app::App;

mod app;
mod errors;
mod tui;

fn main() -> Result<()> {
    crate::errors::init()?;
    let mut app = App::new();
    Ok(())
}
