use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::{
    app::TopicActivityMenuState,
    tui::{Screen, make_list_state},
};

use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    Terminal,
    layout::{Constraint, Direction, Layout},
    prelude::CrosstermBackend,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

/// Screen for displaying topic activity.
pub struct TopicActivityScreen<'a> {
    terminal: &'a mut Terminal<CrosstermBackend<std::io::Stdout>>,
    menu_state: Arc<Mutex<TopicActivityMenuState>>,
    tick_rate: Duration,
    last_tick: Instant,
}

impl<'a> TopicActivityScreen<'a> {
    pub fn new(
        terminal: &'a mut Terminal<CrosstermBackend<std::io::Stdout>>,
        menu_state: Arc<Mutex<TopicActivityMenuState>>,
    ) -> Self {
        Self {
            terminal,
            menu_state,
            tick_rate: Duration::from_millis(250),
            last_tick: Instant::now(),
        }
    }

    /// Renders the topic activity screen UI.
    fn render_topic_activity_screen_ui(f: &mut ratatui::Frame, app: &TopicActivityMenuState) {
        let size = f.area();

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(30), // topic list
                    Constraint::Percentage(70), // activity
                ]
                .as_ref(),
            )
            .split(size);

        // --- Topic list ---
        let items: Vec<ListItem> = app
            .topics
            .iter()
            .map(|t| ListItem::new(Line::from(Span::raw(t.name.clone()))))
            .collect();

        let topics_list = List::new(items)
            .block(Block::default().title("Topics").borders(Borders::ALL))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::REVERSED),
            );

        f.render_stateful_widget(
            topics_list,
            chunks[0],
            &mut make_list_state(app.selected_index),
        );

        // --- Activity panel ---
        let activity_text = if let Some(topic) = app.topics.get(app.selected_index) {
            let mut lines = vec![Line::from(Span::styled(
                format!("[{}]", topic.name),
                Style::default().add_modifier(Modifier::BOLD),
            ))];

            lines.push(Line::from(""));

            if topic.messages.is_empty() {
                lines.push(Line::from("No messages yet..."));
            } else {
                for msg in &topic.messages {
                    let timestamp_span = Span::styled(
                        format!("<{}>: ", msg.timestamp),
                        Style::default()
                            .fg(Color::LightRed)
                            .add_modifier(Modifier::BOLD),
                    );

                    let payload_span = Span::raw(&msg.payload);
                    lines.push(Line::from(vec![timestamp_span, payload_span]));
                }
            }
            lines
        } else {
            vec![Line::from("No topics")]
        };

        let activity = Paragraph::new(activity_text)
            .block(Block::default().title("Activity").borders(Borders::ALL));

        f.render_widget(activity, chunks[1]);
    }
}

impl Screen for TopicActivityScreen<'_> {
    fn run(&mut self) -> std::io::Result<()> {
        loop {
            {
                let menu_guard = self
                    .menu_state
                    .lock()
                    .map_err(|_| {
                        std::io::Error::new(
                            std::io::ErrorKind::Other, "App mutex poisoned"
                        )
                    })?;

                self.terminal.draw(|f| {
                    TopicActivityScreen::render_topic_activity_screen_ui(f, &*menu_guard);
                })?;
            }

            if self.handle_input()? {
                break;
            }

            // Tick
            if self.last_tick.elapsed() >= self.tick_rate {
                self.last_tick = Instant::now();
            }
        }

        Ok(())
    }

    fn handle_input(&mut self) -> std::io::Result<bool> {
        let timeout = self
            .tick_rate
            .checked_sub(self.last_tick.elapsed())
            .unwrap_or(Duration::from_secs(0));

        if !event::poll(timeout)? {
            return Ok(false);
        }

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(true),

                KeyCode::Down => {
                    if let Ok(mut topic_activity_menu_state) = self.menu_state.lock() {
                        topic_activity_menu_state.next();
                    }
                }
                KeyCode::Up => {
                    if let Ok(mut topic_activity_menu_state) = self.menu_state.lock() {
                        topic_activity_menu_state.previous();
                    }
                }
                _ => {}
            }
        }

        Ok(false)
    }
}
