use std::ops::{Deref, DerefMut};

use color_eyre::eyre::{Result, bail};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use indexmap::IndexMap;
use serde::Deserialize;

use crate::{action::Action, key_mode::KeyMode};

#[derive(Debug, Default)]
pub struct KeyBindingsNode {
    pub display: String,
    pub action: Action,
    pub description: Option<String>,
    pub next: IndexMap<KeyEvent, KeyBindingsNode>,
}

#[cfg(test)]
impl KeyBindingsNode {
    pub fn set_action(&mut self, action: Action) {
        self.action = action;
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum KeyBindingValue {
    Simple(Action),
    Detailed {
        #[serde(default)]
        action: Action,
        #[serde(default)]
        description: Option<String>,
    },
}

#[derive(Debug, Default)]
pub struct KeyBindings(IndexMap<KeyMode, KeyBindingsNode>);

impl Deref for KeyBindings {
    type Target = IndexMap<KeyMode, KeyBindingsNode>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for KeyBindings {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'de> Deserialize<'de> for KeyBindings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let parsed_map =
            IndexMap::<KeyMode, IndexMap<String, KeyBindingValue>>::deserialize(deserializer)?;
        let bindings = parsed_map
            .into_iter()
            .map(|(mode, inner_map)| {
                let mut root_bindings = KeyBindingsNode::default();

                for (raw, value) in inner_map {
                    let key_events = parse_key_sequence(&raw).map_err(serde::de::Error::custom)?;

                    add_binding_to_tree(&mut root_bindings, key_events, value);
                }

                Ok((mode, root_bindings))
            })
            .collect::<Result<IndexMap<_, _>, D::Error>>()?;

        Ok(KeyBindings(bindings))
    }
}

pub fn add_binding_to_tree(
    root: &mut KeyBindingsNode,
    key_events: Vec<(KeyEvent, String)>,
    value: KeyBindingValue,
) {
    let mut current = root;
    let mut iter = key_events.into_iter().peekable();

    while let Some(key_event) = iter.next() {
        let is_last = iter.peek().is_none();
        if is_last {
            let mut node = KeyBindingsNode::from(value.clone());
            node.display = key_event.1;
            current.next.insert(key_event.0, node);
        } else {
            let node = current.next.entry(key_event.0).or_default();
            node.display = key_event.1;
            current = node;
        }
    }
}

impl From<KeyBindingValue> for KeyBindingsNode {
    fn from(value: KeyBindingValue) -> Self {
        match value {
            KeyBindingValue::Simple(action) => Self {
                display: String::new(),
                action,
                description: None,
                next: IndexMap::new(),
            },
            KeyBindingValue::Detailed {
                action,
                description,
            } => Self {
                display: String::new(),
                action,
                description,
                next: IndexMap::new(),
            },
        }
    }
}

pub fn parse_key_sequence(raw: &str) -> Result<Vec<(KeyEvent, String)>> {
    let mut events = Vec::new();
    let mut remaining = raw;

    while !remaining.is_empty() {
        if let Some(rest) = remaining.strip_prefix('<') {
            if let Some(end) = rest.find('>') {
                let (inside, next) = rest.split_at(end);
                remaining = &next[1..];
                events.push((parse_key_event(inside)?, inside.to_string()));
            } else {
                bail!("Unclosed '<' in `{}`", raw);
            }
        } else {
            bail!("Expected '<' at start of key segment in `{}`", raw);
        }
    }
    Ok(events)
}

fn parse_key_event(raw: &str) -> Result<KeyEvent> {
    let (remaining, modifiers) = extract_modifiers(raw);
    parse_key_code_with_modifiers(remaining, modifiers)
}

fn extract_modifiers(raw: &str) -> (&str, KeyModifiers) {
    let mut modifiers = KeyModifiers::empty();
    let mut current = raw;

    while let Some(rest) = current
        .strip_prefix("ctrl-")
        .or_else(|| current.strip_prefix("alt-"))
        .or_else(|| current.strip_prefix("shift-"))
    {
        match &current[..current.len() - rest.len()] {
            "ctrl-" => modifiers.insert(KeyModifiers::CONTROL),
            "alt-" => modifiers.insert(KeyModifiers::ALT),
            "shift-" => modifiers.insert(KeyModifiers::SHIFT),
            _ => unreachable!(),
        }
        current = rest;
    }
    (current, modifiers)
}

fn parse_key_code_with_modifiers(raw: &str, mut modifiers: KeyModifiers) -> Result<KeyEvent> {
    let c = match raw {
        "esc" => KeyCode::Esc,
        "enter" => KeyCode::Enter,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        "backtab" => {
            modifiers.insert(KeyModifiers::SHIFT);
            KeyCode::BackTab
        }
        "backspace" => KeyCode::Backspace,
        "delete" => KeyCode::Delete,
        "insert" => KeyCode::Insert,
        "f1" => KeyCode::F(1),
        "f2" => KeyCode::F(2),
        "f3" => KeyCode::F(3),
        "f4" => KeyCode::F(4),
        "f5" => KeyCode::F(5),
        "f6" => KeyCode::F(6),
        "f7" => KeyCode::F(7),
        "f8" => KeyCode::F(8),
        "f9" => KeyCode::F(9),
        "f10" => KeyCode::F(10),
        "f11" => KeyCode::F(11),
        "f12" => KeyCode::F(12),
        "space" => KeyCode::Char(' '),
        "hyphen" | "minus" => KeyCode::Char('-'),
        "tab" => KeyCode::Tab,
        c if c.len() == 1 => {
            let mut c = c.chars().next().unwrap();
            if modifiers.contains(KeyModifiers::SHIFT) {
                c = c.to_ascii_uppercase();
            }
            KeyCode::Char(c)
        }
        _ => bail!("Unable to parse {raw}"),
    };
    Ok(KeyEvent::new(c, modifiers))
}

#[cfg(test)]
pub mod test_utils;

#[cfg(test)]
mod tests {
    use super::{Action, KeyBindingsNode, KeyCode, KeyEvent, KeyMode, KeyModifiers, Result};
    use crate::settings::{ConfigSource, Settings};
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_single_key_without_description() -> Result<()> {
        let config_toml = r#"
            [keybindings.TorrentList]
            "<q>" = "Quit"
        "#;
        let config_source = ConfigSource::String(config_toml.into());
        let settings = Settings::new(config_source)?;
        let key_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        let keybindings: &KeyBindingsNode = settings
            .keybindings
            .get(&KeyMode::TorrentList)
            .and_then(|k| k.next.get(&key_event))
            .unwrap_or_else(|| panic!("KeyEvent {key_event:#?} not found"));
        assert_eq!(keybindings.action, Action::Quit);
        assert_eq!(keybindings.description, None);
        assert_eq!(keybindings.next.is_empty(), true);
        Ok(())
    }

    #[test]
    fn parse_single_key_with_description() -> Result<()> {
        let config_toml = r#"
            [keybindings.TorrentList]
            "<q>" = { action = "Quit", description = "Quit" }
        "#;
        let config_source = ConfigSource::String(config_toml.into());
        let settings = Settings::new(config_source)?;
        let key_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        let keybindings: &KeyBindingsNode = settings
            .keybindings
            .get(&KeyMode::TorrentList)
            .and_then(|k| k.next.get(&key_event))
            .unwrap_or_else(|| panic!("KeyEvent {key_event:#?} not found"));

        assert_eq!(keybindings.action, Action::Quit);
        assert_eq!(keybindings.description, Some("Quit".to_string()));
        assert_eq!(keybindings.next.is_empty(), true);

        Ok(())
    }

    #[test]
    fn parse_keys_with_ctrl_modifier() -> Result<()> {
        let config_toml = r#"
            [keybindings.TorrentList]
            "<ctrl-q>" = "Quit"
        "#;
        let config_source = ConfigSource::String(config_toml.into());
        let settings = Settings::new(config_source)?;
        let key_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
        let keybindings: &KeyBindingsNode = settings
            .keybindings
            .get(&KeyMode::TorrentList)
            .and_then(|k| k.next.get(&key_event))
            .unwrap_or_else(|| panic!("KeyEvent {key_event:#?} not found"));

        assert_eq!(keybindings.action, Action::Quit);
        assert_eq!(keybindings.description, None);
        assert_eq!(keybindings.next.is_empty(), true);

        Ok(())
    }

    #[test]
    fn parse_keys_with_alt_modifier() -> Result<()> {
        let config_toml = r#"
            [keybindings.TorrentList]
            "<alt-q>" = "Quit"
        "#;
        let config_source = ConfigSource::String(config_toml.into());
        let settings = Settings::new(config_source)?;
        let key_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::ALT);
        let keybindings: &KeyBindingsNode = settings
            .keybindings
            .get(&KeyMode::TorrentList)
            .and_then(|k| k.next.get(&key_event))
            .unwrap_or_else(|| panic!("KeyEvent {key_event:#?} not found"));

        assert_eq!(keybindings.action, Action::Quit);
        assert_eq!(keybindings.description, None);
        assert_eq!(keybindings.next.is_empty(), true);

        Ok(())
    }

    #[test]
    fn parse_keys_with_shift_modifier() -> Result<()> {
        let config_toml = r#"
            [keybindings.TorrentList]
            "<shift-q>" = "Quit"
        "#;
        let config_source = ConfigSource::String(config_toml.into());
        let settings = Settings::new(config_source)?;
        let key_event = KeyEvent::new(KeyCode::Char('Q'), KeyModifiers::SHIFT);
        let keybindings: &KeyBindingsNode = settings
            .keybindings
            .get(&KeyMode::TorrentList)
            .and_then(|k| k.next.get(&key_event))
            .unwrap_or_else(|| panic!("KeyEvent {key_event:#?} not found"));

        assert_eq!(keybindings.action, Action::Quit);
        assert_eq!(keybindings.description, None);
        assert_eq!(keybindings.next.is_empty(), true);

        Ok(())
    }

    #[test]
    fn make_keybindings_tree() -> Result<()> {
        let config_toml = r#"
          [keybindings.TorrentList]
          "<ctrl-a><alt-b>" = "AddTorrent"
        "#;
        let config_source = ConfigSource::String(config_toml.into());
        let settings = Settings::new(config_source)?;
        let key_event1 = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
        let key_event2 = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::ALT);

        let keybindings: &KeyBindingsNode = settings
            .keybindings
            .get(&KeyMode::TorrentList)
            .and_then(|k| k.next.get(&key_event1))
            .unwrap_or_else(|| panic!("KeyEvent {key_event1:#?} not found"));

        assert_eq!(keybindings.action, Action::NoOp);
        assert_eq!(keybindings.description, None);
        assert_eq!(keybindings.next.is_empty(), false);

        let next = keybindings
            .next
            .get(&key_event2)
            .unwrap_or_else(|| panic!("KeyEvent {key_event2:#?} not found"));

        assert_eq!(next.action, Action::AddTorrent);
        assert_eq!(next.description, None);
        assert_eq!(next.next.is_empty(), true);

        Ok(())
    }

    #[test]
    fn keybindings_with_common_prefix_share_node() -> Result<()> {
        let config_toml = r#"
            [keybindings.TorrentList]
            "<ctrl-a>" = { description = "Add" }
            "<ctrl-a><t>" = { action = "AddTorrent", description = "Torrent" }
        "#;
        let config_source = ConfigSource::String(config_toml.into());
        let settings = Settings::new(config_source)?;
        let key_event1 = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
        let key_event2 = KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE);

        let keybindings = settings
            .keybindings
            .get(&KeyMode::TorrentList)
            .and_then(|k| k.next.get(&key_event1))
            .unwrap_or_else(|| panic!("KeyEvent {key_event1:#?} not found"));

        assert_eq!(keybindings.action, Action::NoOp);
        assert_eq!(keybindings.description, Some("Add".to_string()));
        assert_eq!(keybindings.next.is_empty(), false);

        let next = keybindings
            .next
            .get(&key_event2)
            .unwrap_or_else(|| panic!("KeyEvent {key_event2:#?} not found"));

        assert_eq!(next.action, Action::AddTorrent);
        assert_eq!(next.description, Some("Torrent".to_string()));
        assert_eq!(next.next.is_empty(), true);

        Ok(())
    }
}
