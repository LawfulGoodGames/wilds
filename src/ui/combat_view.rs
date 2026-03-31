use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph, Widget},
};

use crate::app::App;
use crate::combat::{ActionTab, TurnRef, ability_def};

use super::shared::{
    GOLD, dim_style, hint_bar, normal_style, render_placeholder, resource_bar, selected_style,
};

pub fn render_combat(app: &App, area: Rect, buf: &mut Buffer) {
    let Some(combat) = &app.combat else {
        render_placeholder("Combat", "No active combat.", area, buf);
        return;
    };
    let outer = Block::bordered()
        .title(format!(" Combat: {} ", combat.encounter_name))
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(GOLD));
    let inner = outer.inner(area);
    outer.render(area, buf);
    let chunks = Layout::vertical([
        Constraint::Length(8),
        Constraint::Min(1),
        Constraint::Length(2),
    ])
    .split(inner);
    let header = Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(chunks[0]);

    let player_lines = vec![
        Line::from(Span::styled(
            &combat.player.name,
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
        )),
        Line::from(resource_bar(
            "HP",
            combat.player.resources.hp,
            combat.player.resources.max_hp,
            16,
            Color::LightRed,
        )),
        Line::from(resource_bar(
            "Mana",
            combat.player.resources.mana,
            combat.player.resources.max_mana,
            16,
            Color::LightBlue,
        )),
        Line::from(resource_bar(
            "Stam",
            combat.player.resources.stamina,
            combat.player.resources.max_stamina,
            16,
            Color::LightGreen,
        )),
        Line::from(format!(
            "Defense {}  Initiative {}",
            combat.player.defense, combat.player.initiative
        )),
        Line::from(format!(
            "Resist P:{} F:{} I:{} L:{} X:{} H:{} S:{}",
            combat.player.resistances.physical,
            combat.player.resistances.fire,
            combat.player.resistances.frost,
            combat.player.resistances.lightning,
            combat.player.resistances.poison,
            combat.player.resistances.holy,
            combat.player.resistances.shadow
        )),
        Line::from(format!(
            "Turn: {}",
            if combat.current_turn() == TurnRef::Player {
                "Player"
            } else {
                "Enemy"
            }
        )),
    ];
    Paragraph::new(player_lines)
        .block(
            Block::bordered()
                .title(" Player ")
                .border_type(BorderType::Rounded)
                .style(dim_style()),
        )
        .render(header[0], buf);

    let enemy_lines = combat
        .enemies
        .iter()
        .enumerate()
        .flat_map(|(idx, enemy)| {
            let style = if idx == combat.selected_target {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                normal_style()
            };
            [
                Line::from(Span::styled(
                    format!(
                        "{}{}",
                        if idx == combat.selected_target {
                            "▶ "
                        } else {
                            "  "
                        },
                        enemy.name
                    ),
                    style,
                )),
                Line::from(format!(
                    "  HP {}/{}  DEF {}  {}",
                    enemy.resources.hp,
                    enemy.resources.max_hp,
                    enemy.defense,
                    if enemy.resources.hp == 0 {
                        "[defeated]"
                    } else {
                        enemy.family.as_str()
                    }
                )),
            ]
        })
        .collect::<Vec<_>>();
    Paragraph::new(enemy_lines)
        .block(
            Block::bordered()
                .title(" Enemies ")
                .border_type(BorderType::Rounded)
                .style(dim_style()),
        )
        .render(header[1], buf);

    let lower = Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(chunks[1]);
    let tabs = [ActionTab::Weapon, ActionTab::Ability, ActionTab::Item]
        .iter()
        .flat_map(|tab| {
            let span = if *tab == combat.action_tab {
                Span::styled(format!("[{}]", tab.label()), selected_style())
            } else {
                Span::styled(tab.label().to_string(), dim_style())
            };
            [span, Span::raw(" ")]
        })
        .collect::<Vec<_>>();
    let mut actions = vec![
        Line::from(Span::styled("[Tab] switch action type", dim_style())),
        Line::from(Span::styled("[t] cycle target", dim_style())),
        Line::from(tabs),
        Line::from(""),
    ];
    match combat.action_tab {
        ActionTab::Weapon => {
            for (idx, attack) in combat.player.weapon_attacks.iter().enumerate() {
                let style = if idx == combat.selected_weapon_attack {
                    selected_style()
                } else {
                    normal_style()
                };
                let cost_suffix = if matches!(
                    combat.player.weapon_kind,
                    Some(crate::inventory::WeaponKind::Magic)
                ) {
                    let cost = ((attack.min_damage + attack.max_damage) / 2 / 3).clamp(2, 5);
                    format!("  Mana {cost}")
                } else {
                    String::new()
                };
                actions.push(Line::from(Span::styled(
                    format!(
                        "{}  Hit +{}  Dmg {}{}",
                        attack.name,
                        attack.accuracy_bonus,
                        attack.damage_range_label(),
                        cost_suffix
                    ),
                    style,
                )));
            }
        }
        ActionTab::Ability => {
            for (idx, ability) in combat.player.ability_ids.iter().enumerate() {
                let style = if idx == combat.selected_ability {
                    selected_style()
                } else {
                    normal_style()
                };
                let ability_name = ability_def(ability)
                    .map(|def| def.name)
                    .unwrap_or(ability.as_str());
                let cost_label = ability_def(ability)
                    .and_then(|def| {
                        def.resource_kind.map(|kind| match kind {
                            crate::combat::ResourceKind::Mana => format!("  Mana {}", def.cost),
                            crate::combat::ResourceKind::Stamina => {
                                format!("  Stam {}", def.cost)
                            }
                        })
                    })
                    .unwrap_or_default();
                actions.push(Line::from(Span::styled(
                    format!("{ability_name}{cost_label}"),
                    style,
                )));
            }
        }
        ActionTab::Item => {
            for (idx, item) in combat.consumables.iter().enumerate() {
                let style = if idx == combat.selected_item {
                    selected_style()
                } else {
                    normal_style()
                };
                let name = item
                    .def()
                    .map(|def| def.name)
                    .unwrap_or(item.item_type.as_str());
                actions.push(Line::from(Span::styled(
                    format!("{name} x{}", item.quantity),
                    style,
                )));
            }
            if combat.consumables.is_empty() {
                actions.push(Line::from(Span::styled(
                    "No combat consumables ready.",
                    dim_style(),
                )));
            }
        }
    }
    Paragraph::new(actions)
        .block(
            Block::bordered()
                .title(" Actions ")
                .border_type(BorderType::Rounded)
                .style(dim_style()),
        )
        .wrap(ratatui::widgets::Wrap { trim: true })
        .render(lower[0], buf);

    let visible_start = combat.log.len().saturating_sub(8);
    let new_start = combat.log.len().saturating_sub(combat.new_log_entries);
    let log_lines = combat
        .log
        .iter()
        .enumerate()
        .skip(visible_start)
        .map(|(idx, event)| {
            let style = if idx >= new_start {
                Style::default().fg(GOLD).add_modifier(Modifier::BOLD)
            } else {
                normal_style()
            };
            Line::from(Span::styled(event.to_line(), style))
        })
        .collect::<Vec<_>>();
    Paragraph::new(log_lines)
        .block(
            Block::bordered()
                .title(" Battle Log ")
                .border_type(BorderType::Rounded)
                .style(dim_style()),
        )
        .wrap(ratatui::widgets::Wrap { trim: true })
        .render(lower[1], buf);

    let footer = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).split(chunks[2]);
    if let Some(summary) = &combat.last_roll_summary {
        Paragraph::new(Span::styled(summary, Style::default().fg(Color::Cyan)))
            .alignment(Alignment::Center)
            .render(footer[0], buf);
    }
    hint_bar(
        "Tab next action  1 weapon  2 ability  3 item  t target  ↑ ↓ choose  Enter use  d defend  f flee",
        footer[1],
        buf,
    );
}
