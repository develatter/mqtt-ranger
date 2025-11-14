///! TUI module for mqtt-ranger: Handles terminal initialization, splash screen,
///! configuration form, and main event loop for displaying MQTT topic activity.
///! This module uses the ratatui and crossterm crates to create a user-friendly
///! terminal interface.

use std::{sync::{Arc, Mutex}, time::{Duration, Instant}};

use crate::{
    app::{AppState as App, ConfigFormState, FocusField}, 
    mqtt::MQTTConfig
};
use crossterm::{
    event::{self, Event, KeyCode,},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    layout::{Alignment, Constraint, Direction, Layout},
    prelude::CrosstermBackend,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
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

///! Helper function to create a ListState with the selected index.
fn make_list_state(selected: usize) -> ratatui::widgets::ListState {
    let mut state = ratatui::widgets::ListState::default();
    state.select(Some(selected));
    state
}


///! Helper function to create a centered rectangle for the form.
fn centered_rect(width: u16, height: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
    let clamped_width = width.min(r.width);
    let clamped_height = height.min(r.height);

    let x = r.x + (r.width.saturating_sub(clamped_width)) / 2;
    let y = r.y + (r.height.saturating_sub(clamped_height)) / 2;

    ratatui::layout::Rect::new(x, y, clamped_width, clamped_height)
}

///! Runs the splash screen until a key is pressed.
pub fn run_splash_screen<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
) -> std::io::Result<()> {
    loop {
        terminal.draw(|f| {
            render_splash_screen::<B>(f);
        })?;

        if crossterm::event::poll(Duration::from_millis(100))? {
            if let crossterm::event::Event::Key(_) = crossterm::event::read()? {
                break;
            }
        }
    }
    Ok(())
}

///! Renders the splash screen with ASCII art.
fn render_splash_screen<B: ratatui::backend::Backend>(f: &mut ratatui::Frame) {
    let size = f.area();

    let ascii_art = r#"
▄▄   ▄▄  ▄▄▄ ▄▄▄▄▄▄ ▄▄▄▄▄▄    ▄▄▄▄   ▄▄▄  ▄▄  ▄▄  ▄▄▄▄ ▄▄▄▄▄ ▄▄▄▄  
██▀▄▀██ ██▀██  ██     ██  ▄▄▄ ██▄█▄ ██▀██ ███▄██ ██ ▄▄ ██▄▄  ██▄█▄ 
██   ██ ▀███▀  ██     ██      ██ ██ ██▀██ ██ ▀██ ▀███▀ ██▄▄▄ ██ ██ 
           ▀▀                                                    
"#;

    let show_art = size.width >= 80 && size.height >= 20;

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(if show_art {
            vec![
                Constraint::Percentage(40),
                Constraint::Min(7),    // art
                Constraint::Length(3), // message
                Constraint::Percentage(40),
            ]
        } else {
            vec![
                Constraint::Percentage(45),
                Constraint::Length(3),
                Constraint::Percentage(45),
            ]
        })
        .split(size);

    if show_art {
        let art_paragraph = Paragraph::new(Text::from(ascii_art))
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center)
            .block(Block::default());
        f.render_widget(art_paragraph, layout[1]);
        f.render_widget(
            Paragraph::new("< Press any key to continue >")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center),
            layout[2],
        );
    } else {
        f.render_widget(
            Paragraph::new("< Press any key to continue >")
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Center),
            layout[1],
        );
    }
}


///! Runs the configuration form to collect MQTT connection details from the user.
pub fn run_config_form_screen(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) -> Result<MQTTConfig, Box<dyn std::error::Error>> {
    let mut state = ConfigFormState::new();

    loop {
        terminal.draw(|f| {
            let size = f.area();

            let total_area = centered_rect(40, 17, size);

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(15), 
                    Constraint::Length(2),  // error message
                ])
                .split(total_area);

            let form_area = layout[0];
            let error_area = layout[1];

            //--- FORM BLOCK ---
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

            // --- HOST FIELD ---
            let host_style = if let FocusField::Host = state.focus {
                Style::default().fg(Color::Black).bg(Color::White)
            } else {
                Style::default()
            };
            let host = Paragraph::new::<&str>(state.host.as_ref())
                .style(host_style)
                .block(Block::default().title("Host").borders(Borders::ALL));
            f.render_widget(host, chunks[0]);

            // --- PORT FIELD ---
            let port_style = if let FocusField::Port = state.focus {
                Style::default().fg(Color::Black).bg(Color::White)
            } else {
                Style::default()
            };
            let port = Paragraph::new::<&str>(state.port.as_ref())
                .style(port_style)
                .block(Block::default().title("Port").borders(Borders::ALL));
            f.render_widget(port, chunks[1]);

            // --- ERROR MESSAGE ---
            if let Some(err_msg) = &state.error {
                let error = Paragraph::new(err_msg.clone())
                    .style(Style::default().fg(Color::Red))
                    .alignment(Alignment::Center);
                f.render_widget(error, error_area);
            }
        })?;

        // INPUT HANDLING
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Tab | KeyCode::Down => {
                        state.next_field();
                        state.error = None;
                    }
                    KeyCode::BackTab | KeyCode::Up => {
                        state.prev_field();
                        state.error = None;
                    }
                    KeyCode::Char(c) => {
                        state.insert_char(c);
                        state.error = None;
                    }
                    KeyCode::Backspace => {
                        state.delete_char();
                        state.error = None;
                    }
                    KeyCode::Enter => {
                        if let Ok(port) = state.port.parse::<u16>() {
                            return Ok(MQTTConfig {
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


///! Main event loop for running the TUI application.
pub fn run_topic_activity_screen<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: Arc<Mutex<App>>
) -> std::io::Result<()> {
    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();
    
    loop {
        {
            // Draw the UI.
            let app_state = app.lock().unwrap();
            terminal.draw(|f| render_topic_activity_ui::<B>(f, &*app_state))?;
        }

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        // Handle input events.
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
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


///! Renders the UI components of the TUI application.
pub fn render_topic_activity_ui<B: ratatui::backend::Backend>(f: &mut ratatui::Frame, app: &App) {
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
