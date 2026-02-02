use std::sync::Arc;

use color_eyre::eyre::{OptionExt, Result};
use crossterm::event::KeyEvent;

use crate::{
    action::Action,
    connectors::ConnectorError,
    mode::Mode,
    settings::keybindings::{KeyBindings, KeyBindingsNode},
    torrent::TorrentInfo,
};

pub struct KeyBindingsTrie {
    current_sequence: Vec<KeyEvent>,
}

#[derive(Debug)]
pub enum ConnectorEvents {
    AddOk,
    PauseOk,
    StartOk,
    ForgetOk,
    DeleteOk,
    UpdateTorrentList(Arc<String>, Vec<TorrentInfo>),
    Error(ConnectorError),
}

impl KeyBindingsTrie {
    pub fn new() -> Result<Self> {
        /* let _ = keybindings_settings.get(mode).ok_or_eyre(format!(
            "Key mode {:?} not found. Available key modes: {:?}",
            mode,
            &keybindings_settings.keys().collect::<Vec<_>>()
        ))?; */

        Ok(Self {
            current_sequence: Vec::new(),
        })
    }
    pub fn action(&mut self, key_event: KeyEvent, root: &KeyBindingsNode) -> Option<Action> {
        self.current_sequence.push(key_event);
        let mut current_node = root;
        for key in &self.current_sequence {
            if let Some(next_node) = current_node.next.get(key) {
                current_node = next_node;
            } else {
                self.current_sequence.clear();
                return None;
            }
        }
        if current_node.next.is_empty() {
            self.current_sequence.clear();
            Some(current_node.action)
        } else {
            Some(Action::NoOp)
        }
    }

    /* pub fn key_mode(&mut self, key_mode: Mode, keybindings_settings: &KeyBindings) -> Result<()> {
        let _ = keybindings_settings.get(&key_mode).ok_or_eyre(format!(
            "Key mode {:?} not found. Available key modes {:?}",
            &key_mode,
            &keybindings_settings.keys().collect::<Vec<_>>()
        ))?;
        self.key_mode = key_mode;
        self.current_sequence = Vec::new();
        Ok(())
    } */

    pub fn get_current_node<'a>(&self, root: &'a KeyBindingsNode) -> Result<&'a KeyBindingsNode> {
        let mut current = root;
        for key in &self.current_sequence {
            if let Some(next) = current.next.get(key) {
                current = next;
            } else {
                return Ok(root);
            }
        }
        Ok(current)
    }
}

