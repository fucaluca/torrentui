use std::{collections::HashMap, ops::Deref};

use ratatui::style::{Color, Modifier, Style};
use serde::Deserialize;

use crate::{mode::AddTorrentMode, torrent::State, ui::which_key::WhichKey};

#[derive(Debug, Default, Deserialize, Hash, Eq, PartialEq)]
pub enum StyleMode {
    Active,
    Paused,
    Initializing,
    Error,
    Table,
    Notification,
    WhichKey,
    AddTorrent(AddTorrentMode),
    #[default]
    Default,
}

use std::str::FromStr;

impl FromStr for StyleMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Active" => Ok(StyleMode::Active),
            "Paused" => Ok(StyleMode::Paused),
            "Initializing" => Ok(StyleMode::Initializing),
            "Error" => Ok(StyleMode::Error),
            "Table" => Ok(StyleMode::Table),
            "Notification" => Ok(StyleMode::Notification),
            "WhichKey" => Ok(StyleMode::WhichKey),
            "Default" => Ok(StyleMode::default()),
            _ => Err(format!("Unknown key mode: {}", s)),
        }
    }
}

impl From<&State> for StyleMode {
    fn from(value: &State) -> Self {
        match value {
            State::Active => Self::Active,
            State::Paused => Self::Paused,
            State::Initializing => Self::Initializing,
            State::Error => Self::Error,
        }
    }
}

type StyleMap = HashMap<StyleMode, HashMap<String, Style>>;

#[derive(Debug, Default)]
pub struct Styles(pub StyleMap);

impl Deref for Styles {
    type Target = StyleMap;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Styles {
    pub fn get_style(&self, mode: &StyleMode, name: &str) -> Style {
        *self
            .get(mode)
            .and_then(|s| s.get(name).or(s.get("default")))
            .or(self
                .get(&StyleMode::default())
                .and_then(|s| s.get(name).or(s.get("default"))))
            .unwrap_or(&Style::default())
    }
}

impl<'de> Deserialize<'de> for Styles {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Debug, Deserialize, Default)]
        #[serde(default)]
        struct Helper {
            #[serde(flatten)]
            flat: HashMap<String, HashMap<String, String>>,

            #[serde(rename = "AddTorrent")]
            add_torrent: Option<AddTorrentGroup>,
        }

        #[derive(Debug, Deserialize, Default)]
        #[serde(default, rename_all = "PascalCase")]
        struct AddTorrentGroup {
            input: Option<HashMap<String, String>>,
            connectors: Option<HashMap<String, String>>,
        }

        let helper = Helper::deserialize(deserializer)?;

        let mut parsed_map = HashMap::<StyleMode, HashMap<String, String>>::new();

        for (key_str, bindings) in helper.flat {
            let mode = key_str.parse::<StyleMode>().map_err(|e| {
                serde::de::Error::custom(format!("Unknown mode '{}': {}", key_str, e))
            })?;
            parsed_map.insert(mode, bindings);
        }

        if let Some(add) = helper.add_torrent {
            if let Some(bindings) = add.input {
                parsed_map.insert(StyleMode::AddTorrent(AddTorrentMode::Input), bindings);
            }
            if let Some(bindings) = add.connectors {
                parsed_map.insert(StyleMode::AddTorrent(AddTorrentMode::Connectors), bindings);
            }
        }

        let styles = parsed_map
            .into_iter()
            .map(|(mode, inner_map)| {
                let converted_inner_map = inner_map
                    .into_iter()
                    .map(|(str, style)| (str, parse_style(&style)))
                    .collect();
                (mode, converted_inner_map)
            })
            .collect();
        Ok(Styles(styles))
    }
}

pub fn parse_style(line: &str) -> Style {
    let (foreground, background) =
        line.split_at(line.to_lowercase().find("on ").unwrap_or(line.len()));
    let foreground = process_color_string(foreground);
    let background = process_color_string(&background.replace("on ", ""));

    let mut style = Style::default();
    if let Some(fg) = parse_color(&foreground.0) {
        style = style.fg(fg);
    }
    if let Some(bg) = parse_color(&background.0) {
        style = style.bg(bg);
    }
    style = style.add_modifier(foreground.1 | background.1);
    style
}

