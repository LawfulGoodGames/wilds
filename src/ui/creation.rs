use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph, Widget},
};

use crate::app::App;
use crate::character::{Class, CreationStep, GearPackage, MajorSkill, Race};
use crate::inventory::{EquipSlot, find_def, gear_package_items};

use super::shared::{GOLD, dim_style, hint_bar, normal_style, selected_style, stat_bar};

pub fn render_character_creation(app: &App, area: Rect, buf: &mut Buffer) {
    let outer = Block::bordered()
        .title(" Character Creation ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(GOLD));
    let inner = outer.inner(area);
    outer.render(area, buf);

    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(2),
    ])
    .split(inner);

    render_step_bar(app.creation.step, chunks[0], buf);
    match app.creation.step {
        CreationStep::Name => render_creation_name(app, chunks[2], chunks[3], buf),
        CreationStep::Race => render_creation_race(app, chunks[2], chunks[3], buf),
        CreationStep::Class => render_creation_class(app, chunks[2], chunks[3], buf),
        CreationStep::Stats => render_creation_proficiencies(app, chunks[2], chunks[3], buf),
        CreationStep::Gear => render_creation_gear(app, chunks[2], chunks[3], buf),
        CreationStep::Confirm => render_creation_confirm(app, chunks[2], chunks[3], buf),
    }
}

fn render_step_bar(current: CreationStep, area: Rect, buf: &mut Buffer) {
    let spans = CreationStep::ALL
        .iter()
        .enumerate()
        .flat_map(|(idx, step)| {
            let label = if *step == current {
                Span::styled(format!("[{}]", step.label()), selected_style())
            } else if step.index() < current.index() {
                Span::styled(step.label().to_string(), Style::default().fg(Color::Green))
            } else {
                Span::styled(step.label().to_string(), dim_style())
            };
            if idx + 1 < CreationStep::ALL.len() {
                vec![label, Span::styled(" > ", dim_style())]
            } else {
                vec![label]
            }
        })
        .collect::<Vec<_>>();
    Paragraph::new(Line::from(spans))
        .alignment(Alignment::Center)
        .render(area, buf);
}

fn render_creation_name(app: &App, content: Rect, hint: Rect, buf: &mut Buffer) {
    let chunks = Layout::vertical([
        Constraint::Min(1),
        Constraint::Length(3),
        Constraint::Min(1),
    ])
    .split(content);
    let display = if app.creation.name.is_empty() {
        Span::styled("Enter your name...", dim_style())
    } else {
        Span::styled(
            format!("{}_", app.creation.name),
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
        )
    };
    Paragraph::new(Line::from(display))
        .block(
            Block::bordered()
                .title(" Your Name ")
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(GOLD)),
        )
        .alignment(Alignment::Center)
        .render(chunks[1], buf);
    hint_bar("Type name    Enter continue    Esc back", hint, buf);
}

fn render_creation_race(app: &App, content: Rect, hint: Rect, buf: &mut Buffer) {
    let chunks =
        Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).split(content);
    let list = Race::ALL
        .iter()
        .enumerate()
        .map(|(idx, race)| {
            if idx == app.creation.race_cursor {
                Line::from(Span::styled(format!("▶ {}", race.name()), selected_style()))
            } else {
                Line::from(Span::styled(format!("  {}", race.name()), normal_style()))
            }
        })
        .collect::<Vec<_>>();
    Paragraph::new(list)
        .block(
            Block::bordered()
                .title(" Race ")
                .border_type(BorderType::Rounded)
                .style(dim_style()),
        )
        .render(chunks[0], buf);
    let race = app.creation.selected_race();
    let detail = vec![
        Line::from(Span::styled(
            race.name(),
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(race.description(), normal_style())),
        Line::from(""),
        Line::from(vec![
            Span::styled("Bonuses: ", dim_style()),
            Span::styled(race.bonus_label(), Style::default().fg(Color::Green)),
        ]),
    ];
    Paragraph::new(detail)
        .block(
            Block::bordered()
                .title(" Details ")
                .border_type(BorderType::Rounded)
                .style(dim_style()),
        )
        .wrap(ratatui::widgets::Wrap { trim: true })
        .render(chunks[1], buf);
    hint_bar("↑ ↓ navigate    Enter confirm    Esc back", hint, buf);
}

fn render_creation_class(app: &App, content: Rect, hint: Rect, buf: &mut Buffer) {
    let chunks =
        Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).split(content);
    let list = Class::ALL
        .iter()
        .enumerate()
        .map(|(idx, class)| {
            if idx == app.creation.class_cursor {
                Line::from(Span::styled(
                    format!("▶ {}", class.name()),
                    selected_style(),
                ))
            } else {
                Line::from(Span::styled(format!("  {}", class.name()), normal_style()))
            }
        })
        .collect::<Vec<_>>();
    Paragraph::new(list)
        .block(
            Block::bordered()
                .title(" Class ")
                .border_type(BorderType::Rounded)
                .style(dim_style()),
        )
        .render(chunks[0], buf);
    let class = app.creation.selected_class();
    let detail = vec![
        Line::from(Span::styled(
            class.name(),
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(class.description(), normal_style())),
        Line::from(""),
        Line::from(vec![
            Span::styled("Class leaning: ", dim_style()),
            Span::styled(class.primary_stats(), Style::default().fg(Color::Cyan)),
        ]),
    ];
    Paragraph::new(detail)
        .block(
            Block::bordered()
                .title(" Details ")
                .border_type(BorderType::Rounded)
                .style(dim_style()),
        )
        .wrap(ratatui::widgets::Wrap { trim: true })
        .render(chunks[1], buf);
    hint_bar("↑ ↓ navigate    Enter confirm    Esc back", hint, buf);
}

