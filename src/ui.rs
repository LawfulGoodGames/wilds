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
            Screen::NewGame => render_placeholder("New Game", "Character creation coming soon...", area, buf),
            Screen::LoadGame => render_placeholder("Load Game", "Save file browser coming soon...", area, buf),
            Screen::Options => render_placeholder("Options", "Settings coming soon...", area, buf),
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
            Constraint::Length(7), // ASCII title
            Constraint::Length(1), // spacer
            Constraint::Min(4),    // menu items
            Constraint::Length(2), // hint
        ])
        .split(inner);

    let title = Paragraph::new(TITLE)
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center);
    title.render(chunks[0], buf);

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
                Line::from(Span::styled(
                    item.label(),
                    Style::default().fg(Color::White),
                ))
            }
        })
        .collect();

    let menu = Paragraph::new(menu_lines).alignment(Alignment::Center);
    menu.render(chunks[2], buf);

    let hint = Paragraph::new("↑ ↓ / j k  navigate    Enter  select    q  quit")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    hint.render(chunks[3], buf);
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

    let content = Paragraph::new(message)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center);
    content.render(chunks[0], buf);

    let hint = Paragraph::new("Esc / q  back to menu")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    hint.render(chunks[1], buf);
}
