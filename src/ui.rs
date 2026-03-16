use crate::app::{App, Mode};
use crate::config::Config;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

// Catppuccin Mocha Palette
const MANTLE: Color = Color::Rgb(24, 24, 37);
const BASE: Color = Color::Rgb(30, 30, 46);
const TEXT: Color = Color::Rgb(205, 214, 244);
const BLUE: Color = Color::Rgb(137, 180, 250);
const GREEN: Color = Color::Rgb(166, 227, 161);
const SURFACE1: Color = Color::Rgb(69, 71, 90);

pub fn draw(f: &mut Frame, app: &mut App, _config: &Config) {
    let size = f.area();

    // Theming
    let bg_color = BASE;
    let fg_color = TEXT;
    let highlight_color = BLUE;
    let insert_color = GREEN;

    // Background
    let block = Block::default().style(Style::default().bg(bg_color).fg(fg_color));
    f.render_widget(block, size);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),    // List
            Constraint::Length(3), // Input area
            Constraint::Length(1), // Status bar
        ])
        .split(size);

    // List
    let items: Vec<ListItem> = app
        .vars
        .iter()
        .enumerate()
        .map(|(i, var)| {
            let style = if i == app.selected {
                Style::default()
                    .fg(bg_color)
                    .bg(highlight_color)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(fg_color)
            };

            let val = if i == app.selected && matches!(app.mode, Mode::Insert) {
                app.input.value()
            } else {
                &var.value
            };

            let content = format!(" {} = {} ", var.key, val);
            ListItem::new(Line::from(content)).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Environment Variables ")
                .border_style(Style::default().fg(SURFACE1)),
        )
        .highlight_style(
            Style::default()
                .fg(bg_color)
                .bg(highlight_color)
                .add_modifier(Modifier::BOLD),
        );

    let mut state = ListState::default();
    state.select(Some(app.selected));
    f.render_stateful_widget(list, chunks[0], &mut state);

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

    let input_color = match app.mode {
        Mode::Insert => insert_color,
        Mode::Normal => SURFACE1,
    };

    let input_text = app.input.value();
    let cursor_pos = app.input.visual_cursor();

    let input = Paragraph::new(input_text)
        .style(Style::default().fg(fg_color))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(input_title)
                .border_style(Style::default().fg(input_color)),
        );
    f.render_widget(input, chunks[1]);

    if let Mode::Insert = app.mode {
        f.set_cursor_position(ratatui::layout::Position::new(
            chunks[1].x + 1 + cursor_pos as u16,
            chunks[1].y + 1,
        ));
    }

    // Status bar
    let status_style = Style::default().bg(MANTLE).fg(fg_color);
    let (mode_str, mode_style) = match app.mode {
        Mode::Normal => (
            " NORMAL ",
            Style::default()
                .bg(BLUE)
                .fg(bg_color)
                .add_modifier(Modifier::BOLD),
        ),
        Mode::Insert => (
            " INSERT ",
            Style::default()
                .bg(GREEN)
                .fg(bg_color)
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
        Span::styled(format!(" {} ", status_msg), status_style),
    ]);

    let status = Paragraph::new(status_line).style(status_style);
    f.render_widget(status, chunks[2]);
}
