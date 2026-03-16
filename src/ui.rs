use crate::app::{App, Mode};
use crate::config::Config;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

// Catppuccin Mocha Palette
const CRUST: Color = Color::Rgb(17, 17, 27);
const SURFACE0: Color = Color::Rgb(49, 50, 68);
const SURFACE1: Color = Color::Rgb(69, 71, 90);
const TEXT: Color = Color::Rgb(205, 214, 244);
const BLUE: Color = Color::Rgb(137, 180, 250);
const GREEN: Color = Color::Rgb(166, 227, 161);
const LAVENDER: Color = Color::Rgb(180, 190, 254);
const MAUVE: Color = Color::Rgb(203, 166, 247);
const PEACH: Color = Color::Rgb(250, 179, 135);

pub fn draw(f: &mut Frame, app: &mut App, _config: &Config) {
    let size = f.area();

    // Background
    f.render_widget(Block::default().style(Style::default().bg(CRUST)), size);

    // Main layout with horizontal padding
    let outer_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(1), // Left padding
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Right padding
        ])
        .split(size);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Top padding
            Constraint::Min(3),    // List
            Constraint::Length(3), // Input area
            Constraint::Length(1), // Bottom padding
            Constraint::Length(1), // Status bar
        ])
        .split(outer_layout[1]);

    let max_key_len = app
        .vars
        .iter()
        .map(|v| v.key.len())
        .max()
        .unwrap_or(20)
        .min(40); // Cap at 40 to prevent long keys from hiding values

    // List
    let items: Vec<ListItem> = app
        .vars
        .iter()
        .enumerate()
        .map(|(i, var)| {
            let is_selected = i == app.selected;
            
            let val = if is_selected && matches!(app.mode, Mode::Insert) {
                app.input.value()
            } else {
                &var.value
            };

            let key_style = if is_selected {
                Style::default().fg(CRUST).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(LAVENDER)
            };

            let value_style = if is_selected {
                Style::default().fg(CRUST)
            } else {
                Style::default().fg(TEXT)
            };

            let line = Line::from(vec![
                Span::styled(format!(" {:<width$} ", var.key, width = max_key_len), key_style),
                Span::styled("│ ", Style::default().fg(SURFACE1)),
                Span::styled(format!(" {} ", val), value_style),
            ]);

            let item_style = if is_selected {
                Style::default().bg(BLUE)
            } else {
                Style::default().fg(TEXT)
            };

            ListItem::new(line).style(item_style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Config Variables ")
                .title_style(Style::default().fg(MAUVE).add_modifier(Modifier::BOLD))
                .border_style(Style::default().fg(SURFACE1)),
        );

    let mut state = ListState::default();
    state.select(Some(app.selected));
    f.render_stateful_widget(list, chunks[1], &mut state);

    // Input Area
    let current_var = app.vars.get(app.selected);
    let input_title = if let Some(var) = current_var {
        if var.default_value.is_empty() {
            format!(" Editing: {} ", var.key)
        } else {
            format!(" Editing: {} (Default: {}) ", var.key, var.default_value)
        }
    } else {
        " Input ".to_string()
    };

    let input_border_color = match app.mode {
        Mode::Insert => GREEN,
        Mode::Normal => SURFACE1,
    };

    let input_text = app.input.value();
    let cursor_pos = app.input.visual_cursor();

    let input = Paragraph::new(input_text)
        .style(Style::default().fg(TEXT))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(input_title)
                .title_style(Style::default().fg(PEACH).add_modifier(Modifier::BOLD))
                .border_style(Style::default().fg(input_border_color)),
        );
    f.render_widget(input, chunks[2]);

    if let Mode::Insert = app.mode {
        f.set_cursor_position(ratatui::layout::Position::new(
            chunks[2].x + 1 + cursor_pos as u16,
            chunks[2].y + 1,
        ));
    }

    // Status bar (modern pill style at the bottom)
    let (mode_str, mode_style) = match app.mode {
        Mode::Normal => (
            " NORMAL ",
            Style::default()
                .bg(BLUE)
                .fg(CRUST)
                .add_modifier(Modifier::BOLD),
        ),
        Mode::Insert => (
            " INSERT ",
            Style::default()
                .bg(GREEN)
                .fg(CRUST)
                .add_modifier(Modifier::BOLD),
        ),
    };

    let status_msg = app.status_message.as_deref().unwrap_or_else(|| {
        match app.mode {
            Mode::Normal => " navigation | i: edit | :w: save | :q: quit ",
            Mode::Insert => " Esc: back to normal | Enter: commit ",
        }
    });

    let status_line = Line::from(vec![
        Span::styled(mode_str, mode_style),
        Span::styled(format!(" {} ", status_msg), Style::default().bg(SURFACE0).fg(TEXT)),
    ]);

    let status = Paragraph::new(status_line).style(Style::default().bg(SURFACE0));
    f.render_widget(status, chunks[4]);
}
