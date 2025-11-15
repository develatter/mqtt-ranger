///! Application state and main event loop for the TUI application.
///! This module defines the data structures and logic for managing
///! the state of the MQTT topics and their associated messages.


/// Association of an MQTT topic with its messages.
/// Each topic has a name and a list of messages received on that topic.
pub struct TopicActivity {
    pub name: String,
    pub messages: Vec<MessageActivity>,
}

/// Represents a single MQTT message activity,
pub struct MessageActivity {
    pub payload: String,
    pub timestamp: String,
}

/// Represents the overall state of the application,
/// including the list of topics and the currently selected topic.
pub struct AppState {
    pub topics: Vec<TopicActivity>,
    pub selected_index: usize,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            topics: Vec::new(),
            selected_index: 0,
        }
    }

    /// Move the selection to the next topic in the list.
    pub fn next(&mut self) {
        if !self.topics.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.topics.len();
        }
    }

    /// Move the selection to the previous topic in the list.
    pub fn previous(&mut self) {
        if !self.topics.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.topics.len() - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }
}


/// Represents the fields in the configuration form.
#[derive(Copy, Clone)]
pub enum FocusField {
    Host,
    Port,
}

/// Represents the state of the configuration form.
pub struct ConfigFormState {
    pub host: String,
    pub port: String,
    pub focus: FocusField,
    pub error: Option<String>,
}

impl ConfigFormState {
    pub fn new() -> Self {
        Self {
            host: "".into(),
            port: "".into(),
            focus: FocusField::Host,
            error: None,
        }
    }

    /// Move focus to the next field in the form.
    pub fn next_field(&mut self) {
        self.focus = match self.focus {
            FocusField::Host => FocusField::Port,
            FocusField::Port => FocusField::Host,
        };
    }

    /// Move focus to the previous field in the form.
    pub fn prev_field(&mut self) {
        self.focus = match self.focus {
            FocusField::Host => FocusField::Port,
            FocusField::Port => FocusField::Host,
        };
    }

    /// Insert a character into the currently focused field.
    pub fn insert_char(&mut self, c: char) {
        match self.focus {
            FocusField::Host => self.host.push(c),
            FocusField::Port => self.port.push(c),
        }
    }

    /// Delete the last character from the currently focused field.
    pub fn delete_char(&mut self) {
        match self.focus {
            FocusField::Host => {
                self.host.pop();
            }
            FocusField::Port => {
                self.port.pop();
            }
        }
    }
}

