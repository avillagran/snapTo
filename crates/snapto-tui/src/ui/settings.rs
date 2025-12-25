use crate::app::{get_section_fields, get_uploader_fields, App, FieldType, SettingsSection};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};
use snapto_core::{ClipboardCopyMode, HistoryMode};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Help
        ])
        .split(area);

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(20), // Sections
            Constraint::Min(0),     // Settings
        ])
        .margin(1)
        .split(chunks[0]);

    draw_sections(f, app, main_chunks[0]);
    draw_settings_content(f, app, main_chunks[1]);
    draw_help(f, app, chunks[1]);
}

fn draw_sections(f: &mut Frame, app: &App, area: Rect) {
    let sections = vec![
        SettingsSection::General,
        SettingsSection::Naming,
        SettingsSection::History,
        SettingsSection::Uploads,
        SettingsSection::Security,
    ];

    let items: Vec<ListItem> = sections
        .iter()
        .map(|section| {
            let style = if *section == app.settings_section {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(section.name()).style(style)
        })
        .collect();

    let block = Block::default()
        .title(" Sections ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn draw_settings_content(f: &mut Frame, app: &App, area: Rect) {
    match app.settings_section {
        SettingsSection::General => draw_editable_settings(f, app, area, "General Settings"),
        SettingsSection::Naming => draw_editable_settings(f, app, area, "Naming Settings"),
        SettingsSection::History => draw_editable_settings(f, app, area, "History Settings"),
        SettingsSection::Uploads => draw_uploads_settings(f, app, area),
        SettingsSection::Security => draw_editable_settings(f, app, area, "Security Settings"),
    }
}

fn draw_editable_settings(f: &mut Frame, app: &App, area: Rect, title: &str) {
    let fields = get_section_fields(app.settings_section);
    let mut lines: Vec<Line> = Vec::new();

    for (i, field) in fields.iter().enumerate() {
        let is_selected = i == app.settings_selected;
        let is_editing = is_selected && app.settings_editing;

        // Get current value
        let value = get_display_value(app, field.name);

        // Build the line
        let prefix = if is_selected { "▶ " } else { "  " };

        let label_style = if is_selected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let value_style = match field.field_type {
            FieldType::Bool => {
                let is_true = value == "Yes";
                if is_selected {
                    Style::default()
                        .fg(if is_true { Color::Green } else { Color::Red })
                        .add_modifier(Modifier::BOLD)
                        .bg(Color::DarkGray)
                } else {
                    Style::default().fg(if is_true { Color::Green } else { Color::Red })
                }
            }
            FieldType::Enum => {
                if is_selected {
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD)
                        .bg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::Magenta)
                }
            }
            _ => {
                if is_selected {
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                        .bg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::White)
                }
            }
        };

        if is_editing {
            // Show edit mode with cursor
            let cursor_pos = app.edit_cursor;
            let buffer = &app.edit_buffer;

            let before_cursor = buffer[..cursor_pos.min(buffer.len())].to_string();
            let cursor_char = buffer.chars().nth(cursor_pos).map(|c| c.to_string()).unwrap_or_else(|| " ".to_string());
            let after_cursor = if cursor_pos < buffer.len() { buffer[cursor_pos + 1..].to_string() } else { String::new() };

            lines.push(Line::from(vec![
                Span::styled(prefix, label_style),
                Span::styled(format!("{}: ", field.label), label_style),
                Span::styled(before_cursor, Style::default().fg(Color::White).bg(Color::DarkGray)),
                Span::styled(cursor_char, Style::default().fg(Color::Black).bg(Color::White)),
                Span::styled(after_cursor, Style::default().fg(Color::White).bg(Color::DarkGray)),
            ]));
        } else {
            // Normal display
            let type_hint = match field.field_type {
                FieldType::Bool => " [Space to toggle]",
                FieldType::Enum => " [Space to cycle]",
                FieldType::Text | FieldType::Number => " [Enter to edit]",
                FieldType::Password => " [Enter to set]",
            };

            lines.push(Line::from(vec![
                Span::styled(prefix, label_style),
                Span::styled(format!("{}: ", field.label), label_style),
                Span::styled(value.clone(), value_style),
                if is_selected {
                    Span::styled(type_hint, Style::default().fg(Color::DarkGray))
                } else {
                    Span::raw("")
                },
            ]));
        }

        // Add spacing between fields
        lines.push(Line::from(""));
    }

    if fields.is_empty() {
        lines.push(Line::from(Span::styled(
            "No editable fields in this section",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

fn get_display_value(app: &App, field_name: &str) -> String {
    match app.settings_section {
        SettingsSection::General => match field_name {
            "local_save_dir" => app.config.general.local_save_dir.clone().unwrap_or_else(|| "Not set".to_string()),
            "copy_url_to_clipboard" => if app.config.general.copy_url_to_clipboard { "Yes" } else { "No" }.to_string(),
            "clipboard_copy_mode" => match app.config.general.clipboard_copy_mode {
                ClipboardCopyMode::Auto => "auto",
                ClipboardCopyMode::Url => "url",
                ClipboardCopyMode::Path => "path",
            }.to_string(),
            "show_notifications" => if app.config.general.show_notifications { "Yes" } else { "No" }.to_string(),
            "default_uploader" => app.config.general.default_uploader.clone(),
            _ => String::new(),
        },
        SettingsSection::Naming => match field_name {
            "template" => app.config.naming.template.clone(),
            "date_format" => app.config.naming.date_format.clone(),
            "time_format" => app.config.naming.time_format.clone(),
            "default_extension" => app.config.naming.default_extension.clone(),
            _ => String::new(),
        },
        SettingsSection::History => match field_name {
            "enabled" => if app.config.history.enabled { "Yes" } else { "No" }.to_string(),
            "mode" => match app.config.history.mode {
                HistoryMode::Metadata => "metadata",
                HistoryMode::Thumbnails => "thumbnails",
                HistoryMode::Full => "full",
            }.to_string(),
            "retention_days" => {
                if app.config.history.retention_days == 0 {
                    "Forever".to_string()
                } else {
                    format!("{} days", app.config.history.retention_days)
                }
            }
            "max_entries" => app.config.history.max_entries.to_string(),
            _ => String::new(),
        },
        SettingsSection::Security => match field_name {
            "use_system_keychain" => if app.config.security.use_system_keychain { "Yes" } else { "No" }.to_string(),
            "encrypt_credentials" => if app.config.security.encrypt_credentials { "Yes" } else { "No" }.to_string(),
            _ => String::new(),
        },
        _ => String::new(),
    }
}

fn draw_uploads_settings(f: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();

    if app.uploader_names.is_empty() {
        lines.push(Line::from(Span::styled(
            "No upload destinations configured.",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Press 'a' to add a new destination.",
            Style::default().fg(Color::Yellow),
        )));
    } else {
        for (uploader_idx, name) in app.uploader_names.iter().enumerate() {
            let is_selected_uploader = uploader_idx == app.uploader_selected;
            let Some(upload) = app.config.uploads.get(name) else { continue };

            // Uploader header
            let header_style = if is_selected_uploader {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            };

            lines.push(Line::from(vec![
                Span::styled(
                    if upload.enabled { "✓ " } else { "✗ " },
                    Style::default().fg(if upload.enabled { Color::Green } else { Color::Red }),
                ),
                Span::styled(name.clone(), header_style),
                Span::styled(format!(" [{}]", upload.uploader_type), Style::default().fg(Color::Cyan)),
            ]));

            // Show fields for this uploader
            let fields = get_uploader_fields(&upload.uploader_type);
            for (field_idx, field) in fields.iter().enumerate() {
                let is_selected = is_selected_uploader && field_idx == app.uploader_field_selected;
                let is_editing = is_selected && app.uploader_editing;

                let value = get_uploader_display_value(upload, field.name, name, app);
                let prefix = if is_selected { "  ▶ " } else { "    " };

                let label_style = if is_selected {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::DarkGray)
                };

                let value_style = match field.field_type {
                    FieldType::Bool => {
                        let is_true = value == "Yes";
                        if is_selected {
                            Style::default()
                                .fg(if is_true { Color::Green } else { Color::Red })
                                .add_modifier(Modifier::BOLD)
                                .bg(Color::DarkGray)
                        } else {
                            Style::default().fg(if is_true { Color::Green } else { Color::Red })
                        }
                    }
                    FieldType::Enum => {
                        if is_selected {
                            Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD).bg(Color::DarkGray)
                        } else {
                            Style::default().fg(Color::Magenta)
                        }
                    }
                    FieldType::Password => {
                        if is_selected {
                            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD).bg(Color::DarkGray)
                        } else {
                            Style::default().fg(Color::Yellow)
                        }
                    }
                    _ => {
                        if is_selected {
                            Style::default().fg(Color::White).add_modifier(Modifier::BOLD).bg(Color::DarkGray)
                        } else {
                            Style::default().fg(Color::White)
                        }
                    }
                };

                if is_editing {
                    let cursor_pos = app.edit_cursor;
                    let buffer = &app.edit_buffer;

                    // For password fields, display asterisks instead of actual characters
                    let (display_before, display_cursor, display_after) = if field.field_type == FieldType::Password {
                        let before = "*".repeat(cursor_pos.min(buffer.len()));
                        let cursor = if cursor_pos < buffer.len() { "*".to_string() } else { " ".to_string() };
                        let after = if cursor_pos < buffer.len() { "*".repeat(buffer.len() - cursor_pos - 1) } else { String::new() };
                        (before, cursor, after)
                    } else {
                        let before = buffer[..cursor_pos.min(buffer.len())].to_string();
                        let cursor = buffer.chars().nth(cursor_pos).map(|c| c.to_string()).unwrap_or_else(|| " ".to_string());
                        let after = if cursor_pos < buffer.len() { buffer[cursor_pos + 1..].to_string() } else { String::new() };
                        (before, cursor, after)
                    };

                    lines.push(Line::from(vec![
                        Span::styled(prefix, label_style),
                        Span::styled(format!("{}: ", field.label), label_style),
                        Span::styled(display_before, Style::default().fg(Color::White).bg(Color::DarkGray)),
                        Span::styled(display_cursor, Style::default().fg(Color::Black).bg(Color::White)),
                        Span::styled(display_after, Style::default().fg(Color::White).bg(Color::DarkGray)),
                    ]));
                } else {
                    let type_hint = if is_selected {
                        match field.field_type {
                            FieldType::Bool => " [Space]",
                            FieldType::Enum => " [Space]",
                            FieldType::Text | FieldType::Number => " [Enter]",
                            FieldType::Password => " [Enter to set]",
                        }
                    } else {
                        ""
                    };

                    lines.push(Line::from(vec![
                        Span::styled(prefix, label_style),
                        Span::styled(format!("{}: ", field.label), label_style),
                        Span::styled(value, value_style),
                        Span::styled(type_hint, Style::default().fg(Color::DarkGray)),
                    ]));
                }
            }
            lines.push(Line::from(""));
        }
    }

    let block = Block::default()
        .title(" Upload Destinations ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);

    // Draw add uploader popup if active
    if app.show_add_uploader {
        draw_add_uploader_popup(f, app, area);
    }
}

fn get_uploader_display_value(upload: &snapto_core::UploadConfig, field_name: &str, uploader_name: &str, app: &App) -> String {
    match field_name {
        "enabled" => if upload.enabled { "Yes" } else { "No" }.to_string(),
        "type" => upload.uploader_type.clone(),
        "host" => upload.host.clone().unwrap_or_else(|| "Not set".to_string()),
        "port" => upload.port.map(|p| p.to_string()).unwrap_or_else(|| "22".to_string()),
        "username" => upload.username.clone().unwrap_or_else(|| "Not set".to_string()),
        "remote_path" => upload.remote_path.clone().unwrap_or_else(|| "Not set".to_string()),
        "base_url" => upload.base_url.clone().unwrap_or_else(|| "Not set".to_string()),
        "local_path" => upload.local_path.clone().unwrap_or_else(|| "Not set".to_string()),
        "use_key_auth" => if upload.use_key_auth.unwrap_or(true) { "Yes" } else { "No" }.to_string(),
        "key_path" => upload.key_path.clone().unwrap_or_else(|| "~/.ssh/id_rsa".to_string()),
        "timeout" => upload.timeout.map(|t| format!("{}s", t)).unwrap_or_else(|| "30s".to_string()),
        "password" => {
            // Check if password is stored in keychain
            let keychain_key = format!("ssh_password_{}", uploader_name);
            if let Some(ref km) = app.keychain_manager {
                match km.get(&keychain_key) {
                    Ok(Some(_)) => return "••••••••".to_string(),
                    Ok(None) => return "(not set)".to_string(),
                    Err(e) => return format!("(error: {})", e),
                }
            }
            "(no keychain)".to_string()
        }
        _ => String::new(),
    }
}

fn draw_add_uploader_popup(f: &mut Frame, app: &App, area: Rect) {
    let popup_width = 50;
    let popup_height = 10;
    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    f.render_widget(Clear, popup_area);

    let type_names = ["local", "sftp", "ssh"];
    let selected_type = type_names[app.new_uploader_type];

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Name: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&app.new_uploader_name, Style::default().fg(Color::White)),
            Span::styled("_", Style::default().fg(Color::White).add_modifier(Modifier::SLOW_BLINK)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Type: ", Style::default().fg(Color::DarkGray)),
            Span::styled(selected_type, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" (Tab to change)", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(": Create  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(": Cancel", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let block = Block::default()
        .title(" Add New Uploader ")
        .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, popup_area);
}

fn draw_help(f: &mut Frame, app: &App, area: Rect) {
    let help_text = if app.settings_editing || app.uploader_editing {
        Line::from(vec![
            Span::styled("Type", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(": Edit value  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(": Save  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(": Cancel", Style::default().fg(Color::DarkGray)),
        ])
    } else if app.settings_section == SettingsSection::Uploads {
        Line::from(vec![
            Span::styled("j/k", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(": Navigate  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Space/Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(": Edit  ", Style::default().fg(Color::DarkGray)),
            Span::styled("a", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(": Add  ", Style::default().fg(Color::DarkGray)),
            Span::styled("d", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(": Delete  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Ctrl+S", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::styled(": Save", Style::default().fg(Color::DarkGray)),
        ])
    } else {
        Line::from(vec![
            Span::styled("h/l", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(": Sections  ", Style::default().fg(Color::DarkGray)),
            Span::styled("j/k", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(": Navigate  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Enter/Space", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(": Edit  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Ctrl+S", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(": Save to file", Style::default().fg(Color::DarkGray)),
        ])
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new(vec![help_text]).block(block);
    f.render_widget(paragraph, area);
}
