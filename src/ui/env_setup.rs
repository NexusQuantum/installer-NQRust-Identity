use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::FormData;
use crate::ui::{get_orange_accent, get_orange_color};

pub struct EnvSetupView<'a> {
    pub form_data: &'a FormData,
}

pub fn render_env_setup(frame: &mut Frame, view: &EnvSetupView<'_>) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(15),
            Constraint::Length(2),
        ])
        .split(area);

    let data = view.form_data;

    let title_text = if !data.selected_provider.is_empty() {
        format!("🔧 Generate .env File - {}", data.get_api_key_name())
    } else {
        "🔧 Generate .env File".to_string()
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

    let data = view.form_data;

    let mut form_lines = vec![
        Line::from(""),
        Line::from("Please provide your API key(s):"),
        Line::from(""),
    ];

    let needs_openai = data.needs_openai_embedding();

    // Field 0: Provider API Key
    let field0_style = if data.current_field == 0 {
        if data.editing {
            Style::default()
                .fg(get_orange_color())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(get_orange_color())
                .add_modifier(Modifier::BOLD)
        }
    } else {
        Style::default().fg(Color::White)
    };

    let api_key_name = data.get_api_key_name();
    let key_display = if data.api_key.is_empty() {
        "_".repeat(50)
    } else {
        let masked = if data.api_key.len() > 8 {
            format!(
                "{}...{}",
                &data.api_key[..4],
                &data.api_key[data.api_key.len() - 4..]
            )
        } else {
            "*".repeat(data.api_key.len())
        };
        format!("{}{}", masked, "_".repeat(50 - masked.len().min(50)))
    };

    form_lines.push(Line::from(vec![Span::styled(
        format!("{} API Key: ", api_key_name),
        field0_style,
    )]));
    form_lines.push(Line::from(""));
    form_lines.push(Line::from(vec![
        Span::styled(&key_display[..50.min(key_display.len())], field0_style),
        Span::styled(" *", Style::default().fg(Color::Red)),
    ]));
    form_lines.push(Line::from(""));

    // Field 1: OpenAI API Key (if needed for embedding)
    if needs_openai {
        let field1_style = if data.current_field == 1 {
            if data.editing {
                Style::default()
                    .fg(get_orange_color())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(get_orange_color())
                    .add_modifier(Modifier::BOLD)
            }
        } else {
            Style::default().fg(Color::White)
        };

        let openai_key_display: String = if data.openai_api_key.is_empty() {
            "_".repeat(50)
        } else {
            let masked = if data.openai_api_key.len() > 8 {
                format!(
                    "{}...{}",
                    &data.openai_api_key[..4],
                    &data.openai_api_key[data.openai_api_key.len() - 4..]
                )
            } else {
                "*".repeat(data.openai_api_key.len())
            };
            format!("{}{}", masked, "_".repeat(50 - masked.len().min(50)))
        };

        form_lines.push(Line::from(vec![Span::styled(
            "OpenAI API Key (for embedding): ",
            field1_style,
        )]));
        form_lines.push(Line::from(""));
        let openai_display_slice =
            openai_key_display[..50.min(openai_key_display.len())].to_string();
        form_lines.push(Line::from(vec![
            Span::styled(openai_display_slice, field1_style),
            Span::styled(" *", Style::default().fg(Color::Red)),
        ]));
        form_lines.push(Line::from(""));
        form_lines.push(Line::from(Span::styled(
            "ℹ️  This provider uses OpenAI embedding model, OpenAI API key is required",
            Style::default().fg(Color::Yellow),
        )));
        form_lines.push(Line::from(""));
    }

    if data.selected_provider == "lm_studio" || data.selected_provider == "ollama" {
        form_lines.push(Line::from(Span::styled(
            "ℹ️  No API key required for local services",
            Style::default().fg(Color::Yellow),
        )));
        form_lines.push(Line::from(""));
    }

    if !data.error_message.is_empty() {
        form_lines.push(Line::from(""));
        form_lines.push(Line::from(Span::styled(
            &data.error_message,
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )));
    }

    form_lines.push(Line::from(""));
    form_lines.push(Line::from(Span::styled(
        "* Required field",
        Style::default().fg(Color::DarkGray),
    )));

    let form = Paragraph::new(form_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(get_orange_accent()))
            .title("Configuration Form")
            .title_style(
                Style::default()
                    .fg(get_orange_color())
                    .add_modifier(Modifier::BOLD),
            ),
    );
    frame.render_widget(form, chunks[1]);

    let help_text = if data.editing {
        "Type to edit, Enter to finish, Esc to cancel"
    } else {
        "↑↓ to navigate, Enter to edit, Ctrl+S to save, Esc to cancel"
    };

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .centered();
    frame.render_widget(help, chunks[2]);
}
