use std::time::Duration;

use crate::{app::AppState as App, mqtt::MQTTConfig};
use crossterm::{event::{self, Event, KeyCode}, execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode}};
use ratatui::{
    Terminal, layout::{Constraint, Direction, Layout}, prelude::CrosstermBackend, style::{Modifier, Style}, text::{Line, Span}, widgets::{Block, Borders, List, ListItem, Paragraph}
};

///! Initializes the terminal in raw mode and sets up the alternate screen for the TUI application.
pub fn init_terminal() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>, Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

///! Restores the terminal to its original state by disabling raw mode and leaving the alternate screen.
pub fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<(), Box<dyn std::error::Error>> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}



///! Renders the UI components of the TUI application.
pub fn ui<B: ratatui::backend::Backend>(f: &mut ratatui::Frame, app: &App) {
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

    // --- Topic list panel ---
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

    f.render_stateful_widget(topics_list, chunks[0], &mut make_list_state(app.selected_index));

    // --- Activity panel ---
    let activity_text = if let Some(topic) = app.topics.get(app.selected_index) {
        let mut lines = vec![Line::from(Span::styled(
            format!("Topic: {}", topic.name),
            Style::default().add_modifier(Modifier::BOLD),
        ))];
        lines.push(Line::from(""));

        if topic.messages.is_empty() {
            lines.push(Line::from("No messages yet..."));
        } else {
            for msg in &topic.messages {
                lines.push(Line::from(msg.clone()));
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

///! Helper function to create a ListState with the selected index.
fn make_list_state(selected: usize) -> ratatui::widgets::ListState {
    let mut state = ratatui::widgets::ListState::default();
    state.select(Some(selected));
    state
}


#[derive(Copy, Clone)]
enum FocusField {
    ClientId,
    Host,
    Port,
}

struct FormState {
    client_id: String,
    host: String,
    port: String, 
    focus: FocusField,
    error: Option<String>,
}

impl FormState {
    fn new() -> Self {
        Self {
            client_id: "mqtt-tui-client".into(),
            host: "localhost".into(),
            port: "1883".into(),
            focus: FocusField::ClientId,
            error: None,
        }
    }

    fn next_field(&mut self) {
        self.focus = match self.focus {
            FocusField::ClientId => FocusField::Host,
            FocusField::Host => FocusField::Port,
            FocusField::Port => FocusField::ClientId,
        };
    }

    fn prev_field(&mut self) {
        self.focus = match self.focus {
            FocusField::ClientId => FocusField::Port,
            FocusField::Host => FocusField::ClientId,
            FocusField::Port => FocusField::Host,
        };
    }
}

pub fn run_config_form(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<MQTTConfig, Box<dyn std::error::Error>> {
    let mut state = FormState::new();

    let res = loop {
        terminal.draw(|f| {
            let size = f.area();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(3),  // title
                        Constraint::Length(3),  // client_id
                        Constraint::Length(3),  // host
                        Constraint::Length(3),  // port
                        Constraint::Length(3),  // info / error
                        Constraint::Min(0),     // filler
                    ]
                    .as_ref(),
                )
                .direction(Direction::Horizontal)
                .split(size);

            // Title
            let title = Paragraph::new(Span::from(Span::styled(
                "MQTT Configuration",
                Style::default().add_modifier(Modifier::BOLD),
            )))
            .block(Block::default().borders(Borders::BOTTOM));
            f.render_widget(title, chunks[0]);

            // Helper to draw a field
            let draw_field = |label: String, value: String, focused: bool| {
                let mut text = Vec::new();
                text.push(Line::from(label.clone()));
                text.push(Line::from(value.clone()));

                let mut block = Block::default().borders(Borders::ALL);
                let mut style = Style::default();

                if focused {
                    style = style.add_modifier(Modifier::BOLD | Modifier::REVERSED);
                }

                Paragraph::new(text).block(block).style(style)
            };

            // client_id
            let client_widget = draw_field(
                "Client ID:".to_string(),
                state.client_id.clone(),
                matches!(state.focus, FocusField::ClientId),
            );

            f.render_widget(client_widget, chunks[1]);

            // host
            let host_widget = draw_field(
                "Host:".to_string(),
                state.host.clone(),
                matches!(state.focus, FocusField::Host),
            );
            f.render_widget(host_widget, chunks[2]);

            // port
            let port_widget = draw_field(
                "Port:".to_string(),
                state.port.clone(),
                matches!(state.focus, FocusField::Port),
            );
            f.render_widget(port_widget, chunks[3]);

            // Info / error message
            let info_text = if let Some(err) = &state.error {
                Span::from(Span::styled(
                    format!("Error: {err}"),
                    Style::default().add_modifier(Modifier::BOLD),
                ))
            } else {
                Span::from("Tab / ↑ / ↓ to change field · Enter to accept · Esc to quit")
            };

            let info = Paragraph::new(info_text);
            f.render_widget(info, chunks[4]);
        })?;

        // Event handling
        if event::poll(Duration::from_millis(200))? {
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Esc => {
                        // cancel: exit with error
                        break Err("configuration cancelled by user".into());
                    }

                    KeyCode::Tab | KeyCode::Down => {
                        state.next_field();
                    }
                    KeyCode::Up => {
                        state.prev_field();
                    }

                    KeyCode::Backspace => {
                        let target = match state.focus {
                            FocusField::ClientId => &mut state.client_id,
                            FocusField::Host => &mut state.host,
                            FocusField::Port => &mut state.port,
                        };
                        target.pop();
                    }

                    KeyCode::Char(c) => {
                        let target = match state.focus {
                            FocusField::ClientId => &mut state.client_id,
                            FocusField::Host => &mut state.host,
                            FocusField::Port => &mut state.port,
                        };
                        target.push(c);
                    }

                    KeyCode::Enter => {
                        // Validate and return config
                        match state.port.parse::<u16>() {
                            Ok(port) => {
                                let cfg = MQTTConfig {
                                    id: state.client_id.clone(),
                                    host: state.host.clone(),
                                    port,
                                };
                                break Ok(cfg);
                            }
                            Err(_) => {
                                state.error = Some("Port must be a valid number (0-65535)".into());
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    };
    res
}

