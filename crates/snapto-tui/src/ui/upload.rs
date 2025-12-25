use crate::app::{App, UploadStatus};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(5),  // Progress bar
            Constraint::Length(7),  // Destination info
            Constraint::Min(0),     // Result
        ])
        .margin(2)
        .split(area);

    draw_title(f, chunks[0]);
    draw_progress(f, app, chunks[1]);
    draw_destination(f, app, chunks[2]);
    draw_result(f, app, chunks[3]);
}

fn draw_title(f: &mut Frame, area: Rect) {
    let title = Paragraph::new("Upload in Progress")
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default());

    f.render_widget(title, area);
}

fn draw_progress(f: &mut Frame, app: &App, area: Rect) {
    let progress = app.upload_progress.unwrap_or(0.0);
    let label = format!("{:.0}%", progress);

    let gauge = Gauge::default()
        .block(
            Block::default()
                .title(" Progress ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .gauge_style(
            Style::default()
                .fg(Color::Green)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .label(label)
        .ratio(progress / 100.0);

    f.render_widget(gauge, area);
}

fn draw_destination(f: &mut Frame, app: &App, area: Rect) {
    let destination = app
        .config
        .uploads
        .get(&app.config.general.default_uploader)
        .map(|d| d.uploader_type.as_str())
        .unwrap_or("Unknown");

    let content = vec![
        Line::from(vec![
            Span::styled("Destination: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                &app.config.general.default_uploader,
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Type: ", Style::default().fg(Color::DarkGray)),
            Span::styled(destination, Style::default().fg(Color::White)),
        ]),
    ];

    let block = Block::default()
        .title(" Upload Info ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(content).block(block);
    f.render_widget(paragraph, area);
}

fn draw_result(f: &mut Frame, app: &App, area: Rect) {
    let (title, content, border_color) = match &app.upload_result {
        Some(UploadStatus::InProgress) => (
            " Status ",
            vec![
                Line::from(""),
                Line::from(Span::styled(
                    "Uploading...",
                    Style::default().fg(Color::Yellow),
                )),
            ],
            Color::Yellow,
        ),
        Some(UploadStatus::Success { url }) => (
            " Success ",
            vec![
                Line::from(""),
                Line::from(Span::styled(
                    "Upload completed successfully!",
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled("URL: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(url, Style::default().fg(Color::Green)),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "The URL has been copied to your clipboard.",
                    Style::default().fg(Color::DarkGray),
                )),
            ],
            Color::Green,
        ),
        Some(UploadStatus::Error { message }) => (
            " Error ",
            vec![
                Line::from(""),
                Line::from(Span::styled(
                    "Upload failed!",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Error: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(message, Style::default().fg(Color::Red)),
                ]),
            ],
            Color::Red,
        ),
        None => (
            " Status ",
            vec![
                Line::from(""),
                Line::from(Span::styled(
                    "Preparing upload...",
                    Style::default().fg(Color::DarkGray),
                )),
            ],
            Color::DarkGray,
        ),
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let paragraph = Paragraph::new(content).block(block);
    f.render_widget(paragraph, area);
}
