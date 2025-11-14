///! Application state and main event loop for the TUI application.
///! This module defines the data structures and logic for managing
///! the state of the MQTT topics and their associated messages.


use ratatui::Terminal;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use crate::app::AppState as App;
use crossterm::event::{self, Event as CEvent, KeyCode};
use crate::tui::run_topic_activity_ui;

///! Association of an MQTT topic with its messages.
///! Each topic has a name and a list of messages received on that topic.
pub struct TopicActivity {
    pub name: String,
    pub messages: Vec<String>,
}

///! Represents the overall state of the application,
///! including the list of topics and the currently selected topic.
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

    ///! Move the selection to the next topic in the list.
    pub fn next(&mut self) {
        if !self.topics.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.topics.len();
        }
    }

    ///! Move the selection to the previous topic in the list.
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


///! Main event loop for running the TUI application.
pub fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: Arc<Mutex<App>>
) -> std::io::Result<()> {
    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();
    
    loop {
        {
            // Draw the UI.
            let app_state = app.lock().unwrap();
            terminal.draw(|f| run_topic_activity_ui::<B>(f, &*app_state))?;
        }

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        // Handle input events.
        if crossterm::event::poll(timeout)? {
            if let CEvent::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Down => {
                        if let Ok(mut app_state) = app.lock() {
                            app_state.next();
                        }
                    }
                    KeyCode::Up => {
                        if let Ok(mut app_state) = app.lock() {
                            app_state.previous();
                        }
                    }
                    _ => {}
                }
            }
        }

        last_tick = Instant::now();
    }
}