/* #[cfg(test)]
mod test {
    use super::{Action, KeyBindingsTrie, Mode, Result};
    use crate::settings::keybindings::{
        KeyBindings, KeyBindingsNode, test_utils::KeyBindingsTestBuilder,
    };
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use pretty_assertions::assert_eq;

    #[test]
    fn get_action() -> Result<()> {
        let key_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        let keybindings_node = KeyBindingsTestBuilder::new()
            .key_event(key_event)
            .action(Action::Quit)
            .build();

        let mut keybindings_settings = KeyBindings::default();
        let key_mode = Mode::default();
        keybindings_settings.insert(key_mode, keybindings_node);

        let mut keybindings_trie = KeyBindingsTrie::builder()
            .key_mode(key_mode)
            .build(&keybindings_settings)?;

        let action = keybindings_trie.action(key_event, &keybindings_settings);
        assert_eq!(action.is_some(), true);
        assert_eq!(action.unwrap(), Action::Quit);

        Ok(())
    }

    #[test]
    fn update_key_mode() -> Result<()> {
        let keybindings_node_1 = KeyBindingsNode::default();
        let keybindings_node_2 = KeyBindingsNode::default();
        let mut keybindings_settings = KeyBindings::default();
        let key_mode1 = Mode::default();
        let key_mode2 = Mode::AddTorrent;
        keybindings_settings.insert(key_mode1, keybindings_node_1);
        keybindings_settings.insert(key_mode2, keybindings_node_2);

        let mut keybindings_trie = KeyBindingsTrie::builder()
            .key_mode(key_mode1)
            .build(&keybindings_settings)?;

        assert_eq!(keybindings_settings.contains_key(&key_mode1), true);

        keybindings_trie.key_mode(key_mode2, &keybindings_settings)?;

        assert_eq!(keybindings_settings.contains_key(&key_mode2), true);

        Ok(())
    }

    #[test]
    fn error_when_updating_to_nonexistent_key_mode() -> Result<()> {
        let mut keybindings_settings = KeyBindings::default();
        let existing_key_mode = Mode::default();
        let non_existent_key_mode = Mode::AddTorrent;

        keybindings_settings.insert(existing_key_mode, KeyBindingsNode::default());

        let mut keybindings_trie = KeyBindingsTrie::builder()
            .key_mode(existing_key_mode)
            .build(&keybindings_settings)?;

        assert_eq!(keybindings_settings.contains_key(&existing_key_mode), true);
        assert_eq!(
            keybindings_trie
                .key_mode(non_existent_key_mode, &keybindings_settings)
                .is_err(),
            true
        );

        Ok(())
    }

    #[test]
    fn reset_to_root_on_wrong_key() -> Result<()> {
        let key_event_a = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        let mut root = KeyBindingsTestBuilder::new()
            .key_event(key_event_a)
            .action(Action::NoOp)
            .build();

        let node1 = root
            .next
            .get_mut(&key_event_a)
            .expect("Builder should create intermediate node");

        let mut node2 = KeyBindingsNode::default();

        let key_event_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE);
        let leaf = KeyBindingsNode::default();
        node2.next.insert(key_event_c.into(), leaf);

        let key_event_b = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE);
        node1.next.insert(key_event_b.into(), node2);

        let key_mode = Mode::default();
        let mut keybindings_settings = KeyBindings::default();
        keybindings_settings.insert(key_mode, root);

        let mut keybindings_trie = KeyBindingsTrie::builder()
            .key_mode(key_mode)
            .build(&keybindings_settings)?;

        assert_eq!(keybindings_trie.current_sequence.is_empty(), true);

        keybindings_trie
            .action(key_event_a, &keybindings_settings)
            .expect("Method action should return Action");

        assert_eq!(keybindings_trie.current_sequence.is_empty(), false);

        let action_is_none = keybindings_trie
            .action(
                KeyEvent::new(KeyCode::Char('z'), KeyModifiers::NONE),
                &keybindings_settings,
            )
            .is_none();

        assert_eq!(action_is_none, true);

        assert_eq!(keybindings_trie.current_sequence.is_empty(), true);

        Ok(())
    }

    #[test]
    fn reset_to_root_on_leaf_node() -> Result<()> {
        let key_event_a = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        let mut root = KeyBindingsTestBuilder::new()
            .key_event(key_event_a)
            .action(Action::NoOp)
            .build();

        let intermediate = root
            .next
            .get_mut(&key_event_a)
            .expect("Builder should create intermediate node");

        let mut leaf = KeyBindingsNode::default();
        leaf.set_action(Action::Quit);

        let key_event_b = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE);
        intermediate.next.insert(key_event_b.into(), leaf);

        let mut keybindings_settings = KeyBindings::default();
        let key_mode = Mode::default();
        keybindings_settings.insert(key_mode, root);

        let mut keybindings_trie = KeyBindingsTrie::builder()
            .key_mode(key_mode)
            .build(&keybindings_settings)?;

        assert_eq!(keybindings_trie.current_sequence.is_empty(), true);

        let action_a = keybindings_trie
            .action(key_event_a, &keybindings_settings)
            .expect("Method action should return Action");

        assert_eq!(action_a, Action::NoOp);
        assert_eq!(keybindings_trie.current_sequence.is_empty(), false);

        let action_b = keybindings_trie
            .action(key_event_b, &keybindings_settings)
            .expect("Method action should return Action");

        assert_eq!(action_b, Action::Quit);
        assert_eq!(keybindings_trie.current_sequence.is_empty(), true);

        Ok(())
    }
} */