fn process_color_string(color_str: &str) -> (String, Modifier) {
    let color = color_str
        .replace("grey", "gray")
        .replace("bright ", "")
        .replace("bold ", "")
        .replace("underline ", "")
        .replace("inverse ", "");

    let mut modifiers = Modifier::empty();
    if color_str.contains("underline") {
        modifiers |= Modifier::UNDERLINED;
    }
    if color_str.contains("bold") {
        modifiers |= Modifier::BOLD;
    }
    if color_str.contains("inverse") {
        modifiers |= Modifier::REVERSED;
    }

    (color, modifiers)
}

fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim_start();
    let s = s.trim_end();
    if s.contains("bright color") {
        let s = s.trim_start_matches("bright ");
        let c = s
            .trim_start_matches("color")
            .parse::<u8>()
            .unwrap_or_default();
        Some(Color::Indexed(c.wrapping_shl(8)))
    } else if s.contains("color") {
        let c = s
            .trim_start_matches("color")
            .parse::<u8>()
            .unwrap_or_default();
        Some(Color::Indexed(c))
    } else if s.contains("rgb") {
        let colors = *s
            .split(':')
            .collect::<Vec<&str>>()
            .last()
            .unwrap_or_else(|| panic!("Cannot parse {s}"));

        let mut colors_iter = colors.split(',').map(|c| {
            c.parse::<u8>()
                .unwrap_or_else(|_| panic!("Cannot parse {s}"))
        });
        let red = colors_iter.next().unwrap_or_default();
        let green = colors_iter.next().unwrap_or_default();
        let blue = colors_iter.next().unwrap_or_default();
        Some(Color::Rgb(red, green, blue))
    } else if s == "dark gray" {
        Some(Color::DarkGray)
    } else if s.contains("gray") {
        Some(Color::Gray)
    } else if s == "black" {
        Some(Color::Black)
    } else if s == "red" {
        Some(Color::Red)
    } else if s == "green" {
        Some(Color::Green)
    } else if s == "yellow" {
        Some(Color::Yellow)
    } else if s == "blue" {
        Some(Color::Blue)
    } else if s == "magenta" {
        Some(Color::Magenta)
    } else if s == "cyan" {
        Some(Color::Cyan)
    } else if s == "white" {
        Some(Color::White)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use ratatui::style::{Color, Modifier, Style};

    use crate::settings::styles::{parse_color, parse_style, process_color_string};

    #[test]
    fn test_parse_style_default() {
        let style = parse_style("");
        assert_eq!(style, Style::default());
    }

    #[test]
    fn test_parse_style_by_name() {
        let style = parse_style("dark gray");
        assert_eq!(style.fg, Some(Color::DarkGray));
    }

    #[test]
    fn test_parse_style_foreground() {
        let style = parse_style("red");
        assert_eq!(style.fg, Some(Color::Red));
    }

    #[test]
    fn test_parse_style_background() {
        let style = parse_style("on blue");
        assert_eq!(style.bg, Some(Color::Blue));
    }

    #[test]
    fn test_parse_style_modifiers() {
        let style = parse_style("underline red on blue");
        assert_eq!(style.fg, Some(Color::Red));
        assert_eq!(style.bg, Some(Color::Blue));
    }

    #[test]
    fn test_process_color_string() {
        let (color, modifiers) = process_color_string("underline bold inverse gray");
        assert_eq!(color, "gray");
        assert!(modifiers.contains(Modifier::UNDERLINED));
        assert!(modifiers.contains(Modifier::BOLD));
        assert!(modifiers.contains(Modifier::REVERSED));
    }

    #[test]
    fn test_parse_color_rgb() {
        let color = parse_color("rgb:0,0,0");
        assert_eq!(color, Some(Color::Rgb(0, 0, 0)));
    }

    #[test]
    fn test_parse_color_unknown() {
        let color = parse_color("unknown");
        assert_eq!(color, None);
    }
}
