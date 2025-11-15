///! TUI module for mqtt-ranger: Handles terminal initialization, splash screen,
///! configuration form, and main event loop for displaying MQTT topic activity.
///! This module uses the ratatui and crossterm crates to create a user-friendly
///! terminal interface.

use crossterm::{
    execute,
    terminal::{
        EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
    },
};
use ratatui::{
    Terminal,
    prelude::CrosstermBackend
};

pub mod splash;
pub mod config_form;
pub mod topic_activity;


/// Initializes the terminal in raw mode and sets up the alternate screen for the TUI application.
pub fn init_terminal()
-> Result<Terminal<CrosstermBackend<std::io::Stdout>>, Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Restores the terminal to its original state by disabling raw mode and leaving the alternate screen.
pub fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) -> Result<(), Box<dyn std::error::Error>> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

/// Helper function to create a ListState with the selected index.
fn make_list_state(selected: usize) -> ratatui::widgets::ListState {
    let mut state = ratatui::widgets::ListState::default();
    state.select(Some(selected));
    state
}

/// Helper function to create a centered rectangle for the form.
fn centered_rect(width: u16, height: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
    let clamped_width = width.min(r.width);
    let clamped_height = height.min(r.height);

    let x = r.x + (r.width.saturating_sub(clamped_width)) / 2;
    let y = r.y + (r.height.saturating_sub(clamped_height)) / 2;

    ratatui::layout::Rect::new(x, y, clamped_width, clamped_height)
}

/// Trait representing a screen in the TUI application.
pub trait Screen {

    /// Runs the main event loop for the screen.
    fn run(&mut self) -> std::io::Result<()>;

    /// Handles input events for the screen.
    fn handle_input(&mut self) -> std::io::Result<bool>;
}