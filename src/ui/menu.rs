use super::{GOLD, TITLE, dim_style, hint_bar, normal_style, render_centered, selected_style};
use crate::app::{App, MenuItem};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph, Widget},
};

pub fn render_main_menu(app: &App, area: Rect, buf: &mut Buffer) {
    let outer = Block::bordered()
        .border_type(BorderType::Rounded)
        .style(dim_style());
    let inner = outer.inner(area);
    outer.render(area, buf);

    let chunks = Layout::vertical([
        Constraint::Length(7),
        Constraint::Length(1),
        Constraint::Min(4),
        Constraint::Length(2),
    ])
    .split(inner);

    Paragraph::new(TITLE)
        .style(Style::default().fg(GOLD))
        .alignment(Alignment::Center)
        .render(chunks[0], buf);

    let lines = MenuItem::ALL
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            if idx == app.selected {
                Line::from(Span::styled(
                    format!("> {} <", item.label()),
                    selected_style(),
                ))
            } else {
                Line::from(Span::styled(item.label(), normal_style()))
            }
        })
        .collect::<Vec<_>>();
    Paragraph::new(lines)
        .alignment(Alignment::Center)
        .render(chunks[2], buf);
    hint_bar("↑ ↓ navigate    Enter select    q quit", chunks[3], buf);
}

pub fn render_options(app: &App, area: Rect, buf: &mut Buffer) {
    let outer = Block::bordered()
        .title(" Options ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(GOLD));
    let inner = outer.inner(area);
    outer.render(area, buf);
    let chunks = Layout::vertical([Constraint::Min(1), Constraint::Length(2)]).split(inner);
    let settings = &app.settings;
    let entries = [
        (
            "Sound Effects",
            if settings.sound_effects {
                "On".to_string()
            } else {
                "Off".to_string()
            },
        ),
        ("Music Volume", format!("{}%", settings.music_volume)),
        ("Font Size", settings.font_size.label().to_string()),
        ("Color Theme", settings.color_theme.label().to_string()),
        (
            "Show Hints",
            if settings.show_hints {
                "On".to_string()
            } else {
                "Off".to_string()
            },
        ),
        ("Difficulty", settings.difficulty.label().to_string()),
    ];
    let lines = entries
        .iter()
        .enumerate()
        .map(|(idx, (label, value))| {
            let style = if idx == app.options_cursor {
                selected_style()
            } else {
                normal_style()
            };
            Line::from(vec![
                Span::styled(format!("{label:<20}"), style),
                Span::styled(format!("{value}"), style),
            ])
        })
        .collect::<Vec<_>>();
    Paragraph::new(lines).render(chunks[0], buf);
    hint_bar(
        "↑ ↓ navigate    ← → change    Esc save & back",
        chunks[1],
        buf,
    );
}

pub fn render_load_game(app: &App, area: Rect, buf: &mut Buffer) {
    let outer = Block::bordered()
        .title(" Load Game ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(GOLD));
    let inner = outer.inner(area);
    outer.render(area, buf);
    if app.saved_characters.is_empty() {
        render_centered(inner, buf, "No saved adventurers found.");
        return;
    }
    let panels =
        Layout::horizontal([Constraint::Percentage(44), Constraint::Percentage(56)]).split(inner);
    let list = app
        .saved_characters
        .iter()
        .enumerate()
        .map(|(idx, character)| {
            let row = format!(
                "  {:<14} {:<8} Lv.{}",
                character.name,
                character.class.name(),
                character.level
            );
            if idx == app.load_cursor {
                Line::from(Span::styled(format!("▶{}", &row[1..]), selected_style()))
            } else {
                Line::from(Span::styled(row, normal_style()))
            }
        })
        .collect::<Vec<_>>();
    Paragraph::new(list)
        .block(
            Block::bordered()
                .title(" Saves ")
                .border_type(BorderType::Rounded)
                .style(dim_style()),
        )
        .render(panels[0], buf);
    let ch = &app.saved_characters[app.load_cursor];
    let detail = vec![
        Line::from(Span::styled(
            &ch.name,
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
        )),
        Line::from(format!("{} {}", ch.race.name(), ch.class.name())),
        Line::from(format!("Level {}  XP {}", ch.level, ch.xp)),
        Line::from(format!("HP {}/{}", ch.resources.hp, ch.resources.max_hp)),
        Line::from(format!(
            "Mana {}/{}  Stamina {}/{}",
            ch.resources.mana,
            ch.resources.max_mana,
            ch.resources.stamina,
            ch.resources.max_stamina
        )),
        Line::from(""),
        Line::from(format!("Gear kit: {}", ch.gear)),
    ];
    Paragraph::new(detail)
        .block(
            Block::bordered()
                .title(" Details ")
                .border_type(BorderType::Rounded)
                .style(dim_style()),
        )
        .render(panels[1], buf);
}
