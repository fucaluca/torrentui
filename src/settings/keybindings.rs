use color_eyre::eyre::bail;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use indexmap::IndexMap;
use serde::Deserialize;
use snafu::Snafu;
use tracing::debug;

use crate::domain::{
    action::Action,
    modes::{AddMagnetMode, KeyMode},
};

#[derive(Debug, Default)]
pub struct KeyBindingsNode {
    pub display: String,
    pub action: Action,
    pub description: Option<String>,
    pub next: IndexMap<KeyEvent, KeyBindingsNode>,
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
pub struct KeyBindings {
    map: IndexMap<KeyMode, KeyBindingsNode>,
    current_sequence: Vec<KeyEvent>,
}

#[derive(Debug, Snafu)]
pub enum KeyBindingsError {
    #[snafu(display("Key mode {:?} not found. Available key modes: {:?}", mode, keys))]
    GetModeFailed { mode: KeyMode, keys: Vec<KeyMode> },
}

impl KeyBindings {
    pub fn get_current_node(&self, mode: &KeyMode) -> Option<&KeyBindingsNode> {
        let root = self.map.get(mode)?;
        let mut current = root;
        for key in &self.current_sequence {
            if let Some(next) = current.next.get(key) {
                current = next;
            } else {
                return Some(root);
            }
        }
        Some(current)
    }
    pub fn action(
        &mut self,
        mode: KeyMode,
        key_event: KeyEvent,
    ) -> Result<Option<Action>, KeyBindingsError> {
        let root = self.map.get(&mode).ok_or_else(|| {
            let keys = self.map.keys().cloned().collect::<Vec<_>>();
            KeyBindingsError::GetModeFailed { mode, keys }
        })?;
        self.current_sequence.push(key_event);
        let mut current_node = root;
        for key in &self.current_sequence {
            if let Some(next_node) = current_node.next.get(key) {
                current_node = next_node;
            } else {
                self.current_sequence.clear();
                return Ok(None);
            }
        }
        if current_node.next.is_empty() {
            self.current_sequence.clear();
            Ok(Some(current_node.action.clone()))
        } else {
            Ok(Some(Action::Next))
        }
    }
}

impl<'de> Deserialize<'de> for KeyBindings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Debug, Deserialize, Default)]
        struct Helper {
            #[serde(flatten)]
            flat: IndexMap<String, IndexMap<String, KeyBindingValue>>,

            #[serde(rename = "add-magnet")]
            add_magnet: Option<AddMagnetGroup>,
        }

        #[derive(Debug, Deserialize, Default)]
        #[serde(rename_all = "kebab-case")]
        struct AddMagnetGroup {
            input: Option<IndexMap<String, KeyBindingValue>>,
            connectors: Option<IndexMap<String, KeyBindingValue>>,
        }

        let helper = Helper::deserialize(deserializer)?;

        let mut parsed_map = IndexMap::<KeyMode, IndexMap<String, KeyBindingValue>>::new();

        for (key_str, bindings) in helper.flat {
            let mode = key_str.parse::<KeyMode>().map_err(|e| {
                serde::de::Error::custom(format!("Unknown mode '{}': {}", key_str, e))
            })?;
            parsed_map.insert(mode, bindings);
        }

        if let Some(add) = helper.add_magnet {
            if let Some(bindings) = add.input {
                parsed_map.insert(KeyMode::AddMagnet(AddMagnetMode::Input), bindings);
            }
            if let Some(bindings) = add.connectors {
                parsed_map.insert(KeyMode::AddMagnet(AddMagnetMode::Connectors), bindings);
            }
        }

        let bindings = parsed_map
            .into_iter()
            .map(|(mode, inner_map)| {
                let mut root_bindings = KeyBindingsNode::default();

                for (raw, kb_value) in inner_map {
                    let key_events = parse_key_sequence(&raw).map_err(serde::de::Error::custom)?;
                    debug!("Add {raw} = {kb_value:?} to keybindings trie");
                    add_binding_to_tree(&mut root_bindings, key_events, kb_value);
                }

                Ok((mode, root_bindings))
            })
            .collect::<Result<IndexMap<_, _>, D::Error>>()?;

        Ok(KeyBindings {
            map: bindings,
            current_sequence: Vec::new(),
        })
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

