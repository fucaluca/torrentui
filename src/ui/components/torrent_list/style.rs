use ratatui::style::Style;

use crate::settings::styles::{StyleMode, Styles};

pub struct StyleHelper<'a> {
    styles: &'a Styles,
}

impl<'a> StyleHelper<'a> {
    pub fn new(styles: &'a Styles) -> Self {
        Self { styles }
    }

    pub fn get_style(&self, mode: &StyleMode, name: &str) -> Style {
        *self
            .styles
            .get(mode)
            .and_then(|s| s.get(name).or(s.get("default")))
            .or(self
                .styles
                .get(&StyleMode::default())
                .and_then(|s| s.get(name).or(s.get("default"))))
            .unwrap_or(&Style::default())
    }
}
