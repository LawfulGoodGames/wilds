use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph, Widget},
};

use crate::app::{App, CharacterTab};
use crate::character::MajorSkill;
use crate::combat::ability_def;
use crate::inventory::{EquipSlot, find_def};

use super::shared::{
    GOLD, dim_style, hint_bar, normal_style, progress_bar, render_placeholder, render_status_bar,
    selected_style,
};

pub fn render_character_sheet(app: &App, area: Rect, buf: &mut Buffer) {
    let Some(ch) = &app.active_character else {
        render_placeholder("Character", "No active character.", area, buf);
        return;
    };
    let outer = Block::bordered()
        .title(" Character ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(GOLD));
    let inner = outer.inner(area);
    outer.render(area, buf);

    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(2),
    ])
    .split(inner);
    render_status_bar(app, chunks[0], buf);
    let tabs = CharacterTab::ALL
        .iter()
        .map(|tab| {
            if *tab == app.character_tab {
                Span::styled(format!("[{}]", tab.label()), selected_style())
            } else {
                Span::styled(tab.label().to_string(), dim_style())
            }
        })
        .flat_map(|span| [span, Span::raw("  ")])
        .collect::<Vec<_>>();
    Paragraph::new(Line::from(tabs))
        .alignment(Alignment::Center)
        .render(chunks[1], buf);

    match app.character_tab {
        CharacterTab::Abilities => {
            let lines = ch
                .known_abilities
                .iter()
                .enumerate()
                .map(|(idx, ability)| {
                    let style = if idx
                        == app
                            .character_cursor
                            .min(ch.known_abilities.len().saturating_sub(1))
                    {
                        selected_style()
                    } else {
                        normal_style()
                    };
                    Line::from(Span::styled(
                        format!(
                            "{}  Rank {}  Cooldown {}",
                            ability_def(&ability.ability_id)
                                .map(|def| def.name)
                                .unwrap_or(ability.ability_id.as_str()),
                            ability.rank,
                            ability.cooldown_remaining
                        ),
                        style,
                    ))
                })
                .collect::<Vec<_>>();
            Paragraph::new(lines)
                .block(
                    Block::bordered()
                        .title(" Abilities ")
                        .border_type(BorderType::Rounded)
                        .style(dim_style()),
                )
                .render(chunks[2], buf);
        }
        CharacterTab::Proficiencies => {
            let total_entries = MajorSkill::ALL.len() + ch.proficiencies.len();
            let lines = MajorSkill::ALL
                .iter()
                .enumerate()
                .map(|(idx, skill)| {
                    let style = if idx == app.character_cursor.min(total_entries.saturating_sub(1))
                    {
                        selected_style()
                    } else {
                        normal_style()
                    };
                    Line::from(Span::styled(
                        format!(
                            "{}  Rank {:>2}  Mod {:+}",
                            skill.full_name(),
                            ch.major_skill(*skill),
                            ch.stats.modifier(*skill)
                        ),
                        style,
                    ))
                })
                .chain(ch.proficiencies.iter().enumerate().map(|(idx, skill)| {
                    let list_idx = MajorSkill::ALL.len() + idx;
                    let style =
                        if list_idx == app.character_cursor.min(total_entries.saturating_sub(1)) {
                            selected_style()
                        } else {
                            normal_style()
                        };
                    Line::from(Span::styled(
                        format!("{}  Rank {:>2}", skill.kind.name(), skill.level()),
                        style,
                    ))
                }))
                .collect::<Vec<_>>();
            let panels =
                Layout::horizontal([Constraint::Percentage(42), Constraint::Percentage(58)])
                    .split(chunks[2]);
            Paragraph::new(lines)
                .block(
                    Block::bordered()
                        .title(" All Proficiencies ")
                        .border_type(BorderType::Rounded)
                        .style(dim_style()),
                )
                .render(panels[0], buf);

            let selected_idx = app.character_cursor.min(total_entries.saturating_sub(1));
            let detail = if selected_idx < MajorSkill::ALL.len() {
                let major = MajorSkill::ALL[selected_idx];
                let score = ch.major_skill(major);
                let modifier = ch.stats.modifier(major);
                let mut lines = vec![
                    Line::from(Span::styled(
                        major.full_name(),
                        Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                    Line::from(major.description()),
                    Line::from(""),
                    Line::from(format!(
                        "Rank {}  XP to next {}  Modifier {:+}",
                        score,
                        ch.major_skill_xp_to_next(major),
                        modifier
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        progress_bar(ch.major_skill_progress(major), 28),
                        Style::default().fg(Color::Cyan),
                    )),
                    Line::from(""),
                    Line::from(format!("Governing stat: {}", major.full_name())),
                    Line::from(format!("Affects: {}", major.effects_summary())),
                    Line::from(format!("Training focus: {}", major.training_focus())),
                ];
                lines.push(Line::from(""));
                lines.push(Line::from(
                    app.status_message
                        .as_deref()
                        .unwrap_or("Train proficiencies from the town's Train action."),
                ));
                lines
            } else {
                ch.proficiencies
                    .get(selected_idx - MajorSkill::ALL.len())
                    .map(|skill| {
                        let mut lines = vec![
                            Line::from(Span::styled(
                                skill.kind.name(),
                                Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
                            )),
                            Line::from(""),
                            Line::from(skill.kind.description()),
                            Line::from(""),
                            Line::from(format!(
                                "Rank {}  XP to next {}",
                                skill.level(),
                                skill.xp_to_next()
                            )),
                            Line::from(Span::styled(
                                progress_bar(skill.progress(), 28),
                                Style::default().fg(Color::Cyan),
                            )),
                            Line::from(""),
                            Line::from(format!(
                                "Governing stat: {}",
                                skill.kind.governing_stat().full_name()
                            )),
                            Line::from(format!("Affects: {}", skill.kind.effects_summary())),
                            Line::from(format!("Training focus: {}", skill.kind.training_focus())),
                        ];
                        lines.push(Line::from(""));
                        lines.push(Line::from(
                            app.status_message
                                .as_deref()
                                .unwrap_or("Train proficiencies from the town's Train action."),
                        ));
                        lines
                    })
                    .unwrap_or_else(|| vec![Line::from("No proficiency selected.")])
            };
            Paragraph::new(detail)
                .block(
                    Block::bordered()
                        .title(" Proficiency Detail ")
                        .border_type(BorderType::Rounded)
                        .style(dim_style()),
                )
                .wrap(ratatui::widgets::Wrap { trim: true })
                .render(panels[1], buf);
        }
        CharacterTab::Equipment => {
            let lines = EquipSlot::ALL
                .iter()
                .map(|slot| {
                    let item = app
                        .equipment
                        .get_slot(*slot)
                        .and_then(find_def)
                        .map(|def| def.name)
                        .unwrap_or("(empty)");
                    Line::from(format!("{:<8} {}", slot.label(), item))
                })
                .collect::<Vec<_>>();
            Paragraph::new(lines)
                .block(
                    Block::bordered()
                        .title(" Equipment Summary ")
                        .border_type(BorderType::Rounded)
                        .style(dim_style()),
                )
                .render(chunks[2], buf);
        }
    }

    let hint = if app.character_tab == CharacterTab::Proficiencies {
        "← → / Tab change tab    ↑ ↓ browse    Train from town    Esc return"
    } else {
        "← → / Tab change tab    ↑ ↓ browse    Esc return"
    };
    hint_bar(hint, chunks[3], buf);
}
