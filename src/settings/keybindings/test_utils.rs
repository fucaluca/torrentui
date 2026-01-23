use crossterm::event::KeyEvent;

use crate::{action::Action, settings::keybindings::KeyBindingsNode};

pub struct KeyBindingsTestBuilder {
    key_event: Option<KeyEvent>,
    action: Option<Action>,
    description: Option<String>,
}

impl KeyBindingsTestBuilder {
    pub fn new() -> Self {
        Self {
            key_event: None,
            action: None,
            description: None,
        }
    }

    pub fn key_event(mut self, key_event: KeyEvent) -> Self {
        self.key_event = Some(key_event);

        self
    }

    pub fn action(mut self, action: Action) -> Self {
        self.action = Some(action);
        self
    }

    pub fn build(self) -> KeyBindingsNode {
        let mut keybindings_root = KeyBindingsNode::default();
        let mut keybindings_node = KeyBindingsNode::default();
        let key_event = self.key_event.expect("Field key_event must be specified");

        keybindings_node.action = self.action.expect("Field action must be specified");
        keybindings_node.description = self.description;
        keybindings_root.next.insert(key_event, keybindings_node);

        keybindings_root
    }
}
