use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph, Widget},
};

use crate::app::{App, MenuItem, Screen};

const TITLE: &str = r"
 __        _____ _     ____  ____
 \ \      / /_ _| |   |  _ \/ ___|
  \ \ /\ / / | || |   | | | \___ \
   \ V  V /  | || |___| |_| |___) |
    \_/\_/  |___|_____|____/|____/
";

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.screen {
            Screen::MainMenu => render_main_menu(self, area, buf),
            Screen::Options => render_options(self, area, buf),
            Screen::NewGame => render_placeholder("New Game", "Character creation coming soon...", area, buf),
            Screen::LoadGame => render_placeholder("Load Game", "Save file browser coming soon...", area, buf),
        }
    }
}

fn render_main_menu(app: &App, area: Rect, buf: &mut Buffer) {
    let outer = Block::bordered()
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(Color::DarkGray));
    let inner = outer.inner(area);
    outer.render(area, buf);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Length(1),
            Constraint::Min(4),
            Constraint::Length(2),
        ])
        .split(inner);

    Paragraph::new(TITLE)
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center)
        .render(chunks[0], buf);

    let menu_lines: Vec<Line> = MenuItem::ALL
        .iter()
        .enumerate()
        .map(|(i, item)| {
            if i == app.selected {
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        format!("> {} <", item.label()),
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("  "),
                ])
            } else {
                Line::from(Span::styled(item.label(), Style::default().fg(Color::White)))
            }
        })
        .collect();

    Paragraph::new(menu_lines)
        .alignment(Alignment::Center)
        .render(chunks[2], buf);

    Paragraph::new("↑ ↓ / j k  navigate    Enter  select    q  quit")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .render(chunks[3], buf);
}

fn render_options(app: &App, area: Rect, buf: &mut Buffer) {
    let block = Block::bordered()
        .title(" Options ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(Color::Yellow));
    let inner = block.inner(area);
    block.render(area, buf);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(2)])
        .split(inner);

    let s = &app.settings;
    let entries: [(&str, String); 6] = [
        ("Sound Effects", bool_label(s.sound_effects).to_string()),
        ("Music Volume", format!("{}%", s.music_volume)),
        ("Font Size", s.font_size.label().to_string()),
        ("Color Theme", s.color_theme.label().to_string()),
        ("Show Hints", bool_label(s.show_hints).to_string()),
        ("Difficulty", s.difficulty.label().to_string()),
    ];

    let lines: Vec<Line> = entries
        .iter()
        .enumerate()
        .map(|(i, (label, value))| {
            let selected = i == app.options_cursor;
            let row_style = if selected {
                Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let value_span = if selected {
                Span::styled(format!(" ◄ {value} ► "), row_style)
            } else {
                Span::styled(format!("   {value}   "), row_style)
            };

            Line::from(vec![
                Span::styled(format!("  {label:<20}", label = label), row_style),
                value_span,
            ])
        })
        .collect();

    Paragraph::new(lines).render(chunks[0], buf);

    Paragraph::new("↑ ↓ / j k  navigate    ◄ ► / h l  change    Esc  save & back")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .render(chunks[1], buf);
}

fn render_placeholder(title: &str, message: &str, area: Rect, buf: &mut Buffer) {
    let block = Block::bordered()
        .title(format!(" {title} "))
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(Color::Yellow));
    let inner = block.inner(area);
    block.render(area, buf);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(2)])
        .split(inner);

    Paragraph::new(message)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .render(chunks[0], buf);

    Paragraph::new("Esc / q  back to menu")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .render(chunks[1], buf);
}

fn bool_label(v: bool) -> &'static str {
    if v { "On" } else { "Off" }
}
