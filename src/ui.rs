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
            Block::default().style(Style::default().bg(theme.bg_normal())),
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
    let matching_indices = app.matching_indices();
    let items: Vec<ListItem> = app
        .vars
        .iter()
        .enumerate()
        .map(|(i, var)| {
            let is_selected = i == app.selected;
            let is_match = matching_indices.contains(&i);

            // Indentation based on depth
            let indent = "  ".repeat(var.depth);
            let prefix = if var.is_group { "+ " } else { "  " };

            // Determine colors based on depth
            let depth_color = if is_selected {
                theme.bg_normal()
            } else {
                match var.depth % 4 {
                    0 => theme.tree_depth_1(),
                    1 => theme.tree_depth_2(),
                    2 => theme.tree_depth_3(),
                    3 => theme.tree_depth_4(),
                    _ => theme.fg_normal(),
                }
            };

            // Determine colors based on status and selection
            let text_color = if is_selected {
                theme.fg_highlight()
            } else {
                match var.status {
                    crate::format::ItemStatus::MissingFromActive if !var.is_group => theme.fg_dimmed(),
                    crate::format::ItemStatus::Modified => theme.fg_modified(),
                    _ => theme.fg_normal(),
                }
            };

            let key_style = if is_selected {
                Style::default()
                    .fg(theme.fg_highlight())
                    .add_modifier(Modifier::BOLD)
            } else if is_match {
                Style::default()
                    .fg(theme.bg_search())
                    .add_modifier(Modifier::UNDERLINED)
            } else if var.status == crate::format::ItemStatus::MissingFromActive && !var.is_group {
                Style::default()
                    .fg(theme.fg_dimmed())
                    .add_modifier(Modifier::DIM)
            } else {
                Style::default().fg(depth_color)
            };

            let mut key_spans = vec![
                Span::raw(indent),
                Span::styled(prefix, Style::default().fg(theme.border_normal())),
                Span::styled(&var.key, key_style),
            ];

            // Add status indicator if not present
            match var.status {
                crate::format::ItemStatus::MissingFromActive if !var.is_group => {
                    let missing_style = if is_selected {
                        Style::default().fg(theme.fg_highlight()).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.fg_warning()).add_modifier(Modifier::BOLD)
                    };
                    key_spans.push(Span::styled(" (missing)", missing_style));
                }
                crate::format::ItemStatus::MissingFromActive if var.is_group => {
                    let missing_style = if is_selected {
                        Style::default().fg(theme.fg_highlight()).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.fg_warning()).add_modifier(Modifier::BOLD)
                    };
                    key_spans.push(Span::styled(" (missing group)", missing_style));
                }
                crate::format::ItemStatus::Modified => {
                    if !is_selected {
                        key_spans.push(Span::styled(" (*)", Style::default().fg(theme.fg_modified())));
                    }
                }
                _ => {}
            }

            let item_style = if is_selected {
                Style::default().bg(theme.bg_highlight())
            } else {
                Style::default().fg(text_color)
            };

            if var.is_group {
                ListItem::new(Line::from(key_spans)).style(item_style)
            } else {
                // Show live input text for the selected item if in Insert mode.
                let val = if is_selected && matches!(app.mode, Mode::Insert) {
                    app.input.value()
                } else {
                    var.value.as_deref().unwrap_or("")
                };

                let value_style = if is_selected {
                    Style::default().fg(theme.fg_highlight())
                } else {
                    Style::default().fg(theme.fg_normal())
                };

                let mut val_spans = vec![
                    Span::raw(format!("{}    └─ ", "  ".repeat(var.depth))),
                    Span::styled(val, value_style),
                ];

                if let Some(t_val) = &var.template_value
                    && Some(t_val) != var.value.as_ref() {
                        let t_style = if is_selected {
                            Style::default().fg(theme.bg_normal()).add_modifier(Modifier::DIM)
                        } else {
                            Style::default().fg(theme.fg_dimmed()).add_modifier(Modifier::ITALIC)
                        };
                        val_spans.push(Span::styled(format!(" [Def: {}]", t_val), t_style));
                    }

                ListItem::new(vec![Line::from(key_spans), Line::from(val_spans)]).style(item_style)
            }
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Config Variables ")
            .title_style(
                Style::default()
                    .fg(theme.fg_accent())
                    .add_modifier(Modifier::BOLD),
            )
            .border_style(Style::default().fg(theme.border_normal())),
    );

    let mut state = ListState::default();
    state.select(Some(app.selected));
    f.render_stateful_widget(list, chunks[1], &mut state);

    // Render the focused input area.
    let current_var = app.vars.get(app.selected);
    let mut input_title = " Input ".to_string();
    let mut extra_info = String::new();

    if let Some(var) = current_var {
        if matches!(app.mode, Mode::InsertKey) {
            input_title = format!(" Rename Key: {} ", var.path_string());
        } else if var.is_group {
            input_title = format!(" Group: {} ", var.path_string());
        } else {
            input_title = format!(" Editing: {} ", var.path_string());
            if let Some(t_val) = &var.template_value {
                extra_info = format!("  [Template: {}]", t_val);
            }
        }
    }

    let input_border_color = match app.mode {
        Mode::Insert | Mode::InsertKey => theme.border_active(),
        Mode::Normal | Mode::Search => theme.border_normal(),
    };

    let input_text = app.input.value();
    let cursor_pos = app.input.visual_cursor();

    // Show template value in normal mode if it differs
    let display_text = if let Some(var) = current_var {
        if matches!(app.mode, Mode::InsertKey) {
            input_text.to_string()
        } else if var.is_group {
            "<group>".to_string()
        } else if matches!(app.mode, Mode::Normal) {
            format!("{}{}", input_text, extra_info)
        } else {
            input_text.to_string()
        }
    } else {
        input_text.to_string()
    };

    let input = Paragraph::new(display_text)
        .style(Style::default().fg(theme.fg_normal()))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(input_title)
                .title_style(
                    Style::default()
                        .fg(theme.fg_accent()) // Make title pop
                        .add_modifier(Modifier::BOLD),
                )
                .border_style(Style::default().fg(input_border_color)),
        );
    f.render_widget(input, chunks[2]);

    // Position the terminal cursor correctly when in Insert mode.
    if matches!(app.mode, Mode::Insert) || matches!(app.mode, Mode::InsertKey) {
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
                .bg(theme.bg_highlight())
                .fg(theme.bg_normal())
                .add_modifier(Modifier::BOLD),
        ),
        Mode::Insert => (
            " INSERT ",
            Style::default()
                .bg(theme.bg_active())
                .fg(theme.bg_normal())
                .add_modifier(Modifier::BOLD),
        ),
        Mode::InsertKey => (
            " RENAME ",
            Style::default()
                .bg(theme.bg_active())
                .fg(theme.bg_normal())
                .add_modifier(Modifier::BOLD),
        ),
        Mode::Search => (
            " SEARCH ",
            Style::default()
                .bg(theme.bg_search())
                .fg(theme.bg_normal())
                .add_modifier(Modifier::BOLD),
        ),
    };

    let status_msg = if let Some(msg) = &app.status_message {
        msg.clone()
    } else {
        let kb = &config.keybinds;
        match app.mode {
            Mode::Normal => {
                let mut parts = vec![
                    format!("{}/{} move", kb.down, kb.up),
                    format!("{}/{} jump", kb.jump_top, kb.jump_bottom),
                    format!("{} search", kb.search),
                ];
                if !app.selected_is_group() {
                    parts.push(format!("{}/{}/{} edit", kb.edit, kb.edit_append, kb.edit_substitute));
                }
                parts.push(format!("{} rename", kb.rename));
                parts.push(format!("{} toggle", kb.toggle_group));
                if app.selected_is_missing() {
                    parts.push(format!("{} add", "a")); // 'a' is currently hardcoded in runner
                }
                if app.selected_is_array() {
                    parts.push(format!("{}/{} array", kb.append_item, kb.prepend_item));
                } else {
                    parts.push(format!("{}/{} add", kb.append_item, kb.prepend_item));
                    parts.push(format!("{}/{} group", kb.append_group, kb.prepend_group));
                }
                parts.push(format!("{} del", kb.delete_item));
                parts.push(format!("{} undo", kb.undo));
                parts.push(format!("{} save", kb.save));
                parts.push(format!("{} quit", kb.quit));
                parts.join(" · ")
            }
            Mode::Insert => "Esc cancel · Enter commit".to_string(),
            Mode::InsertKey => "Esc cancel · Enter rename".to_string(),
            Mode::Search => "Esc normal · type to filter".to_string(),
        }
    };

    let status_line = Line::from(vec![
        Span::styled(mode_str, mode_style),
        Span::styled(
            format!(" {} ", status_msg),
            Style::default().bg(theme.border_normal()).fg(theme.fg_normal()),
        ),
    ]);

    let status = Paragraph::new(status_line).style(Style::default().bg(theme.border_normal()));
    f.render_widget(status, chunks[4]);
}
