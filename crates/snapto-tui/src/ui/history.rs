use crate::app::App;
use chrono::Local;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Table
            Constraint::Length(5), // Help
        ])
        .margin(1)
        .split(area);

    draw_history_table(f, app, chunks[0]);
    draw_help(f, app, chunks[1]);

    // Draw reupload menu popup if active
    if app.show_reupload_menu {
        draw_reupload_menu(f, app, area);
    }

    // Draw password prompt popup if active
    if app.show_password_prompt {
        draw_password_prompt(f, app, area);
    }
}

fn draw_history_table(f: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["Date", "Filename", "Destination", "Size", "URL"]
        .iter()
        .map(|h| {
            Cell::from(*h).style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
        });

    let header = Row::new(header_cells)
        .style(Style::default().bg(Color::DarkGray))
        .height(1);

    let rows: Vec<Row> = app
        .history
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let local_time = entry.created_at.with_timezone(&Local);
            let date = local_time.format("%Y-%m-%d %H:%M").to_string();
            let size = format_size(entry.size);
            let url = entry
                .url
                .as_ref()
                .map(|u| {
                    if u.len() > 40 {
                        format!("{}...", &u[..37])
                    } else {
                        u.clone()
                    }
                })
                .unwrap_or_else(|| "N/A".to_string());

            let style = if i == app.history_selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(date),
                Cell::from(entry.filename.clone()),
                Cell::from(entry.destination.clone()),
                Cell::from(size),
                Cell::from(url),
            ])
            .style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(16),
        Constraint::Min(20),
        Constraint::Length(15),
        Constraint::Length(10),
        Constraint::Min(20),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .title(format!(" Upload History ({} entries) ", app.history.len()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .column_spacing(1);

    f.render_widget(table, area);
}

fn draw_help(f: &mut Frame, app: &App, area: Rect) {
    let help_text = if app.show_password_prompt {
        vec![
            Line::from(vec![
                Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled(": Confirm  ", Style::default().fg(Color::DarkGray)),
                Span::styled("Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::styled(": Cancel  ", Style::default().fg(Color::DarkGray)),
                Span::styled("Password stored securely in keychain", Style::default().fg(Color::DarkGray)),
            ]),
        ]
    } else if app.show_reupload_menu {
        vec![
            Line::from(vec![
                Span::styled("j/k", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(": Navigate  ", Style::default().fg(Color::DarkGray)),
                Span::styled("Enter", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(": Confirm  ", Style::default().fg(Color::DarkGray)),
                Span::styled("Esc", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(": Cancel", Style::default().fg(Color::DarkGray)),
            ]),
        ]
    } else {
        vec![
            Line::from(vec![
                Span::styled("j/k", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(": Navigate  ", Style::default().fg(Color::DarkGray)),
                Span::styled("c", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(": Copy URL  ", Style::default().fg(Color::DarkGray)),
                Span::styled("r", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled(": Re-upload  ", Style::default().fg(Color::DarkGray)),
                Span::styled("d", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::styled(": Delete", Style::default().fg(Color::DarkGray)),
            ]),
        ]
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new(help_text).block(block);
    f.render_widget(paragraph, area);
}

fn draw_reupload_menu(f: &mut Frame, app: &App, area: Rect) {
    // Calculate popup size and position
    let popup_width = 40;
    let popup_height = (app.available_uploaders.len() + 4).min(15) as u16;
    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // Clear the background
    f.render_widget(Clear, popup_area);

    // Build list items
    let items: Vec<ListItem> = app
        .available_uploaders
        .iter()
        .enumerate()
        .map(|(i, (name, config))| {
            let style = if i == app.reupload_selected {
                Style::default()
                    .bg(Color::Cyan)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let prefix = if i == app.reupload_selected { "▶ " } else { "  " };
            let type_str = &config.uploader_type;
            ListItem::new(format!("{}{} [{}]", prefix, name, type_str)).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(" Select Destination ")
            .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(list, popup_area);
}

fn draw_password_prompt(f: &mut Frame, app: &App, area: Rect) {
    // Calculate popup size and position
    let popup_width = 50;
    let popup_height = 7;
    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // Clear the background
    f.render_widget(Clear, popup_area);

    // Get uploader name for title
    let uploader_name = app.pending_reupload
        .as_ref()
        .map(|p| p.uploader_name.as_str())
        .unwrap_or("SSH");

    // Create password display (masked)
    let password_display = "*".repeat(app.password_buffer.len());
    let cursor = if app.password_buffer.is_empty() { "_" } else { "" };

    let content = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Password: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{}{}", password_display, cursor),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Enter", Style::default().fg(Color::Green)),
            Span::styled(": Confirm  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::styled(": Cancel", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let block = Block::default()
        .title(format!(" Enter Password for {} ", uploader_name))
        .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    let paragraph = Paragraph::new(content).block(block);
    f.render_widget(paragraph, popup_area);
}

fn format_size(bytes: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = KB * 1024;
    const GB: usize = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
