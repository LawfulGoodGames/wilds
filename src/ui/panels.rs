use super::{
    GOLD, dim_style, hint_bar, normal_style, progress_bar, render_item_detail, render_placeholder,
    render_status_bar, selected_style,
};
use crate::achievements::achievement_defs;
use crate::app::{App, TownAction};
use crate::combat::encounter_def;
use crate::inventory::{EquipSlot, find_def};
use crate::world::{AreaId, QuestId, VendorId, area_def, quest_def, vendor_def};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph, Widget},
};

pub fn render_town(app: &App, area: Rect, buf: &mut Buffer) {
    let Some(ch) = &app.active_character else {
        render_placeholder("Town", "No active character.", area, buf);
        return;
    };
    let outer = Block::bordered()
        .title(" Hearthmere ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(GOLD));
    let inner = outer.inner(area);
    outer.render(area, buf);

    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(1),
        Constraint::Length(2),
    ])
    .split(inner);
    render_status_bar(app, chunks[0], buf);
    let panels = Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(chunks[1]);

    let list = TownAction::ALL
        .iter()
        .enumerate()
        .map(|(idx, action)| {
            if idx == app.town_cursor {
                Line::from(Span::styled(
                    format!("▶ {}", action.label()),
                    selected_style(),
                ))
            } else {
                Line::from(Span::styled(
                    format!("  {}", action.label()),
                    normal_style(),
                ))
            }
        })
        .collect::<Vec<_>>();
    Paragraph::new(list)
        .block(
            Block::bordered()
                .title(" Town Actions ")
                .border_type(BorderType::Rounded)
                .style(dim_style()),
        )
        .render(panels[0], buf);

    let detail = vec![
        Line::from(Span::styled(
            "Town State",
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!(
            "Unlocked areas: {}",
            app.world_state.unlocked_areas.len()
        )),
        Line::from(format!(
            "Active quests: {}",
            app.world_state.active_quests.len()
        )),
        Line::from(format!(
            "Completed quests: {}",
            app.world_state.completed_quests.len()
        )),
        Line::from(""),
        Line::from(Span::styled(
            app.status_message
                .as_deref()
                .unwrap_or("The town is tense, but stable enough to prepare."),
            normal_style(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("{} watches the square and checks supplies.", ch.name),
            dim_style(),
        )),
    ];
    Paragraph::new(detail)
        .block(
            Block::bordered()
                .title(" Overview ")
                .border_type(BorderType::Rounded)
                .style(dim_style()),
        )
        .wrap(ratatui::widgets::Wrap { trim: true })
        .render(panels[1], buf);

    hint_bar(
        "↑ ↓ choose    Enter confirm    x explore    c character    i inventory    e equipment    q quests    h achievements    v vendors    r rest",
        chunks[2],
        buf,
    );
}

pub fn render_explore(app: &App, area: Rect, buf: &mut Buffer) {
    let outer = Block::bordered()
        .title(" Explore ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(GOLD));
    let inner = outer.inner(area);
    outer.render(area, buf);
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(1),
        Constraint::Length(2),
    ])
    .split(inner);
    render_status_bar(app, chunks[0], buf);
    let panels = Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[1]);
    let list = AreaId::ALL
        .iter()
        .enumerate()
        .map(|(idx, area)| {
            let unlocked = app.world_state.is_area_unlocked(*area);
            let prefix = if idx == app.explore_cursor {
                "▶"
            } else {
                " "
            };
            let label = if unlocked {
                area.label()
            } else {
                "Locked Route"
            };
            let style = if idx == app.explore_cursor {
                selected_style()
            } else if unlocked {
                normal_style()
            } else {
                Style::default().fg(Color::DarkGray)
            };
            Line::from(Span::styled(format!("{prefix} {label}"), style))
        })
        .collect::<Vec<_>>();
    Paragraph::new(list)
        .block(
            Block::bordered()
                .title(" Areas ")
                .border_type(BorderType::Rounded)
                .style(dim_style()),
        )
        .render(panels[0], buf);
    let area = area_def(AreaId::ALL[app.explore_cursor]);
    let detail = vec![
        Line::from(Span::styled(
            area.name,
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(area.description),
        Line::from(""),
        Line::from(vec![
            Span::styled("Danger: ", dim_style()),
            Span::styled(area.danger, normal_style()),
        ]),
        Line::from(vec![
            Span::styled("Encounter pool: ", dim_style()),
            Span::styled(
                area.encounters
                    .iter()
                    .map(|encounter_id| encounter_def(encounter_id).name)
                    .collect::<Vec<_>>()
                    .join(", "),
                Style::default().fg(Color::Cyan),
            ),
        ]),
    ];
    Paragraph::new(detail)
        .block(
            Block::bordered()
                .title(" Route Detail ")
                .border_type(BorderType::Rounded)
                .style(dim_style()),
        )
        .wrap(ratatui::widgets::Wrap { trim: true })
        .render(panels[1], buf);
    hint_bar(
        "↑ ↓ choose area    Enter travel    Esc return",
        chunks[2],
        buf,
    );
}

pub fn render_inventory(app: &App, area: Rect, buf: &mut Buffer) {
    let outer = Block::bordered()
        .title(" Inventory ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(GOLD));
    let inner = outer.inner(area);
    outer.render(area, buf);
    let chunks = Layout::vertical([
        Constraint::Min(1),
        Constraint::Length(1),
        Constraint::Length(2),
    ])
    .split(inner);
    let panels = Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[0]);

    let list = if app.inventory.items.is_empty() {
        vec![Line::from(Span::styled("(empty)", dim_style()))]
    } else {
        app.inventory
            .items
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                let name = item
                    .def()
                    .map(|def| def.name)
                    .unwrap_or(item.item_type.as_str());
                let style = if idx == app.inventory.cursor {
                    selected_style()
                } else {
                    normal_style()
                };
                Line::from(Span::styled(
                    format!("{:<24} x{}", name, item.quantity),
                    style,
                ))
            })
            .collect::<Vec<_>>()
    };
    Paragraph::new(list)
        .block(
            Block::bordered()
                .title(" Pack ")
                .border_type(BorderType::Rounded)
                .style(dim_style()),
        )
        .render(panels[0], buf);

    let detail = match app.inventory.selected_def() {
        None => vec![Line::from("No item selected.")],
        Some(def) => {
            let mut lines = render_item_detail(def);
            lines.push(Line::from(""));
            if def.is_usable() {
                lines.push(Line::from("Enter: use"));
            }
            if def.is_equippable() {
                lines.push(Line::from("Right / e: equip"));
            }
            lines
        }
    };
    Paragraph::new(detail)
        .block(
            Block::bordered()
                .title(" Details ")
                .border_type(BorderType::Rounded)
                .style(dim_style()),
        )
        .scroll((app.detail_scroll, 0))
        .wrap(ratatui::widgets::Wrap { trim: false })
        .render(panels[1], buf);
    if let Some(msg) = &app.inventory.last_use_message {
        Paragraph::new(Span::styled(msg, Style::default().fg(Color::Green)))
            .alignment(Alignment::Center)
            .render(chunks[1], buf);
    }
    hint_bar(
        "↑ ↓ choose    [ ] detail    Enter use    Right/e equip    Esc return",
        chunks[2],
        buf,
    );
}

