use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph, Widget},
};

use crate::app::App;
use crate::character::{Class, CreationStep, GearPackage, MajorSkill, MinorSkill, Race};
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
    let minor_bonuses = c.selected_race().minor_skill_bonuses();
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

    let panels = Layout::horizontal([Constraint::Percentage(48), Constraint::Percentage(52)])
        .split(chunks[1]);
    let lines = MajorSkill::ALL
        .iter()
        .enumerate()
        .map(|(idx, skill)| {
            let base = c.base_stats.by_skill(*skill);
            let bonus = bonuses.by_skill(*skill);
            let total = base + bonus;
            let label_style = if idx == c.stat_cursor {
                selected_style()
            } else {
                normal_style()
            };
            let mut spans = vec![
                Span::styled(
                    format!("      {:<13} {:>2}", skill.full_name(), base),
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
        .chain(
            MinorSkill::ALL
                .iter()
                .enumerate()
                .map(|(minor_idx, skill)| {
                    let idx = MajorSkill::ALL.len() + minor_idx;
                    let rank = c.minor_proficiency_rank(*skill);
                    let bonus = minor_bonuses.by_skill(*skill);
                    let total = rank + bonus;
                    let label_style = if idx == c.stat_cursor {
                        selected_style()
                    } else {
                        normal_style()
                    };
                    let mut spans = vec![
                        Span::styled(
                            format!("      {:<13} {:>2}", skill.name(), rank),
                            label_style,
                        ),
                        Span::styled(stat_bar(rank - 8, 5), Style::default().fg(Color::Cyan)),
                    ];
                    if bonus > 0 {
                        spans.push(Span::styled(
                            format!("  +{bonus} = {total}"),
                            Style::default().fg(Color::Green),
                        ));
                    }
                    Line::from(spans)
                }),
        )
        .collect::<Vec<_>>();
    let visible_rows = panels[0].height.saturating_sub(2) as usize;
    let scroll_offset = if visible_rows == 0 {
        0
    } else {
        c.stat_cursor.saturating_sub(visible_rows.saturating_sub(1))
    };
    Paragraph::new(lines)
        .block(
            Block::bordered()
                .title(" Proficiencies ")
                .border_type(BorderType::Rounded)
                .style(dim_style()),
        )
        .scroll((scroll_offset as u16, 0))
        .render(panels[0], buf);

    let detail = if c.stat_cursor < MajorSkill::ALL.len() {
        let skill = MajorSkill::ALL[c.stat_cursor];
        let base = c.base_stats.by_skill(skill);
        let bonus = bonuses.by_skill(skill);
        major_creation_detail(skill, base, bonus)
    } else {
        let skill = MinorSkill::ALL[c.stat_cursor - MajorSkill::ALL.len()];
        let rank = c.minor_proficiency_rank(skill);
        let bonus = minor_bonuses.by_skill(skill);
        minor_creation_detail(skill, rank, bonus)
    };
    Paragraph::new(detail)
        .block(
            Block::bordered()
                .title(" Detail ")
                .border_type(BorderType::Rounded)
                .style(dim_style()),
        )
        .wrap(ratatui::widgets::Wrap { trim: true })
        .render(panels[1], buf);
    hint_bar(
        "↑ ↓ navigate    ← → adjust    Enter continue    Esc back",
        hint,
        buf,
    );
}

fn major_creation_detail(skill: MajorSkill, base: i32, bonus: i32) -> Vec<Line<'static>> {
    let total = base + bonus;
    let affects = match skill {
        MajorSkill::Charisma => "Affects weapon accuracy, crit pressure, and direct martial tempo.",
        MajorSkill::Strength => {
            "Affects physical damage, stamina-heavy actions, and melee scaling."
        }
        MajorSkill::Constitution => {
            "Affects defence, max HP growth, and staying power in long fights."
        }
        MajorSkill::Dexterity => {
            "Affects ranged pressure, initiative, dodge, and battlefield tempo."
        }
        MajorSkill::Wisdom => {
            "Affects healing power, holy resilience, and support-oriented combat value."
        }
        MajorSkill::Intelligence => "Affects spell power, mana scaling, and arcane effectiveness.",
    };
    let mut lines = vec![
        Line::from(Span::styled(
            skill.full_name(),
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(skill.description()),
        Line::from(""),
        Line::from(format!("Starting rank: {base}")),
        Line::from(format!("Current total after race bonus: {total}")),
        Line::from(""),
        Line::from("What it affects:"),
        Line::from(affects),
    ];
    if bonus > 0 {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("Race bonus: +{bonus}"),
            Style::default().fg(Color::Green),
        )));
    }
    lines
}

fn minor_creation_detail(skill: MinorSkill, rank: i32, bonus: i32) -> Vec<Line<'static>> {
    let total = rank + bonus;
    let affects = match skill {
        MinorSkill::Vitality => {
            "Improves endurance-focused study and supports rough-travel survivability themes."
        }
        MinorSkill::Agility => "Supports movement, traversal, and finesse-oriented utility checks.",
        MinorSkill::Alchemy => "Supports potion-making, reagents, and magical field preparation.",
        MinorSkill::Larceny => "Supports locks, traps, sleight of hand, and opportunistic utility.",
        MinorSkill::Runecraft => "Supports runes, wards, catalysts, and arcane utility work.",
        MinorSkill::Crafting => "Supports practical tools, leatherwork, and fine assembly.",
    };
    let mut lines = vec![
        Line::from(Span::styled(
            skill.name(),
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(skill.description()),
        Line::from(""),
        Line::from(format!("Starting rank: {rank}")),
        Line::from(format!("Current total after race bonus: {total}")),
        Line::from(format!(
            "Governing proficiency: {}",
            skill.governing_stat().full_name()
        )),
        Line::from(""),
        Line::from("What it affects:"),
        Line::from(affects),
    ];
    if bonus > 0 {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("Race bonus: +{bonus}"),
            Style::default().fg(Color::Green),
        )));
    }
    lines
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
    let mut proficiency_chunks = MajorSkill::ALL
        .iter()
        .map(|skill| format!("{} {}", skill.full_name(), stats.by_skill(*skill)))
        .collect::<Vec<_>>();
    proficiency_chunks.extend(MinorSkill::ALL.iter().map(|skill| {
        format!(
            "{} {}",
            skill.name(),
            app.creation.final_minor_proficiency_rank(*skill)
        )
    }));
    let proficiency_lines = proficiency_chunks
        .chunks(3)
        .map(|chunk| Line::from(chunk.join("  ")))
        .collect::<Vec<_>>();

    let mut lines = vec![
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
    ];
    lines.extend(proficiency_lines);
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Gear: ", dim_style()),
        Span::styled(app.creation.selected_gear().name(), normal_style()),
    ]));
    Paragraph::new(lines)
        .block(
            Block::bordered()
                .title(" Confirm ")
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(GOLD)),
        )
        .wrap(ratatui::widgets::Wrap { trim: true })
        .render(content, buf);
    hint_bar("Enter begin adventure    Esc back", hint, buf);
}
