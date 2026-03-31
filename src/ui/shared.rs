use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Clear, Paragraph, Widget},
};

use crate::app::App;
use crate::inventory::{ItemDef, WeaponKind};

pub const GOLD: Color = Color::Yellow;
const DIM: Color = Color::Gray;
const TEXT: Color = Color::White;
const HIGHLIGHT: Color = Color::Black;

pub fn selected_style() -> Style {
    Style::default()
        .fg(HIGHLIGHT)
        .bg(GOLD)
        .add_modifier(Modifier::BOLD)
}

pub fn normal_style() -> Style {
    Style::default().fg(TEXT)
}

pub fn dim_style() -> Style {
    Style::default().fg(DIM)
}

pub fn render_status_bar(app: &App, area: Rect, buf: &mut Buffer) {
    let Some(ch) = &app.active_character else {
        return;
    };
    let spans = vec![
        Span::styled(
            format!("{} ", ch.name),
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("Lv.{} {}  ", ch.level, ch.class.name()),
            normal_style(),
        ),
        Span::styled(
            format!("HP {}/{}  ", ch.resources.hp, ch.resources.max_hp),
            Style::default().fg(Color::Red),
        ),
        Span::styled(
            format!("Mana {}/{}  ", ch.resources.mana, ch.resources.max_mana),
            Style::default().fg(Color::Blue),
        ),
        Span::styled(
            format!(
                "Stamina {}/{}  ",
                ch.resources.stamina, ch.resources.max_stamina
            ),
            Style::default().fg(Color::Green),
        ),
        Span::styled(format!("Gold {}  ", ch.gold), Style::default().fg(GOLD)),
        Span::styled(app.world_state.time_label(), dim_style()),
    ];
    let text = Line::from(spans);
    Paragraph::new(text)
        .alignment(Alignment::Center)
        .render(area, buf);
}

pub fn render_item_detail(def: &ItemDef) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::from(Span::styled(
            def.name,
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(def.description),
        Line::from(""),
        Line::from(format!("[{} • {}]", def.kind.label(), def.rarity.label())),
        Line::from(format!("Value: {}", def.base_value)),
    ];
    if def.is_equippable() {
        lines.push(Line::from(format!(
            "Slot: {}",
            def.equip_slot.map(|slot| slot.label()).unwrap_or("None")
        )));
        lines.push(Line::from(format!("Type: {}", def.combat_role_label())));
        lines.push(Line::from(""));
        lines.push(Line::from("Combat impact:"));
        let mut impact_lines = vec![];
        if def.equipment_stats.armor != 0 {
            impact_lines.push(format!("Armor +{}", def.equipment_stats.armor));
        }
        if def.equipment_stats.attack_bonus != 0 {
            impact_lines.push(format!("Accuracy +{}", def.equipment_stats.attack_bonus));
        }
        if def.equipment_stats.spell_power != 0 {
            impact_lines.push(format!("Spell power +{}", def.equipment_stats.spell_power));
        }
        if def.equipment_stats.crit_bonus != 0 {
            impact_lines.push(format!("Crit +{}%", def.equipment_stats.crit_bonus));
        }
        if def.equipment_stats.initiative_bonus != 0 {
            impact_lines.push(format!(
                "Initiative +{}",
                def.equipment_stats.initiative_bonus
            ));
        }
        if def.equipment_stats.resistances.physical != 0 {
            impact_lines.push(format!(
                "Physical resist +{}",
                def.equipment_stats.resistances.physical
            ));
        }
        if def.equipment_stats.resistances.fire != 0 {
            impact_lines.push(format!(
                "Fire resist +{}",
                def.equipment_stats.resistances.fire
            ));
        }
        if def.equipment_stats.resistances.frost != 0 {
            impact_lines.push(format!(
                "Frost resist +{}",
                def.equipment_stats.resistances.frost
            ));
        }
        if def.equipment_stats.resistances.lightning != 0 {
            impact_lines.push(format!(
                "Lightning resist +{}",
                def.equipment_stats.resistances.lightning
            ));
        }
        if def.equipment_stats.resistances.poison != 0 {
            impact_lines.push(format!(
                "Poison resist +{}",
                def.equipment_stats.resistances.poison
            ));
        }
        if def.equipment_stats.resistances.holy != 0 {
            impact_lines.push(format!(
                "Holy resist +{}",
                def.equipment_stats.resistances.holy
            ));
        }
        if def.equipment_stats.resistances.shadow != 0 {
            impact_lines.push(format!(
                "Shadow resist +{}",
                def.equipment_stats.resistances.shadow
            ));
        }
        if impact_lines.is_empty() {
            lines.push(Line::from("No passive stat bonuses."));
        } else {
            lines.extend(impact_lines.into_iter().map(Line::from));
        }
        if let Some(kind) = def.weapon_kind {
            lines.push(Line::from(""));
            lines.push(Line::from("Weapon behavior:"));
            lines.push(Line::from(match kind {
                WeaponKind::Melee => {
                    "Uses melee accuracy and physical damage with free weapon swings."
                }
                WeaponKind::Ranged => {
                    "Uses ranged accuracy and physical damage with free weapon shots."
                }
                WeaponKind::Magic => {
                    "Uses magic accuracy, boosts spell damage, and weapon attacks spend mana."
                }
            }));
        }
        if !def.attacks.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from("Attacks:"));
            for attack in def.attacks {
                let cost_suffix = if matches!(def.weapon_kind, Some(WeaponKind::Magic)) {
                    let cost = ((attack.min_damage + attack.max_damage) / 2 / 3).clamp(2, 5);
                    format!("  Mana {cost}")
                } else {
                    String::new()
                };
                lines.push(Line::from(format!(
                    "{}  Hit +{}  Dmg {}-{}{}",
                    attack.name,
                    attack.accuracy_bonus,
                    attack.min_damage,
                    attack.max_damage,
                    cost_suffix
                )));
            }
        }
    }
    lines
}

