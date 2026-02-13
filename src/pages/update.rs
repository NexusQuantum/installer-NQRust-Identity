use chrono::{DateTime, Utc};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
};

use crate::app::UpdateInfo;
use crate::ui::{get_orange_accent, get_orange_color};

pub struct UpdateListView<'a> {
    pub updates: &'a [UpdateInfo],
    pub selected_index: usize,
    pub message: Option<&'a str>,
    pub logs: &'a [String],
    pub pulling: bool,
    pub progress: Option<f64>,
}

pub fn render_update_list(frame: &mut Frame, view: &UpdateListView<'_>) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
            Constraint::Min(6),
        ])
        .split(area);

    let title_text = if view.pulling {
        "🔄 Pulling selected image..."
    } else {
        "🚀 Check for Updates"
    };

    let title = Paragraph::new(title_text)
        .style(
            Style::default()
                .fg(get_orange_color())
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(get_orange_accent())),
        )
        .centered();
    frame.render_widget(title, chunks[0]);

    if view.updates.is_empty() {
        let placeholder = Paragraph::new("No GHCR-backed services found")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(get_orange_accent()))
                    .title("Services")
                    .title_style(
                        Style::default()
                            .fg(get_orange_color())
                            .add_modifier(Modifier::BOLD),
                    ),
            )
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(placeholder, chunks[1]);
    } else {
        let header = Row::new(vec![
            Cell::from("Service").style(header_style()),
            Cell::from("Current Tag").style(header_style()),
            Cell::from("Latest Release").style(header_style()),
            Cell::from("Remote Updated").style(header_style()),
            Cell::from("Local Image").style(header_style()),
            Cell::from("Status").style(header_style()),
        ]);

        let rows: Vec<Row> = view
            .updates
            .iter()
            .enumerate()
            .map(|(idx, info)| {
                let mut style = if info.status_note.is_some() {
                    Style::default().fg(Color::Red)
                } else if info.has_update {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::Green)
                };

                if idx == view.selected_index && !view.pulling {
                    style = style.add_modifier(Modifier::REVERSED | Modifier::BOLD);
                }

                Row::new(vec![
                    Cell::from(info.display_name.clone()),
                    Cell::from(info.current_tag.clone()),
                    Cell::from(
                        info.latest_release_tag
                            .clone()
                            .unwrap_or_else(|| "—".to_string()),
                    ),
                    Cell::from(format_time(info.remote_latest_updated)),
                    Cell::from(format_time(info.local_created)),
                    Cell::from(status_text(info)),
                ])
                .style(style)
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Length(24),
                Constraint::Length(12),
                Constraint::Length(18),
                Constraint::Length(18),
                Constraint::Length(18),
                Constraint::Min(12),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(get_orange_accent()))
                .title("Services")
                .title_style(
                    Style::default()
                        .fg(get_orange_color())
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .column_spacing(2);

        frame.render_widget(table, chunks[1]);
    }

    let message_text = view
        .message
        .unwrap_or("Enter/P: pull image or self-update installer | R: refresh | Esc: back");

    let message = Paragraph::new(message_text)
        .style(Style::default().fg(Color::Gray))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(get_orange_accent()))
                .title("Instructions")
                .title_style(
                    Style::default()
                        .fg(get_orange_color())
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(message, chunks[2]);

    let mut log_lines: Vec<Line> = if view.logs.is_empty() {
        vec![Line::from(Span::styled(
            "No recent docker operations",
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        view.logs
            .iter()
            .map(|entry| {
                let style = if entry.contains("❌") {
                    Style::default().fg(Color::Red)
                } else if entry.contains("✅") {
                    Style::default().fg(Color::Green)
                } else if entry.contains("⚠️") {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                };

                Line::from(Span::styled(entry.clone(), style))
            })
            .collect()
    };

    // Prepend a simple progress bar when pulling and a value is provided.
    if view.pulling {
        if let Some(pct) = view.progress {
            let pct = pct.clamp(0.0, 100.0);
            let bar_space = chunks[3].width.saturating_sub(12) as usize;
            let filled_width = ((bar_space as f64) * (pct / 100.0)).round() as usize;
            let filled = "█".repeat(filled_width.min(bar_space));
            let empty = "░".repeat(bar_space.saturating_sub(filled.len()));
            let bar = format!("Progress: [{filled}{empty}] {pct:.0}%");
            log_lines.insert(
                0,
                Line::from(Span::styled(bar, Style::default().fg(get_orange_color()))),
            );
        }
    }

    let logs_widget = Paragraph::new(log_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(get_orange_accent()))
                .title("Logs")
                .title_style(
                    Style::default()
                        .fg(get_orange_color())
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .wrap(Wrap { trim: false })
        .scroll((
            view.logs
                .len()
                .saturating_sub(chunks[3].height as usize - 2) as u16,
            0,
        ));
    frame.render_widget(logs_widget, chunks[3]);
}

fn header_style() -> Style {
    Style::default()
        .fg(get_orange_color())
        .add_modifier(Modifier::BOLD)
}

fn status_text(info: &UpdateInfo) -> String {
    if let Some(note) = &info.status_note {
        note.clone()
    } else if info.has_update {
        "Update available".to_string()
    } else {
        "Up to date".to_string()
    }
}

fn format_time(value: Option<DateTime<Utc>>) -> String {
    value
        .map(|dt| dt.format("%Y-%m-%d %H:%M UTC").to_string())
        .unwrap_or_else(|| "—".to_string())
}