pub fn render_equipment(app: &App, area: Rect, buf: &mut Buffer) {
    let outer = Block::bordered()
        .title(" Equipment ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(GOLD));
    let inner = outer.inner(area);
    outer.render(area, buf);
    let chunks = Layout::vertical([
        Constraint::Min(1),
        Constraint::Length(1),
        Constraint::Length(2),
    ])
    .split(inner);
    let panels = Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(chunks[0]);
    let list = EquipSlot::ALL
        .iter()
        .enumerate()
        .map(|(idx, slot)| {
            let name = app
                .equipment
                .get_slot(*slot)
                .and_then(find_def)
                .map(|def| def.name)
                .unwrap_or("(empty)");
            let style = if idx == app.equipment_cursor {
                selected_style()
            } else {
                normal_style()
            };
            Line::from(Span::styled(format!("{:<8} {}", slot.label(), name), style))
        })
        .collect::<Vec<_>>();
    Paragraph::new(list)
        .block(
            Block::bordered()
                .title(" Slots ")
                .border_type(BorderType::Rounded)
                .style(dim_style()),
        )
        .render(panels[0], buf);
    let slot = EquipSlot::ALL[app.equipment_cursor];
    let detail = match app.equipment.get_slot(slot).and_then(find_def) {
        None => vec![Line::from(format!("{} is empty.", slot.label()))],
        Some(def) => {
            let mut lines = render_item_detail(def);
            lines.push(Line::from(""));
            lines.push(Line::from("Enter: unequip"));
            lines
        }
    };
    Paragraph::new(detail)
        .block(
            Block::bordered()
                .title(" Details ")
                .border_type(BorderType::Rounded)
                .style(dim_style()),
        )
        .scroll((app.detail_scroll, 0))
        .wrap(ratatui::widgets::Wrap { trim: false })
        .render(panels[1], buf);
    Paragraph::new(Span::styled(
        app.status_message.as_deref().unwrap_or(&format!(
            "Total armor: {}",
            app.equipment.total_armor_bonus()
        )),
        Style::default().fg(Color::Cyan),
    ))
    .alignment(Alignment::Center)
    .render(chunks[1], buf);
    hint_bar(
        "↑ ↓ choose slot    [ ] detail    Enter unequip    Esc return",
        chunks[2],
        buf,
    );
}

pub fn render_quests(app: &App, area: Rect, buf: &mut Buffer) {
    let outer = Block::bordered()
        .title(" Quest Log ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(GOLD));
    let inner = outer.inner(area);
    outer.render(area, buf);
    let chunks = Layout::vertical([
        Constraint::Min(1),
        Constraint::Length(1),
        Constraint::Length(2),
    ])
    .split(inner);
    let panels = Layout::horizontal([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(chunks[0]);
    let list = QuestId::ALL
        .iter()
        .enumerate()
        .map(|(idx, quest_id)| {
            let def = quest_def(quest_id.id()).unwrap();
            let status = if app.world_state.has_completed(*quest_id) {
                "Complete"
            } else if app.world_state.active_quest(*quest_id).is_some() {
                "Active"
            } else {
                "Available"
            };
            let style = if idx == app.quest_cursor {
                selected_style()
            } else {
                normal_style()
            };
            Line::from(Span::styled(format!("{} [{}]", def.name, status), style))
        })
        .collect::<Vec<_>>();
    Paragraph::new(list)
        .block(
            Block::bordered()
                .title(" Contracts ")
                .border_type(BorderType::Rounded)
                .style(dim_style()),
        )
        .render(panels[0], buf);
    let quest_id = QuestId::ALL[app.quest_cursor];
    let quest = quest_def(quest_id.id()).unwrap();
    let mut detail = vec![
        Line::from(Span::styled(
            quest.name,
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(quest.summary),
        Line::from(format!("Given by {}", quest.giver)),
        Line::from(""),
        Line::from("Objectives:"),
    ];
    for objective in quest.objectives {
        detail.push(Line::from(format!("• {}", objective.text)));
    }
    detail.push(Line::from(""));
    detail.push(Line::from(format!(
        "Rewards: {} XP, {} gold",
        quest.rewards.xp, quest.rewards.gold
    )));
    if let Some(item) = quest.rewards.item_type.and_then(find_def) {
        detail.push(Line::from(format!(
            "Bonus item: {} x{}",
            item.name, quest.rewards.item_qty
        )));
    }
    if !app.world_state.has_completed(quest_id) && app.world_state.active_quest(quest_id).is_none()
    {
        detail.push(Line::from(""));
        detail.push(Line::from("Enter: accept quest"));
    }
    Paragraph::new(detail)
        .block(
            Block::bordered()
                .title(" Quest Detail ")
                .border_type(BorderType::Rounded)
                .style(dim_style()),
        )
        .wrap(ratatui::widgets::Wrap { trim: true })
        .render(panels[1], buf);
    if let Some(msg) = &app.status_message {
        Paragraph::new(Span::styled(msg, Style::default().fg(Color::Green)))
            .alignment(Alignment::Center)
            .render(chunks[1], buf);
    }
    hint_bar(
        "↑ ↓ choose quest    Enter accept    Esc return",
        chunks[2],
        buf,
    );
}

pub fn render_achievements(app: &App, area: Rect, buf: &mut Buffer) {
    let outer = Block::bordered()
        .title(" Achievements ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(GOLD));
    let inner = outer.inner(area);
    outer.render(area, buf);
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(1),
        Constraint::Length(2),
    ])
    .split(inner);
    render_status_bar(app, chunks[0], buf);
    let panels = Layout::horizontal([Constraint::Percentage(44), Constraint::Percentage(56)])
        .split(chunks[1]);
    let defs = achievement_defs();
    let selected_idx = app.achievement_cursor.min(defs.len().saturating_sub(1));
    let visible_rows = panels[0].height.saturating_sub(2) as usize;
    let scroll_offset = if visible_rows == 0 {
        0
    } else {
        selected_idx.saturating_sub(visible_rows.saturating_sub(1))
    };
    let list = defs
        .iter()
        .enumerate()
        .map(|(idx, achievement)| {
            let unlocked = app.achievements.is_unlocked(&achievement.id);
            let style = if idx == selected_idx {
                selected_style()
            } else if unlocked {
                Style::default().fg(Color::Green)
            } else {
                normal_style()
            };
            let marker = if unlocked { "✓" } else { " " };
            Line::from(Span::styled(
                format!("{marker} {}", achievement.name),
                style,
            ))
        })
        .collect::<Vec<_>>();
    Paragraph::new(list)
        .block(
            Block::bordered()
                .title(" Milestones ")
                .border_type(BorderType::Rounded)
                .style(dim_style()),
        )
        .scroll((scroll_offset as u16, 0))
        .render(panels[0], buf);

    let detail = defs
        .get(selected_idx)
        .map(|achievement| {
            let unlocked = app.achievements.is_unlocked(&achievement.id);
            let progress = app.achievements.progress_toward(&achievement.id);
            vec![
                Line::from(Span::styled(
                    achievement.name.clone(),
                    Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(format!("Category: {}", achievement.category)),
                Line::from(if unlocked {
                    "Status: Unlocked".to_string()
                } else {
                    "Status: Locked".to_string()
                }),
                Line::from(""),
                Line::from(achievement.description.clone()),
                Line::from(""),
                Line::from(format!("Progress: {progress}/{}", achievement.target)),
                Line::from(Span::styled(
                    progress_bar(progress as f64 / achievement.target.max(1) as f64, 28),
                    if unlocked {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Cyan)
                    },
                )),
                Line::from(""),
                Line::from(format!(
                    "Unlocked: {}/{}",
                    app.achievements.unlocked_count(),
                    defs.len()
                )),
            ]
        })
        .unwrap_or_else(|| vec![Line::from("No achievement selected.")]);
    Paragraph::new(detail)
        .block(
            Block::bordered()
                .title(" Detail ")
                .border_type(BorderType::Rounded)
                .style(dim_style()),
        )
        .wrap(ratatui::widgets::Wrap { trim: true })
        .render(panels[1], buf);
    hint_bar("↑ ↓ browse achievements    Esc return", chunks[2], buf);
}

pub fn render_shop(app: &App, area: Rect, buf: &mut Buffer) {
    let Some(ch) = &app.active_character else {
        render_placeholder("Shop", "No active character.", area, buf);
        return;
    };
    let outer = Block::bordered()
        .title(" Vendors ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(GOLD));
    let inner = outer.inner(area);
    outer.render(area, buf);
    let chunks = Layout::vertical([
        Constraint::Length(4),
        Constraint::Min(1),
        Constraint::Length(2),
    ])
    .split(inner);
    let vendor_name = vendor_def(VendorId::ALL[app.vendor_cursor]).name;
    Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Vendor ", dim_style()),
            Span::styled(
                format!("({}/{}) ", app.vendor_cursor + 1, VendorId::ALL.len()),
                Style::default().fg(Color::Cyan),
            ),
            Span::styled("← ", dim_style()),
            Span::styled(vendor_name, Style::default().fg(GOLD)),
            Span::styled(" →  ", dim_style()),
            Span::styled(
                if app.shop_buy_mode { "[Buy]" } else { "[Sell]" },
                selected_style(),
            ),
            Span::styled(
                format!("  Gold {}", ch.gold),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(Span::styled("Use Left/Right to change vendor", dim_style())),
    ])
    .alignment(Alignment::Center)
    .render(chunks[0], buf);
    let panels = Layout::horizontal([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(chunks[1]);
    let list = if app.shop_buy_mode {
        let vendor = vendor_def(VendorId::ALL[app.vendor_cursor]);
        vendor
            .inventory
            .iter()
            .enumerate()
            .map(|(idx, entry)| {
                let def = find_def(entry.item_type).unwrap();
                let style = if idx == app.shop_cursor {
                    selected_style()
                } else {
                    normal_style()
                };
                Line::from(Span::styled(
                    format!("{}  {}g", def.name, def.base_value),
                    style,
                ))
            })
            .collect::<Vec<_>>()
    } else {
        app.inventory
            .items
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                let def = item.def().unwrap();
                let style = if idx == app.shop_cursor {
                    selected_style()
                } else {
                    normal_style()
                };
                Line::from(Span::styled(
                    format!(
                        "{} x{}  {}g",
                        def.name,
                        item.quantity,
                        (def.base_value * 40) / 100
                    ),
                    style,
                ))
            })
            .collect::<Vec<_>>()
    };
    Paragraph::new(list)
        .block(
            Block::bordered()
                .title(" Stock ")
                .border_type(BorderType::Rounded)
                .style(dim_style()),
        )
        .render(panels[0], buf);

    let detail = if app.shop_buy_mode {
        let vendor = vendor_def(VendorId::ALL[app.vendor_cursor]);
        vendor
            .inventory
            .get(
                app.shop_cursor
                    .min(vendor.inventory.len().saturating_sub(1)),
            )
            .and_then(|entry| find_def(entry.item_type))
            .map(render_item_detail)
            .unwrap_or_else(|| vec![Line::from("No item selected.")])
    } else {
        app.inventory
            .items
            .get(
                app.shop_cursor
                    .min(app.inventory.items.len().saturating_sub(1)),
            )
            .and_then(|item| item.def())
            .map(render_item_detail)
            .unwrap_or_else(|| vec![Line::from("No item selected.")])
    };
    Paragraph::new(detail)
        .block(
            Block::bordered()
                .title(" Details ")
                .border_type(BorderType::Rounded)
                .style(dim_style()),
        )
        .scroll((app.detail_scroll, 0))
        .wrap(ratatui::widgets::Wrap { trim: true })
        .render(panels[1], buf);

    if let Some(msg) = &app.status_message {
        Paragraph::new(vec![
            Line::from(Span::styled(msg, Style::default().fg(Color::Green))),
            Line::from(Span::styled(
                "Left/Right vendor    Tab buy/sell    Up/Down choose item    [ ] detail",
                dim_style(),
            )),
        ])
        .alignment(Alignment::Center)
        .render(chunks[2], buf);
    } else {
        hint_bar(
            "Left/Right vendor    Tab buy/sell    Up/Down choose item    [ ] detail    Enter transact    Esc return",
            chunks[2],
            buf,
        );
    }
}

pub fn render_dialogue(app: &App, area: Rect, buf: &mut Buffer) {
    let outer = Block::bordered()
        .title(format!(" {} ", app.dialogue_title))
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(GOLD));
    let inner = outer.inner(area);
    outer.render(area, buf);
    let chunks = Layout::vertical([Constraint::Min(1), Constraint::Length(2)]).split(inner);
    let lines = app
        .dialogue_lines
        .iter()
        .map(|line| Line::from(line.clone()))
        .collect::<Vec<_>>();
    Paragraph::new(lines)
        .alignment(Alignment::Left)
        .wrap(ratatui::widgets::Wrap { trim: false })
        .render(chunks[0], buf);
    hint_bar("Enter continue", chunks[1], buf);
}
