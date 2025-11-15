use std::time::Duration;

use crate::{
    app::{ConfigFormState, FocusField},
    mqtt::MQTTConfig,
    tui::{Screen, centered_rect},
};

use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    Terminal,
    layout::{Alignment, Constraint, Direction, Layout},
    prelude::CrosstermBackend,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
};

/// MQTT Configuration Form Screen.
pub struct ConfigFormScreen<'a> {
    terminal: &'a mut Terminal<CrosstermBackend<std::io::Stdout>>,
    state: ConfigFormState,
    result: Option<MQTTConfig>,
}

impl<'a> ConfigFormScreen<'a> {
    pub fn new(terminal: &'a mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Self {
        Self {
            terminal,
            state: ConfigFormState::new(),
            result: None,
        }
    }

    /// The final result of the form (if completed).
    pub fn into_config(self) -> Option<MQTTConfig> {
        self.result
    }

    /// Renders the configuration form UI.
    fn render_config_screen_ui(f: &mut ratatui::Frame, state: &ConfigFormState) {
        let size = f.area();
        let total_area = centered_rect(40, 17, size);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(15),
                Constraint::Length(2),
            ])
            .split(total_area);

        let form_area = layout[0];
        let error_area = layout[1];

        let block = Block::default()
            .title("MQTT Configuration")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Thick);

        f.render_widget(block.clone(), form_area);

        let inner = block.inner(form_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .horizontal_margin(6)
            .vertical_margin(3)
            .constraints([
                Constraint::Length(3), // Host
                Constraint::Length(3), // Port
            ])
            .split(inner);

        // HOST FIELD
        let host_style = match state.focus {
            FocusField::Host => Style::default().fg(Color::Black).bg(Color::White),
            _ => Style::default(),
        };

        let host = Paragraph::new(state.host.as_str())
            .style(host_style)
            .block(Block::default().title("Host").borders(Borders::ALL));
        f.render_widget(host, chunks[0]);

        // PORT FIELD
        let port_style = match state.focus {
            FocusField::Port => Style::default().fg(Color::Black).bg(Color::White),
            _ => Style::default(),
        };

        let port = Paragraph::new(state.port.as_str())
            .style(port_style)
            .block(Block::default().title("Port").borders(Borders::ALL));
        f.render_widget(port, chunks[1]);

        // ERROR MESSAGE
        if let Some(err_msg) = &state.error {
            let error = Paragraph::new(err_msg.clone())
                .style(Style::default().fg(Color::Red))
                .alignment(Alignment::Center);
            f.render_widget(error, error_area);
        }
    }
}


impl Screen for ConfigFormScreen<'_> {
    fn run(&mut self) -> std::io::Result<()> {
        loop {
            let state = &self.state;

            self.terminal.draw(|f| {
                ConfigFormScreen::render_config_screen_ui(f, state);
            })?;

            if self.handle_input()? {
                break;
            }
        }

        Ok(())
    }

    fn handle_input(&mut self) -> std::io::Result<bool> {
        if !event::poll(Duration::from_millis(100))? {
            return Ok(false);
        }

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Tab | KeyCode::Down => {
                    self.state.next_field();
                }
                KeyCode::BackTab | KeyCode::Up => {
                    self.state.prev_field();
                }
                KeyCode::Char(c) => {
                    self.state.insert_char(c);
                }
                KeyCode::Backspace => {
                    self.state.delete_char();
                }
                KeyCode::Enter => {
                    if let Ok(port) = self.state.port.parse::<u16>() {
                        self.result = Some(MQTTConfig {
                            host: self.state.host.clone(),
                            port,
                        });
                        return Ok(true); // formulario completado
                    } else {
                        self.state.error = Some("Port must be a valid number".into());
                    }
                }
                KeyCode::Esc => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "User cancelled config form",
                    ));
                }
                _ => {}
            }
        }
        Ok(false)
    }
}
