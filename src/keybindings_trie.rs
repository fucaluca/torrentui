use std::sync::Arc;

use color_eyre::eyre::{OptionExt, Result};
use crossterm::event::KeyEvent;

use crate::{
    action::Action,
    connectors::ConnectorError,
    key_mode::KeyMode,
    settings::keybindings::{KeyBindings, KeyBindingsNode},
    torrent::TorrentInfo,
};

pub struct KeyBindingsTrie<'a> {
    pub keybindings: &'a KeyBindingsNode,
    keybindings_root: &'a KeyBindingsNode,
    pub keybindings_settings: &'a KeyBindings,
}

pub struct KeyBindingsTrieBuilder<'a> {
    keybindings_settings: &'a KeyBindings,
    key_mode: KeyMode,
}

impl<'a> KeyBindingsTrieBuilder<'a> {
    pub fn new(keybindings_settings: &'a KeyBindings) -> Self {
        Self {
            keybindings_settings,
            key_mode: KeyMode::default(),
        }
    }

    pub fn key_mode(mut self, key_mode: KeyMode) -> Self {
        self.key_mode = key_mode;
        self
    }

    pub fn build(self) -> Result<KeyBindingsTrie<'a>> {
        let keybindings = self
            .keybindings_settings
            .get(&self.key_mode)
            .ok_or_eyre(format!(
                "Key mode {:?} not found. Available key modes: {:?}",
                &self.key_mode,
                &self.keybindings_settings.keys().collect::<Vec<_>>()
            ))?;

        Ok(KeyBindingsTrie {
            keybindings,
            keybindings_root: keybindings,
            keybindings_settings: self.keybindings_settings,
            /* torrent_list: BTreeMap::new(),
            connector_commands: HashMap::new(), */
        })
    }
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

impl<'a> KeyBindingsTrie<'a> {
    pub fn builder(keybindings_settings: &'a KeyBindings) -> KeyBindingsTrieBuilder<'a> {
        KeyBindingsTrieBuilder::new(keybindings_settings)
    }

    pub fn action(&mut self, key_event: KeyEvent) -> Option<Action> {
        self.keybindings
            .next
            .get(&key_event)
            .map(|next| {
                self.keybindings = if next.next.is_empty() {
                    self.keybindings_root
                } else {
                    next
                };
                next.action
            })
            .or_else(|| {
                self.keybindings = self.keybindings_root;
                None
            })
    }

    pub fn key_mode(&mut self, key_mode: KeyMode) -> Result<()> {
        let keybindings = self
            .keybindings_settings
            .get(&key_mode)
            .ok_or_eyre(format!(
                "Key mode {:?} not found. Available key modes {:?}",
                &key_mode,
                &self.keybindings_settings.keys().collect::<Vec<_>>()
            ))?;
        self.keybindings = keybindings;
        self.keybindings_root = keybindings;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::{Action, KeyBindingsTrie, KeyMode, Result};
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
        let key_mode = KeyMode::default();
        keybindings_settings.insert(key_mode, keybindings_node);

        let mut app_state = KeyBindingsTrie::builder(&keybindings_settings)
            .key_mode(key_mode)
            .build()?;

        let action = app_state.action(key_event);
        assert_eq!(action.is_some(), true);
        assert_eq!(action.unwrap(), Action::Quit);

        Ok(())
    }

    #[test]
    fn update_key_mode() -> Result<()> {
        let keybindings_node_1 = KeyBindingsNode::default();
        let keybindings_node_2 = KeyBindingsNode::default();
        let mut keybindings_settings = KeyBindings::default();
        let key_mode1 = KeyMode::default();
        let key_mode2 = KeyMode::AddTorrent;
        keybindings_settings.insert(key_mode1, keybindings_node_1);
        keybindings_settings.insert(key_mode2, keybindings_node_2);

        let mut app_state = KeyBindingsTrie::builder(&keybindings_settings)
            .key_mode(key_mode1)
            .build()?;

        assert_eq!(keybindings_settings.contains_key(&key_mode1), true);

        app_state.key_mode(key_mode2)?;

        assert_eq!(keybindings_settings.contains_key(&key_mode2), true);

        Ok(())
    }

    #[test]
    fn error_when_updating_to_nonexistent_key_mode() -> Result<()> {
        let mut keybindings_settings = KeyBindings::default();
        let existing_key_mode = KeyMode::default();
        let non_existent_key_mode = KeyMode::AddTorrent;

        keybindings_settings.insert(existing_key_mode, KeyBindingsNode::default());

        let mut app_state = KeyBindingsTrie::builder(&keybindings_settings)
            .key_mode(existing_key_mode)
            .build()?;

        assert_eq!(keybindings_settings.contains_key(&existing_key_mode), true);
        assert_eq!(app_state.key_mode(non_existent_key_mode).is_err(), true);

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

        let key_mode = KeyMode::default();
        let mut keybindings_settings = KeyBindings::default();
        keybindings_settings.insert(key_mode, root);

        let mut app_state = KeyBindingsTrie::builder(&keybindings_settings)
            .key_mode(key_mode)
            .build()?;

        assert_eq!(
            std::ptr::eq(app_state.keybindings, app_state.keybindings_root),
            true
        );

        app_state
            .action(key_event_a)
            .expect("Method action should return Action");

        assert_eq!(
            std::ptr::eq(app_state.keybindings, app_state.keybindings_root),
            false
        );

        let action_is_none = app_state
            .action(KeyEvent::new(KeyCode::Char('z'), KeyModifiers::NONE))
            .is_none();

        assert_eq!(action_is_none, true);

        assert_eq!(
            std::ptr::eq(app_state.keybindings, app_state.keybindings_root),
            true
        );

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
        let key_mode = KeyMode::default();
        keybindings_settings.insert(key_mode, root);

        let mut app_state = KeyBindingsTrie::builder(&keybindings_settings)
            .key_mode(key_mode)
            .build()?;

        assert_eq!(
            std::ptr::eq(app_state.keybindings, app_state.keybindings_root),
            true
        );

        let action_a = app_state
            .action(key_event_a)
            .expect("Method action should return Action");

        assert_eq!(action_a, Action::NoOp);
        assert_eq!(
            std::ptr::eq(app_state.keybindings, app_state.keybindings_root),
            false
        );

        let action_b = app_state
            .action(key_event_b)
            .expect("Method action should return Action");

        assert_eq!(action_b, Action::Quit);
        assert_eq!(
            std::ptr::eq(app_state.keybindings, app_state.keybindings_root),
            true
        );

        Ok(())
    }
}