fn render_creation_proficiencies(app: &App, content: Rect, hint: Rect, buf: &mut Buffer) {
    let c = &app.creation;
    let bonuses = c.selected_race().stat_bonuses();
    let chunks = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).split(content);
    Paragraph::new(Line::from(vec![
        Span::styled("Points remaining: ", dim_style()),
        Span::styled(
            c.points_remaining.to_string(),
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  (all proficiencies start at 8, cap 13)", dim_style()),
    ]))
    .alignment(Alignment::Center)
    .render(chunks[0], buf);

    let lines = (0..6)
        .map(|idx| {
            let skill = MajorSkill::ALL[idx];
            let base = c.base_stats.by_skill(skill);
            let bonus = bonuses.by_skill(skill);
            let total = base + bonus;
            let label_style = if idx == c.stat_cursor {
                selected_style()
            } else {
                normal_style()
            };
            let mut spans = vec![
                Span::styled(
                    format!(
                        "  {:<3} {:<13} {:>2}",
                        skill.short_name(),
                        skill.full_name(),
                        base
                    ),
                    label_style,
                ),
                Span::styled(stat_bar(base - 8, 5), Style::default().fg(Color::Cyan)),
            ];
            if bonus > 0 {
                spans.push(Span::styled(
                    format!("  +{bonus} = {total}"),
                    Style::default().fg(Color::Green),
                ));
            }
            Line::from(spans)
        })
        .collect::<Vec<_>>();
    Paragraph::new(lines)
        .block(
            Block::bordered()
                .title(" Proficiencies ")
                .border_type(BorderType::Rounded)
                .style(dim_style()),
        )
        .render(chunks[1], buf);
    hint_bar(
        "↑ ↓ navigate    ← → adjust    Enter continue    Esc back",
        hint,
        buf,
    );
}

fn render_creation_gear(app: &App, content: Rect, hint: Rect, buf: &mut Buffer) {
    let chunks =
        Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).split(content);
    let list = GearPackage::ALL
        .iter()
        .enumerate()
        .map(|(idx, gear)| {
            if idx == app.creation.gear_cursor {
                Line::from(Span::styled(format!("▶ {}", gear.name()), selected_style()))
            } else {
                Line::from(Span::styled(format!("  {}", gear.name()), normal_style()))
            }
        })
        .collect::<Vec<_>>();
    Paragraph::new(list)
        .block(
            Block::bordered()
                .title(" Starting Gear ")
                .border_type(BorderType::Rounded)
                .style(dim_style()),
        )
        .render(chunks[0], buf);
    let gear = app.creation.selected_gear();
    let mut detail = vec![
        Line::from(Span::styled(
            gear.name(),
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(gear.description(), normal_style())),
        Line::from(""),
    ];
    for (slot_key, item_type) in gear_package_items(gear.name()) {
        if let Some(def) = find_def(item_type) {
            let slot = EquipSlot::ALL
                .iter()
                .find(|slot| slot.db_key() == *slot_key)
                .copied();
            detail.push(Line::from(vec![
                Span::styled(
                    format!("{}: ", slot.map(|slot| slot.label()).unwrap_or("Gear")),
                    dim_style(),
                ),
                Span::styled(def.name, normal_style()),
            ]));
        }
    }
    Paragraph::new(detail)
        .block(
            Block::bordered()
                .title(" Contents ")
                .border_type(BorderType::Rounded)
                .style(dim_style()),
        )
        .render(chunks[1], buf);
    hint_bar("↑ ↓ navigate    Enter confirm    Esc back", hint, buf);
}

fn render_creation_confirm(app: &App, content: Rect, hint: Rect, buf: &mut Buffer) {
    let stats = app.creation.final_stats();
    let lines = vec![
        Line::from(vec![
            Span::styled("Name: ", dim_style()),
            Span::styled(&app.creation.name, Style::default().fg(GOLD)),
        ]),
        Line::from(vec![
            Span::styled("Race: ", dim_style()),
            Span::styled(app.creation.selected_race().name(), normal_style()),
        ]),
        Line::from(vec![
            Span::styled("Class: ", dim_style()),
            Span::styled(app.creation.selected_class().name(), normal_style()),
        ]),
        Line::from(""),
        Line::from(Span::styled("Starting Proficiencies", dim_style())),
        Line::from(format!(
            "ATK {}  STR {}  DEF {}",
            stats.charisma, stats.strength, stats.constitution
        )),
        Line::from(format!(
            "RNG {}  PRY {}  MAG {}",
            stats.dexterity, stats.wisdom, stats.intelligence
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Gear: ", dim_style()),
            Span::styled(app.creation.selected_gear().name(), normal_style()),
        ]),
    ];
    Paragraph::new(lines)
        .block(
            Block::bordered()
                .title(" Confirm ")
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(GOLD)),
        )
        .render(content, buf);
    hint_bar("Enter begin adventure    Esc back", hint, buf);
}
