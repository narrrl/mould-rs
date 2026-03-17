use crate::app::{App, Mode};
use crate::config::Config;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
};

/// Renders the main application interface using ratatui.
pub fn draw(f: &mut Frame, app: &mut App, config: &Config) {
    let theme = &config.theme;
    let size = f.area();

    // Render the main background (optional based on transparency config).
    if !theme.transparent {
        f.render_widget(
            Block::default().style(Style::default().bg(theme.crust())),
            size,
        );
    }

    // Horizontal layout with 1-character side margins.
    let outer_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(size);

    // Vertical layout for the main UI components.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Top margin
            Constraint::Min(3),    // Main list
            Constraint::Length(3), // Focused input field
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Status bar
        ])
        .split(outer_layout[1]);

    // Build the interactive list of configuration variables.
    let items: Vec<ListItem> = app
        .vars
        .iter()
        .enumerate()
        .map(|(i, var)| {
            let is_selected = i == app.selected;

            // Show live input text for the selected item if in Insert mode.
            let val = if is_selected && matches!(app.mode, Mode::Insert) {
                app.input.value()
            } else {
                &var.value
            };

            let key_style = if is_selected {
                Style::default()
                    .fg(theme.crust())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.lavender())
            };

            let value_style = if is_selected {
                Style::default().fg(theme.crust())
            } else {
                Style::default().fg(theme.text())
            };

            // Path styling for nested keys (e.g., a.b.c)
            let mut key_spans = Vec::new();
            if let Some(last_dot) = var.key.rfind('.') {
                let path = &var.key[..=last_dot];
                let key = &var.key[last_dot + 1..];

                let path_style = if is_selected {
                    Style::default()
                        .fg(theme.crust())
                        .add_modifier(Modifier::DIM)
                } else {
                    Style::default().fg(theme.surface1())
                };

                key_spans.push(Span::styled(path, path_style));
                key_spans.push(Span::styled(key, key_style));
            } else {
                key_spans.push(Span::styled(&var.key, key_style));
            }

            let item_style = if is_selected {
                Style::default().bg(theme.blue())
            } else {
                Style::default().fg(theme.text())
            };

            // Two-line layout for better readability:
            // Line 1: Key (path.key)
            // Line 2: Value
            let lines = vec![
                Line::from(key_spans),
                Line::from(vec![
                    Span::styled(
                        "  └─ ",
                        if is_selected {
                            Style::default().fg(theme.crust())
                        } else {
                            Style::default().fg(theme.surface1())
                        },
                    ),
                    Span::styled(val, value_style),
                ]),
            ];

            ListItem::new(lines).style(item_style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Config Variables ")
            .title_style(
                Style::default()
                    .fg(theme.mauve())
                    .add_modifier(Modifier::BOLD),
            )
            .border_style(Style::default().fg(theme.surface1())),
    );

    let mut state = ListState::default();
    state.select(Some(app.selected));
    f.render_stateful_widget(list, chunks[1], &mut state);

    // Render the focused input area.
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
        Mode::Insert => theme.green(),
        Mode::Normal => theme.surface1(),
    };

    let input_text = app.input.value();
    let cursor_pos = app.input.visual_cursor();

    let input = Paragraph::new(input_text)
        .style(Style::default().fg(theme.text()))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(input_title)
                .title_style(
                    Style::default()
                        .fg(theme.peach())
                        .add_modifier(Modifier::BOLD),
                )
                .border_style(Style::default().fg(input_border_color)),
        );
    f.render_widget(input, chunks[2]);

    // Position the terminal cursor correctly when in Insert mode.
    if let Mode::Insert = app.mode {
        f.set_cursor_position(ratatui::layout::Position::new(
            chunks[2].x + 1 + cursor_pos as u16,
            chunks[2].y + 1,
        ));
    }

    // Render the modern pill-style status bar.
    let (mode_str, mode_style) = match app.mode {
        Mode::Normal => (
            " NORMAL ",
            Style::default()
                .bg(theme.blue())
                .fg(theme.crust())
                .add_modifier(Modifier::BOLD),
        ),
        Mode::Insert => (
            " INSERT ",
            Style::default()
                .bg(theme.green())
                .fg(theme.crust())
                .add_modifier(Modifier::BOLD),
        ),
    };

    let status_msg = app
        .status_message
        .as_deref()
        .unwrap_or_else(|| match app.mode {
            Mode::Normal => " navigation | i: edit | :w: save | :q: quit ",
            Mode::Insert => " Esc: back to normal | Enter: commit ",
        });

    let status_line = Line::from(vec![
        Span::styled(mode_str, mode_style),
        Span::styled(
            format!(" {} ", status_msg),
            Style::default().bg(theme.surface0()).fg(theme.text()),
        ),
    ]);

    let status = Paragraph::new(status_line).style(Style::default().bg(theme.surface0()));
    f.render_widget(status, chunks[4]);
}
