use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph, Widget},
};

use crate::app::App;
use crate::inventory::ItemDef;

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
        lines.push(Line::from(format!(
            "Armor {}  Attack {}  Spell {}",
            def.equipment_stats.armor,
            def.equipment_stats.attack_bonus,
            def.equipment_stats.spell_power
        )));
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
