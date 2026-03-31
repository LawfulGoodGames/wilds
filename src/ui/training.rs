use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph, Widget},
};

use crate::app::{App, TrainingPhase};
use crate::character::{
    MajorSkill, proficiency_xp_for_level, training_session_plan_for_major,
    training_session_plan_for_minor,
};

use super::shared::{
    GOLD, dim_style, hint_bar, normal_style, progress_bar, render_placeholder, render_status_bar,
    selected_style,
};

pub fn render_training(app: &App, area: Rect, buf: &mut Buffer) {
    let Some(ch) = &app.active_character else {
        render_placeholder("Training", "No active character.", area, buf);
        return;
    };

    let outer = Block::bordered()
        .title(" Training Hall ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(GOLD));
    let inner = outer.inner(area);
    outer.render(area, buf);

    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(1),
        Constraint::Length(10),
        Constraint::Length(2),
    ])
    .split(inner);
    render_status_bar(app, chunks[0], buf);
    let panels = Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[1]);

    let total_entries = MajorSkill::ALL.len() + ch.proficiencies.len();
    let selected_idx = app.training.cursor.min(total_entries.saturating_sub(1));
    let list = MajorSkill::ALL
        .iter()
        .enumerate()
        .map(|(idx, skill)| {
            let style = if idx == selected_idx {
                selected_style()
            } else {
                normal_style()
            };
            Line::from(Span::styled(
                format!("{}  Rank {:>2}", skill.full_name(), ch.major_skill(*skill)),
                style,
            ))
        })
        .chain(ch.proficiencies.iter().enumerate().map(|(idx, skill)| {
            let list_idx = MajorSkill::ALL.len() + idx;
            let style = if list_idx == selected_idx {
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
    Paragraph::new(list)
        .block(
            Block::bordered()
                .title(" Proficiencies ")
                .border_type(BorderType::Rounded)
                .style(dim_style()),
        )
        .render(panels[0], buf);

    let detail = if selected_idx < MajorSkill::ALL.len() {
        let skill = MajorSkill::ALL[selected_idx];
        let current_xp = ch.major_skill_xp(skill);
        let xp_to_next = ch.major_skill_xp_to_next(skill);
        let next_rank_xp = current_xp + xp_to_next as i32;
        let plan = training_session_plan_for_major(skill, ch.major_skill_xp(skill), &ch.stats);
        vec![
            Line::from(Span::styled(
                skill.full_name(),
                Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(skill.description()),
            Line::from(""),
            Line::from(format!(
                "Rank {}  XP to next {}  Mod {:+}",
                ch.major_skill(skill),
                xp_to_next,
                ch.stats.modifier(skill)
            )),
            Line::from(if xp_to_next == 0 {
                format!("XP {}  |  At rank cap", current_xp)
            } else {
                format!(
                    "XP {} / {}  ({} needed)",
                    current_xp, next_rank_xp, xp_to_next
                )
            }),
            Line::from(Span::styled(
                progress_bar(ch.major_skill_progress(skill), 28),
                Style::default().fg(Color::Cyan),
            )),
            Line::from(""),
            Line::from(format!("Affects: {}", skill.effects_summary())),
            Line::from(format!("Training focus: {}", skill.training_focus())),
            Line::from(""),
            Line::from(format!(
                "Expected drill: {} letters  |  Show time {:.1}s",
                plan.beats,
                (plan.response_ticks * plan.beats as u32) as f64 / 30.0
            )),
            Line::from(format!(
                "Rewards: Poor +{} XP / {}h  Solid +{} XP / {}h  Great +{} XP / {}h",
                plan.poor_xp,
                plan.poor_hours,
                plan.solid_xp,
                plan.solid_hours,
                plan.great_xp,
                plan.great_hours
            )),
        ]
    } else {
        let skill = &ch.proficiencies[selected_idx - MajorSkill::ALL.len()];
        let xp_to_next = skill.xp_to_next();
        let next_rank_xp = proficiency_xp_for_level(skill.level() + 1);
        let plan = training_session_plan_for_minor(skill.kind, skill.xp, &ch.stats);
        vec![
            Line::from(Span::styled(
                skill.kind.name(),
                Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(skill.kind.description()),
            Line::from(""),
            Line::from(format!("Rank {}  XP to next {}", skill.level(), xp_to_next)),
            Line::from(if xp_to_next == 0 {
                format!("XP {}  |  At rank cap", skill.xp)
            } else {
                format!(
                    "XP {} / {}  ({} needed)",
                    skill.xp, next_rank_xp, xp_to_next
                )
            }),
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
            Line::from(""),
            Line::from(format!(
                "Expected drill: {} letters  |  Show time {:.1}s",
                plan.beats,
                (plan.response_ticks * plan.beats as u32) as f64 / 30.0
            )),
            Line::from(format!(
                "Rewards: Poor +{} XP / {}h  Solid +{} XP / {}h  Great +{} XP / {}h",
                plan.poor_xp,
                plan.poor_hours,
                plan.solid_xp,
                plan.solid_hours,
                plan.great_xp,
                plan.great_hours
            )),
        ]
    };
    Paragraph::new(detail)
        .block(
            Block::bordered()
                .title(" Training Detail ")
                .border_type(BorderType::Rounded)
                .style(dim_style()),
        )
        .wrap(ratatui::widgets::Wrap { trim: true })
        .render(panels[1], buf);

    let mut lower = if let Some(session) = &app.training.session {
        if session.phase == TrainingPhase::Showing {
            vec![
                Line::from(Span::styled(
                    format!("Memory Drill: {}", session.target.name()),
                    Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from("Memorize this code before it disappears:"),
                Line::from(""),
                Line::from(Span::styled(
                    session
                        .sequence
                        .iter()
                        .map(|ch| ch.to_ascii_uppercase().to_string())
                        .collect::<Vec<_>>()
                        .join(" "),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    progress_bar(
                        session.reveal_ticks_remaining as f64
                            / (session.plan.response_ticks.max(1) * session.plan.beats as u32)
                                as f64,
                        28,
                    ),
                    Style::default().fg(Color::Green),
                )),
                Line::from(format!(
                    "Code length {} letters   Hides in {:.1}s",
                    session.sequence.len(),
                    session.reveal_ticks_remaining as f64 / 30.0
                )),
            ]
        } else {
            let mut slots = Vec::with_capacity(session.sequence.len());
            for idx in 0..session.sequence.len() {
                if idx < session.input_index {
                    slots.push(session.sequence[idx].to_ascii_uppercase().to_string());
                } else {
                    slots.push("_".to_string());
                }
            }
            vec![
                Line::from(Span::styled(
                    format!("Recall: {}", session.target.name()),
                    Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from("Type the code back in the same order:"),
                Line::from(""),
                Line::from(Span::styled(
                    slots.join(" "),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    progress_bar(
                        session.input_index as f64 / session.sequence.len().max(1) as f64,
                        28,
                    ),
                    Style::default().fg(Color::Cyan),
                )),
                Line::from(format!(
                    "Entered {}/{}   Correct {}   Misses {}",
                    session.input_index,
                    session.sequence.len(),
                    session.hits,
                    session.misses
                )),
            ]
        }
    } else if let Some(result) = &app.training.result {
        let mut lines = vec![
            Line::from(Span::styled(
                format!(
                    "Result: {} for {}",
                    result.tier.label(),
                    result.target.name()
                ),
                Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(format!(
                "Score {}/{}   Hits {}   Misses {}",
                result.score, result.max_score, result.hits, result.misses
            )),
            Line::from(format!(
                "Gained +{} XP and spent {}h",
                result.gained_xp, result.hours
            )),
        ];
        if let Some(rank) = result.level_up_rank {
            lines.push(Line::from(Span::styled(
                format!("LEVEL UP! Reached rank {}", rank),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
        }
        if !result.achievement_lines.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from("Achievements:"));
            lines.extend(
                result
                    .achievement_lines
                    .iter()
                    .map(|line| Line::from(line.clone())),
            );
        }
        lines
    } else {
        vec![
            Line::from(Span::styled(
                "Start a Drill",
                Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Choose a proficiency, then press Enter to begin a memory drill."),
            Line::from("The game will briefly show a letter code, then hide it."),
            Line::from(
                "Type the letters back in order. Better recall means more XP and less time spent.",
            ),
        ]
    };

    if let Some(message) = &app.status_message {
        lower.push(Line::from(""));
        lower.push(Line::from(Span::styled(
            message.clone(),
            Style::default().fg(Color::Cyan),
        )));
    }
    Paragraph::new(lower)
        .block(
            Block::bordered()
                .title(" Drill ")
                .border_type(BorderType::Rounded)
                .style(dim_style()),
        )
        .wrap(ratatui::widgets::Wrap { trim: true })
        .render(chunks[2], buf);

    let hint = if app.training.session.is_some() {
        "Type letters when the code is hidden    Esc cancel and return"
    } else {
        "↑ ↓ choose proficiency    Enter begin drill    Esc return"
    };
    hint_bar(hint, chunks[3], buf);
}
