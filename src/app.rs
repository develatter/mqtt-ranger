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

#[cfg(test)]
mod tests {
    use super::*;

    // AppState tests
    #[test]
    fn test_activity_list_generation_with_empty_topics() {
        let app_state = AppState::new();
        assert!(app_state.topics.is_empty());
        assert_eq!(app_state.selected_index, 0);
    }

    #[test]
    fn test_activity_list_generation_with_non_empty_topics() {
        let mut app_state = AppState::new();
        
        // Add topics with messages
        app_state.topics.push(TopicActivity {
            name: "topic1".to_string(),
            messages: vec![
                MessageActivity {
                    payload: "message1".to_string(),
                    timestamp: "2024-01-01 12:00:00".to_string(),
                },
                MessageActivity {
                    payload: "message2".to_string(),
                    timestamp: "2024-01-01 12:01:00".to_string(),
                },
            ],
        });
        
        app_state.topics.push(TopicActivity {
            name: "topic2".to_string(),
            messages: vec![
                MessageActivity {
                    payload: "message3".to_string(),
                    timestamp: "2024-01-01 12:02:00".to_string(),
                },
            ],
        });
        
        // Verify topics are stored correctly
        assert_eq!(app_state.topics.len(), 2);
        assert_eq!(app_state.topics[0].name, "topic1");
        assert_eq!(app_state.topics[0].messages.len(), 2);
        assert_eq!(app_state.topics[1].name, "topic2");
        assert_eq!(app_state.topics[1].messages.len(), 1);
    }

    // ConfigFormState tests
    #[test]
    fn test_insert_char_appends_correctly() {
        let mut state = ConfigFormState::new();
        
        // Test inserting into host field (default focus)
        state.insert_char('l');
        state.insert_char('o');
        state.insert_char('c');
        state.insert_char('a');
        state.insert_char('l');
        assert_eq!(state.host, "local");
        
        // Switch to port field and insert
        state.next_field();
        state.insert_char('1');
        state.insert_char('8');
        state.insert_char('8');
        state.insert_char('3');
        assert_eq!(state.port, "1883");
    }

    #[test]
    fn test_delete_char_removes_correctly() {
        let mut state = ConfigFormState::new();
        
        // Add characters to host
        state.host = "localhost".to_string();
        state.delete_char();
        assert_eq!(state.host, "localhos");
        state.delete_char();
        assert_eq!(state.host, "localho");
        
        // Switch to port and test deletion
        state.next_field();
        state.port = "1883".to_string();
        state.delete_char();
        assert_eq!(state.port, "188");
        
        // Delete all characters
        state.delete_char();
        state.delete_char();
        state.delete_char();
        assert_eq!(state.port, "");
        
        // Delete from empty field should not panic
        state.delete_char();
        assert_eq!(state.port, "");
    }

    #[test]
    fn test_next_field_cycles_focus_forward() {
        let mut state = ConfigFormState::new();
        
        // Start at Host
        assert!(matches!(state.focus, FocusField::Host));
        
        // Move to Port
        state.next_field();
        assert!(matches!(state.focus, FocusField::Port));
        
        // Move back to Host (cycling)
        state.next_field();
        assert!(matches!(state.focus, FocusField::Host));
    }

    #[test]
    fn test_prev_field_cycles_focus_backward() {
        let mut state = ConfigFormState::new();
        
        // Start at Host
        assert!(matches!(state.focus, FocusField::Host));
        
        // Move to Port (going backward from Host)
        state.prev_field();
        assert!(matches!(state.focus, FocusField::Port));
        
        // Move back to Host
        state.prev_field();
        assert!(matches!(state.focus, FocusField::Host));
    }

    #[test]
    fn test_port_validation_rejects_non_numeric() {
        let mut state = ConfigFormState::new();
        state.port = "abc".to_string();
        
        // Attempt to parse port
        let result = state.port.parse::<u16>();
        assert!(result.is_err());
    }

    #[test]
    fn test_port_validation_accepts_valid_numeric() {
        let mut state = ConfigFormState::new();
        state.port = "1883".to_string();
        
        // Attempt to parse port
        let result = state.port.parse::<u16>();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1883);
    }

    #[test]
    fn test_error_state_is_properly_set() {
        let mut state = ConfigFormState::new();
        
        // Initially no error
        assert!(state.error.is_none());
        
        // Set an error
        state.error = Some("Port must be a valid number".to_string());
        assert!(state.error.is_some());
        assert_eq!(state.error.as_ref().unwrap(), "Port must be a valid number");
        
        // Clear error
        state.error = None;
        assert!(state.error.is_none());
    }
}