pub fn parse_key_sequence(raw: &str) -> color_eyre::Result<Vec<(KeyEvent, String)>> {
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

fn parse_key_event(raw: &str) -> color_eyre::Result<KeyEvent> {
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

fn parse_key_code_with_modifiers(
    raw: &str,
    mut modifiers: KeyModifiers,
) -> color_eyre::Result<KeyEvent> {
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
mod tests {
    use super::{Action, KeyBindingsNode, KeyCode, KeyEvent, KeyMode, KeyModifiers};
    use crate::settings::Settings;
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_single_key_without_description() -> color_eyre::Result<()> {
        let config_str = r#"
            [keybindings.torrent-list]
            "<q>" = "Quit"
        "#;
        let settings = Settings::test_settings(config_str)?;
        let key_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        let keybindings: &KeyBindingsNode = settings
            .keybindings
            .map
            .get(&KeyMode::TorrentList)
            .and_then(|k| k.next.get(&key_event))
            .unwrap_or_else(|| panic!("KeyEvent {key_event:#?} not found"));
        assert_eq!(keybindings.action, Action::Quit);
        assert_eq!(keybindings.description, None);
        assert_eq!(keybindings.next.is_empty(), true);
        Ok(())
    }

    #[test]
    fn parse_single_key_with_description() -> color_eyre::Result<()> {
        let config_str = r#"
            [keybindings.torrent-list]
            "<q>" = { action = "Quit", description = "Quit" }
        "#;
        let settings = Settings::test_settings(config_str)?;
        let key_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        let keybindings: &KeyBindingsNode = settings
            .keybindings
            .map
            .get(&KeyMode::TorrentList)
            .and_then(|k| k.next.get(&key_event))
            .unwrap_or_else(|| panic!("KeyEvent {key_event:#?} not found"));

        assert_eq!(keybindings.action, Action::Quit);
        assert_eq!(keybindings.description, Some("Quit".to_string()));
        assert_eq!(keybindings.next.is_empty(), true);

        Ok(())
    }

    #[test]
    fn parse_keys_with_ctrl_modifier() -> color_eyre::Result<()> {
        let config_str = r#"
            [keybindings.torrent-list]
            "<ctrl-q>" = "Quit"
        "#;
        let settings = Settings::test_settings(config_str)?;
        let key_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
        let keybindings: &KeyBindingsNode = settings
            .keybindings
            .map
            .get(&KeyMode::TorrentList)
            .and_then(|k| k.next.get(&key_event))
            .unwrap_or_else(|| panic!("KeyEvent {key_event:#?} not found"));

        assert_eq!(keybindings.action, Action::Quit);
        assert_eq!(keybindings.description, None);
        assert_eq!(keybindings.next.is_empty(), true);

        Ok(())
    }

    #[test]
    fn parse_keys_with_alt_modifier() -> color_eyre::Result<()> {
        let config_str = r#"
            [keybindings.torrent-list]
            "<alt-q>" = "Quit"
        "#;
        let settings = Settings::test_settings(config_str)?;
        let key_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::ALT);
        let keybindings: &KeyBindingsNode = settings
            .keybindings
            .map
            .get(&KeyMode::TorrentList)
            .and_then(|k| k.next.get(&key_event))
            .unwrap_or_else(|| panic!("KeyEvent {key_event:#?} not found"));

        assert_eq!(keybindings.action, Action::Quit);
        assert_eq!(keybindings.description, None);
        assert_eq!(keybindings.next.is_empty(), true);

        Ok(())
    }

    #[test]
    fn parse_keys_with_shift_modifier() -> color_eyre::Result<()> {
        let config_str = r#"
            [keybindings.torrent-list]
            "<shift-q>" = "Quit"
        "#;
        let settings = Settings::test_settings(config_str)?;
        let key_event = KeyEvent::new(KeyCode::Char('Q'), KeyModifiers::SHIFT);
        let keybindings: &KeyBindingsNode = settings
            .keybindings
            .map
            .get(&KeyMode::TorrentList)
            .and_then(|k| k.next.get(&key_event))
            .unwrap_or_else(|| panic!("KeyEvent {key_event:#?} not found"));

        assert_eq!(keybindings.action, Action::Quit);
        assert_eq!(keybindings.description, None);
        assert_eq!(keybindings.next.is_empty(), true);

        Ok(())
    }

    #[test]
    fn make_keybindings_tree() -> color_eyre::Result<()> {
        let config_str = r#"
          [keybindings.torrent-list]
          "<ctrl-a><alt-b>" = "AddMagnet"
        "#;
        let settings = Settings::test_settings(config_str)?;
        let key_event1 = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
        let key_event2 = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::ALT);

        let keybindings: &KeyBindingsNode = settings
            .keybindings
            .map
            .get(&KeyMode::TorrentList)
            .and_then(|k| k.next.get(&key_event1))
            .unwrap_or_else(|| panic!("KeyEvent {key_event1:#?} not found"));

        assert_eq!(keybindings.action, Action::Next);
        assert_eq!(keybindings.description, None);
        assert_eq!(keybindings.next.is_empty(), false);

        let next = keybindings
            .next
            .get(&key_event2)
            .unwrap_or_else(|| panic!("KeyEvent {key_event2:#?} not found"));

        assert_eq!(next.action, Action::AddMagnet);
        assert_eq!(next.description, None);
        assert_eq!(next.next.is_empty(), true);

        Ok(())
    }

    #[test]
    fn keybindings_with_common_prefix_share_node() -> color_eyre::Result<()> {
        let config_str = r#"
            [keybindings.torrent-list]
            "<ctrl-a>" = { description = "Add" }
            "<ctrl-a><t>" = { action = "AddMagnet", description = "Torrent" }
        "#;
        let settings = Settings::test_settings(config_str)?;
        let key_event1 = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
        let key_event2 = KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE);

        let keybindings = settings
            .keybindings
            .map
            .get(&KeyMode::TorrentList)
            .and_then(|k| k.next.get(&key_event1))
            .unwrap_or_else(|| panic!("KeyEvent {key_event1:#?} not found"));

        assert_eq!(keybindings.action, Action::Next);
        assert_eq!(keybindings.description, Some("Add".to_string()));
        assert_eq!(keybindings.next.is_empty(), false);

        let next = keybindings
            .next
            .get(&key_event2)
            .unwrap_or_else(|| panic!("KeyEvent {key_event2:#?} not found"));

        assert_eq!(next.action, Action::AddMagnet);
        assert_eq!(next.description, Some("Torrent".to_string()));
        assert_eq!(next.next.is_empty(), true);

        Ok(())
    }
}
