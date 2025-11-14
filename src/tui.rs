use std::time::Duration;

use crate::{app::AppState as App, mqtt::MQTTConfig};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    layout::{Constraint, Direction, Layout},
    prelude::CrosstermBackend,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
};

///! Initializes the terminal in raw mode and sets up the alternate screen for the TUI application.
pub fn init_terminal()
-> Result<Terminal<CrosstermBackend<std::io::Stdout>>, Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

///! Restores the terminal to its original state by disabling raw mode and leaving the alternate screen.
pub fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) -> Result<(), Box<dyn std::error::Error>> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

///! Renders the UI components of the TUI application.
pub fn run_topic_activity_ui<B: ratatui::backend::Backend>(f: &mut ratatui::Frame, app: &App) {
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

    f.render_stateful_widget(
        topics_list,
        chunks[0],
        &mut make_list_state(app.selected_index),
    );

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

///! Represents the fields in the form.
#[derive(Copy, Clone)]
enum FocusField {
    ClientId,
    Host,
    Port,
}

///! Represents the state of the configuration form.
struct ConfigFormState {
    client_id: String,
    host: String,
    port: String,
    focus: FocusField,
    error: Option<String>,
}

impl ConfigFormState {
    fn new() -> Self {
        Self {
            client_id: "".into(),
            host: "".into(),
            port: "".into(),
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

    fn insert_char(&mut self, c: char) {
        match self.focus {
            FocusField::ClientId => self.client_id.push(c),
            FocusField::Host => self.host.push(c),
            FocusField::Port => self.port.push(c),
        }
    }

    fn delete_char(&mut self) {
        match self.focus {
            FocusField::ClientId => {
                self.client_id.pop();
            }
            FocusField::Host => {
                self.host.pop();
            }
            FocusField::Port => {
                self.port.pop();
            }
        }
    }
}

///! Helper function to create a centered rectangle for the form.
fn centered_rect(width: u16, height: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
    let x = r.x + (r.width.saturating_sub(width)) / 2;
    let y = r.y + (r.height.saturating_sub(height)) / 2;
    ratatui::layout::Rect::new(x, y, width, height)
}


///! Runs the configuration form to collect MQTT connection details from the user.
pub fn run_config_form(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) -> Result<MQTTConfig, Box<dyn std::error::Error>> {
    let mut state = ConfigFormState::new();

    loop {
        terminal.draw(|f| {
            let size = f.area();
            let outer_block = Block::default().borders(Borders::NONE);
            f.render_widget(outer_block.clone(), size);

            let area = centered_rect(40, 15, size);

            let inner = outer_block.inner(area);
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .horizontal_margin(6) 
                .vertical_margin(3) 
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                ])
                .split(inner);

            let block = Block::default()
                .title("MQTT Configuration")
                .borders(Borders::ALL)
                .border_type(BorderType::Thick);

            f.render_widget(block, inner);

            // ID Field
            let id_style = if let FocusField::ClientId = state.focus {
                Style::default().fg(Color::Black).bg(Color::White)
            } else {
                Style::default()
            };
            let id = Paragraph::new::<&str>(state.client_id.as_ref())
                .style(id_style)
                .block(Block::default().title("Name").borders(Borders::ALL));
            f.render_widget(id, chunks[0]);

            // HOST Field
            let host_style = if let FocusField::Host = state.focus {
                Style::default().fg(Color::Black).bg(Color::White)
            } else {
                Style::default()
            };
            let host = Paragraph::new::<&str>(state.host.as_ref())
                .style(host_style)
                .block(Block::default().title("Host").borders(Borders::ALL));
            f.render_widget(host, chunks[1]);

            // PORT Field
            let port_style = if let FocusField::Port = state.focus {
                Style::default().fg(Color::Black).bg(Color::White)
            } else {
                Style::default()
            };
            let port = Paragraph::new::<&str>(state.port.as_ref())
                .style(port_style)
                .block(Block::default().title("Port").borders(Borders::ALL));
            f.render_widget(port, chunks[2]);
        })?;
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Tab | KeyCode::Down => {
                        state.next_field();
                    }
                    KeyCode::BackTab | KeyCode::Up => {
                        state.prev_field();
                    }
                    KeyCode::Char(c) => {
                        state.insert_char(c);
                    }
                    KeyCode::Backspace => {
                        state.delete_char();
                    }
                    KeyCode::Enter => {
                        // Validate port
                        if let Ok(port) = state.port.parse::<u16>() {
                            return Ok(MQTTConfig {
                                id: state.client_id.clone(),
                                host: state.host.clone(),
                                port,
                            });
                        } else {
                            state.error = Some("Port must be a valid number".into());
                        }
                    }
                    KeyCode::Esc => {
                        return Err("User cancelled the configuration form".into());
                    }
                    _ => {}
                }
            }
        }
    }
}


