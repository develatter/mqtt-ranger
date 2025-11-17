use std::{
    time::Duration
};

use crate::tui::Screen;

use ratatui::{
    Terminal,
    layout::{Alignment, Constraint, Direction, Layout},
    prelude::CrosstermBackend,
    style::{Color, Style},
    text::{Text},
    widgets::{Block, Paragraph},
};


/// Splash screen displayed at application startup.
pub struct SplashScreen<'a> {
    terminal: &'a mut Terminal<CrosstermBackend<std::io::Stdout>>,
}

impl<'a> SplashScreen<'a> {
    pub fn new(terminal: &'a mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Self {
        Self { terminal }
    }

    /// Renders the splash screen with ASCII art.
    fn render_splash_screen_ui(f: &mut ratatui::Frame) {
        let size = f.area();

        let ascii_art = r#"
▄▄   ▄▄  ▄▄▄ ▄▄▄▄▄▄ ▄▄▄▄▄▄    ▄▄▄▄   ▄▄▄  ▄▄  ▄▄  ▄▄▄▄ ▄▄▄▄▄ ▄▄▄▄  
██▀▄▀██ ██▀██  ██     ██  ▄▄▄ ██▄█▄ ██▀██ ███▄██ ██ ▄▄ ██▄▄  ██▄█▄ 
██   ██ ▀███▀  ██     ██      ██ ██ ██▀██ ██ ▀██ ▀███▀ ██▄▄▄ ██ ██ 
           ▀▀                                                    
"#;

        let prompt_text = "< Press any key to continue >";
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
                Paragraph::new(prompt_text)
                    .style(Style::default().fg(Color::DarkGray))
                    .alignment(Alignment::Center),
                layout[2],
            );
        } else {
            f.render_widget(
                Paragraph::new(prompt_text)
                    .style(Style::default().fg(Color::White))
                    .alignment(Alignment::Center),
                layout[1],
            );
        }
    }
}

impl Screen for SplashScreen<'_> {
    fn run(&mut self) -> std::io::Result<()> {
        loop {
            self.terminal.draw(|f| {
                Self::render_splash_screen_ui(f);
            })?;

            if self.handle_input()? {
                return Ok(());
            }
        }
    }

    fn handle_input(&mut self) -> std::io::Result<bool> {
        if crossterm::event::poll(Duration::from_millis(100))? {
            if let crossterm::event::Event::Key(_) = crossterm::event::read()? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
