use ratatui::Terminal;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use crate::app::AppState as App;
use crossterm::event::{self, Event as CEvent, KeyCode};
use crate::tui::ui;

pub struct TopicActivity {
    pub name: String,
    pub messages: Vec<String>,
}

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

    pub fn next(&mut self) {
        if !self.topics.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.topics.len();
        }
    }

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


pub fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: Arc<Mutex<App>>,
    tick_rate: Duration,
) -> std::io::Result<()> {
    let mut last_tick = Instant::now();
    
    loop {
        {
            let app_state = app.lock().unwrap();
            terminal.draw(|f| ui::<B>(f, &*app_state))?;
        }

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

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
