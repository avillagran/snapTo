use crate::app::App;
use chrono::Local;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),  // Clipboard status
            Constraint::Length(9),  // Destinations
            Constraint::Min(0),     // Last upload
        ])
        .margin(1)
        .split(area);

    draw_clipboard_status(f, app, chunks[0]);
    draw_destinations(f, app, chunks[1]);
    draw_last_upload(f, app, chunks[2]);
}

fn draw_clipboard_status(f: &mut Frame, app: &App, area: Rect) {
    let status = if app.clipboard_has_image {
        ("Ready", Color::Green)
    } else {
        ("No Image", Color::Yellow)
    };

    let text = vec![
        Line::from(vec![
            Span::raw("Clipboard Status: "),
            Span::styled(status.0, Style::default().fg(status.1).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::DarkGray)),
            Span::styled("u", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" to upload manually", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::DarkGray)),
            Span::styled("r", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" to refresh status", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let block = Block::default()
        .title(" Clipboard ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, area);
}

fn draw_destinations(f: &mut Frame, app: &App, area: Rect) {
    let destinations: Vec<ListItem> = app
        .config
        .uploads
        .iter()
        .map(|(name, upload)| {
            let status_icon = if upload.enabled { "✓" } else { "✗" };
            let status_color = if upload.enabled {
                Color::Green
            } else {
                Color::Red
            };
            let is_default = app.config.general.default_uploader == *name;
            let default_marker = if is_default { " (default)" } else { "" };

            let content = vec![
                Span::styled(format!("{} ", status_icon), Style::default().fg(status_color)),
                Span::styled(name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::styled(default_marker, Style::default().fg(Color::Yellow)),
                Span::raw(" - "),
                Span::styled(&upload.uploader_type, Style::default().fg(Color::Cyan)),
            ];

            ListItem::new(Line::from(content))
        })
        .collect();

    let block = Block::default()
        .title(" Destinations ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let list = List::new(destinations).block(block);
    f.render_widget(list, area);
}

fn draw_last_upload(f: &mut Frame, app: &App, area: Rect) {
    let content = if let Some(entry) = app.get_last_upload() {
        let local_time = entry.created_at.with_timezone(&Local);
        vec![
            Line::from(vec![
                Span::styled("File: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&entry.filename, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Destination: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&entry.destination, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("Size: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format_size(entry.size),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled("Time: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    local_time.format("%Y-%m-%d %H:%M:%S").to_string(),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("URL: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    entry.url.as_deref().unwrap_or("N/A"),
                    Style::default().fg(Color::Green),
                ),
            ]),
        ]
    } else {
        vec![
            Line::from(""),
            Line::from(Span::styled(
                "No uploads yet",
                Style::default().fg(Color::DarkGray),
            )),
        ]
    };

    let block = Block::default()
        .title(" Last Upload ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(content).block(block);
    f.render_widget(paragraph, area);
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
