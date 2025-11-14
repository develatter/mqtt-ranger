use crate::app::AppState as App;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

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

fn make_list_state(selected: usize) -> ratatui::widgets::ListState {
    let mut state = ratatui::widgets::ListState::default();
    state.select(Some(selected));
    state
}