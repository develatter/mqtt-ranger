use std::time::Duration;

use crate::{
    app::{ConfigFormState, FocusField},
    mqtt::MQTTConfig,
    tui::{Screen, centered_rect},
};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration as StdDuration, Instant};

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
    pending_conn: Option<Receiver<Result<(), String>>>,
    last_spinner_tick: Instant,
}

impl<'a> ConfigFormScreen<'a> {
    pub fn new(terminal: &'a mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Self {
        Self {
            terminal,
            state: ConfigFormState::new(),
            result: None,
            pending_conn: None,
            last_spinner_tick: Instant::now(),
        }
    }

    fn update_spinner(&mut self, duration: u64) {
        let now = Instant::now();
        if now.duration_since(self.last_spinner_tick) >= StdDuration::from_millis(duration) {
            self.state.spinner_idx = (self.state.spinner_idx + 1) % 4;
            self.last_spinner_tick = now;
        }
    }

    // Start a background thread to validate the broker and store the receiver
    fn spawn_validation_thread(&mut self, host: String, port: u16, timeout_secs: u64) {
        let (tx, rx): (mpsc::Sender<Result<(), String>>, Receiver<Result<(), String>>) = mpsc::channel();

        thread::spawn(move || {
            let res = crate::mqtt::validate_broker(&host, port, timeout_secs)
                .map_err(|e| e.to_string());
            let _ = tx.send(res);
        });

        self.pending_conn = Some(rx);
    }

    // Process any pending connection result and update state accordingly.
    fn process_pending_conn(&mut self) {
        if let Some(rx) = &self.pending_conn {
            match rx.try_recv() {
                Ok(Ok(())) => {
                    // success: complete form
                    if let Ok(port) = self.state.port.parse::<u16>() {
                        self.result = Some(MQTTConfig {
                            host: self.state.host.clone(),
                            port,
                        });
                    } else {
                        self.state.error = Some("Port must be a valid number".into());
                    }
                }
                Ok(Err(_)) => {
                    self.state.error = Some(format!("Host unreachable: {}", self.state.host));
                    self.state.connecting = false;
                    self.state.spinner_idx = 0;
                    self.pending_conn = None;
                }
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.state.error = Some("Connection check failed (disconnected)".into());
                    self.state.connecting = false;
                    self.state.spinner_idx = 0;
                    self.pending_conn = None;
                }
            }
        }
    }

    // Handle the Enter key press: start validation or ignore if already connecting
    fn on_enter_pressed(&mut self) {
        if let Ok(port) = self.state.port.parse::<u16>() {
            if self.state.connecting {
                return;
            }

            self.state.error = None;
            self.state.connecting = true;
            self.state.spinner_idx = 0;

            let host = self.state.host.clone();
            self.spawn_validation_thread(host, port, 5);
        } else {
            self.state.error = Some("Port must be a valid number".into());
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

        // ERROR / CONNECTING MESSAGE
        if state.connecting {
            // spinner handled in state.spinner_idx (0..=3)
            let dots = ".".repeat(state.spinner_idx);
            let connecting = format!("Connecting{}", dots);
            let connecting = Paragraph::new(connecting)
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Center);
            f.render_widget(connecting, error_area);
        } else if let Some(err_msg) = &state.error {
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
            // Draw UI
            let state = &self.state;
            self.terminal.draw(|f| {
                ConfigFormScreen::render_config_screen_ui(f, state);
            })?;

            // Update spinner every 300ms when connecting
            if self.state.connecting {
                self.update_spinner(300);
            }

            // Process any pending connection result
            self.process_pending_conn();

            // If process_pending_conn set a result, finish
            if self.result.is_some() {
                return Ok(());
            }

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
                    self.on_enter_pressed();
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