pub fn render_placeholder(title: &str, message: &str, area: Rect, buf: &mut Buffer) {
    let outer = Block::bordered()
        .title(format!(" {title} "))
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(GOLD));
    let inner = outer.inner(area);
    outer.render(area, buf);
    render_centered(inner, buf, message);
}

pub fn render_centered(area: Rect, buf: &mut Buffer, message: &str) {
    Paragraph::new(message)
        .alignment(Alignment::Center)
        .render(area, buf);
}

pub fn hint_bar(text: &str, area: Rect, buf: &mut Buffer) {
    Paragraph::new(text)
        .style(dim_style())
        .alignment(Alignment::Center)
        .render(area, buf);
}

pub fn render_audio_wave(app: &App, area: Rect, buf: &mut Buffer) {
    if !app.dialogue_audio.is_playing() || area.width < 18 || area.height < 6 {
        return;
    }

    let width = 18;
    let height = 5;
    let wave_area = Rect {
        x: area.x + area.width.saturating_sub(width + 1),
        y: area.y + area.height.saturating_sub(height + 1),
        width,
        height,
    };

    let frames = [
        "▁▃▅▇█▇▅▃",
        "▃▅▇█▇▅▃▁",
        "▅▇█▇▅▃▁▃",
        "▇█▇▅▃▁▃▅",
        "█▇▅▃▁▃▅▇",
        "▇▅▃▁▃▅▇█",
        "▅▃▁▃▅▇█▇",
        "▃▁▃▅▇█▇▅",
    ];
    let wave = frames[app.audio_wave_phase as usize % frames.len()];

    Clear.render(wave_area, buf);
    Paragraph::new(vec![
        Line::from(Span::styled(
            "PLAYING",
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(wave, Style::default().fg(Color::Cyan))),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::bordered()
            .title(" Audio ")
            .border_type(BorderType::Rounded)
            .style(Style::default().fg(Color::Cyan)),
    )
    .render(wave_area, buf);
}

pub fn stat_bar(filled: i32, max: i32) -> String {
    let filled = filled.max(0) as usize;
    let empty = (max - filled as i32).max(0) as usize;
    format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
}

pub fn progress_bar(pct: f64, width: usize) -> String {
    let filled = (pct * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!(
        "[{}{}] {:>3}%",
        "█".repeat(filled),
        "░".repeat(empty),
        (pct * 100.0) as i32
    )
}

pub fn resource_bar(
    label: &str,
    current: i32,
    max: i32,
    width: usize,
    fill_color: Color,
) -> Line<'static> {
    let max = max.max(1);
    let ratio = current.max(0) as f64 / max as f64;
    let filled = (ratio * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    Line::from(vec![
        Span::styled(format!("{label:<4}"), dim_style()),
        Span::raw(" "),
        Span::styled(format!("{current:>3}/{max:<3}"), normal_style()),
        Span::raw("   "),
        Span::styled("▐", Style::default().fg(fill_color)),
        Span::styled("█".repeat(filled), Style::default().fg(fill_color)),
        Span::styled("░".repeat(empty), Style::default().fg(Color::DarkGray)),
        Span::styled("▌", Style::default().fg(fill_color)),
    ])
}
