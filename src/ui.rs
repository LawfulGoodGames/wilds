use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph, Widget},
};

use crate::achievements::achievement_defs;
use crate::app::{
    App, CharacterTab, MenuItem, Screen, TownAction, active_level_progress, active_xp_to_next,
};
use crate::character::{Class, CreationStep, GearPackage, Race, STAT_FULL, STAT_LABELS};
use crate::combat::{ActionTab, TurnRef, ability_def, encounter_def};
use crate::inventory::{EquipSlot, find_def, gear_package_items};
use crate::world::{AreaId, QuestId, VendorId, area_def, quest_def, vendor_def};

const TITLE: &str = r"
 __        _____ _     ____  ____
 \ \      / /_ _| |   |  _ \/ ___|
  \ \ /\ / / | || |   | | | \___ \
   \ V  V /  | || |___| |_| |___) |
    \_/\_/  |___|_____|____/|____/
";

const GOLD: Color = Color::Yellow;
const DIM: Color = Color::Gray;
const TEXT: Color = Color::White;
const HIGHLIGHT: Color = Color::Black;

fn selected_style() -> Style {
    Style::default()
        .fg(HIGHLIGHT)
        .bg(GOLD)
        .add_modifier(Modifier::BOLD)
}

fn normal_style() -> Style {
    Style::default().fg(TEXT)
}

fn dim_style() -> Style {
    Style::default().fg(DIM)
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.screen {
            Screen::MainMenu => render_main_menu(self, area, buf),
            Screen::CharacterCreation => render_character_creation(self, area, buf),
            Screen::LoadGame => render_load_game(self, area, buf),
            Screen::Options => render_options(self, area, buf),
            Screen::Town => render_town(self, area, buf),
            Screen::Explore => render_explore(self, area, buf),
            Screen::CharacterSheet => render_character_sheet(self, area, buf),
            Screen::Inventory => render_inventory(self, area, buf),
            Screen::Equipment => render_equipment(self, area, buf),
            Screen::Quests => render_quests(self, area, buf),
            Screen::Achievements => render_achievements(self, area, buf),
            Screen::Shop => render_shop(self, area, buf),
            Screen::Dialogue => render_dialogue(self, area, buf),
            Screen::Combat => render_combat(self, area, buf),
        }
    }
}

fn render_main_menu(app: &App, area: Rect, buf: &mut Buffer) {
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

fn render_character_creation(app: &App, area: Rect, buf: &mut Buffer) {
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
        CreationStep::Stats => render_creation_stats(app, chunks[2], chunks[3], buf),
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
            Span::styled("Primary stats: ", dim_style()),
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

fn render_creation_stats(app: &App, content: Rect, hint: Rect, buf: &mut Buffer) {
    let c = &app.creation;
    let bonuses = c.selected_race().stat_bonuses();
    let chunks = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).split(content);
    Paragraph::new(Line::from(vec![
        Span::styled("Points remaining: ", dim_style()),
        Span::styled(
            c.points_remaining.to_string(),
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  (start at 8, cap 13)", dim_style()),
    ]))
    .alignment(Alignment::Center)
    .render(chunks[0], buf);

    let lines = (0..6)
        .map(|idx| {
            let base = c.base_stats.get(idx);
            let bonus = bonuses.get(idx);
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
                        STAT_LABELS[idx], STAT_FULL[idx], base
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
                .title(" Allocate Stats ")
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
        Line::from(format!(
            "STR {}  DEX {}  CON {}",
            stats.strength, stats.dexterity, stats.constitution
        )),
        Line::from(format!(
            "INT {}  WIS {}  CHA {}",
            stats.intelligence, stats.wisdom, stats.charisma
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

fn render_options(app: &App, area: Rect, buf: &mut Buffer) {
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

fn render_load_game(app: &App, area: Rect, buf: &mut Buffer) {
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

fn render_town(app: &App, area: Rect, buf: &mut Buffer) {
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

fn render_explore(app: &App, area: Rect, buf: &mut Buffer) {
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

fn render_character_sheet(app: &App, area: Rect, buf: &mut Buffer) {
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
        CharacterTab::Attributes => {
            let lines = vec![
                Line::from(format!(
                    "Level {}  XP {}  Next {}",
                    ch.level,
                    ch.xp,
                    active_xp_to_next(app)
                )),
                Line::from(format!("Unspent stat points: {}", ch.unspent_stat_points)),
                Line::from(""),
                Line::from(format!(
                    "STR {:>2}  DEX {:>2}  CON {:>2}",
                    ch.stats.strength, ch.stats.dexterity, ch.stats.constitution
                )),
                Line::from(format!(
                    "INT {:>2}  WIS {:>2}  CHA {:>2}",
                    ch.stats.intelligence, ch.stats.wisdom, ch.stats.charisma
                )),
                Line::from(""),
                Line::from(Span::styled(
                    progress_bar(active_level_progress(app), 32),
                    Style::default().fg(Color::Cyan),
                )),
            ];
            Paragraph::new(lines)
                .block(
                    Block::bordered()
                        .title(" Attributes ")
                        .border_type(BorderType::Rounded)
                        .style(dim_style()),
                )
                .render(chunks[2], buf);
        }
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
            let lines = ch
                .proficiencies
                .iter()
                .enumerate()
                .map(|(idx, skill)| {
                    let style = if idx
                        == app
                            .character_cursor
                            .min(ch.proficiencies.len().saturating_sub(1))
                    {
                        selected_style()
                    } else {
                        normal_style()
                    };
                    Line::from(Span::styled(
                        format!("{}  Rank {:>2}", skill.kind.name(), skill.level()),
                        style,
                    ))
                })
                .collect::<Vec<_>>();
            let panels =
                Layout::horizontal([Constraint::Percentage(42), Constraint::Percentage(58)])
                    .split(chunks[2]);
            Paragraph::new(lines)
                .block(
                    Block::bordered()
                        .title(" Proficiencies ")
                        .border_type(BorderType::Rounded)
                        .style(dim_style()),
                )
                .render(panels[0], buf);

            let detail = ch
                .proficiencies
                .get(
                    app.character_cursor
                        .min(ch.proficiencies.len().saturating_sub(1)),
                )
                .map(|skill| {
                    let plan = crate::character::study_plan(skill.kind, skill.xp, &ch.stats);
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
                        Line::from(format!("Study time: {}h", plan.hours)),
                        Line::from(format!("Success chance: {}%", plan.success_chance)),
                        Line::from(format!("On success: +{} XP", plan.success_xp)),
                        Line::from(format!("On setback: +{} XP", plan.failure_xp)),
                        Line::from(format!(
                            "Governing stat: {}",
                            plan.governing_stat.full_name()
                        )),
                    ];
                    if let Some(training) = &app.active_training {
                        if training.skill == skill.kind {
                            lines.push(Line::from(""));
                            lines.push(Line::from(Span::styled(
                                "Training in progress",
                                Style::default()
                                    .fg(Color::Green)
                                    .add_modifier(Modifier::BOLD),
                            )));
                            lines.push(Line::from(Span::styled(
                                progress_bar(training.progress(), 28),
                                Style::default().fg(Color::Green),
                            )));
                            lines.push(Line::from(format!(
                                "Focus remaining: {:.1}s",
                                (training.total_ticks.saturating_sub(training.elapsed_ticks))
                                    as f64
                                    / 30.0
                            )));
                            lines.push(Line::from(format!(
                                "Study hours: {}/{}",
                                ((training.progress() * training.hours as f64).floor() as i32)
                                    .min(training.hours),
                                training.hours
                            )));
                        }
                    }
                    if let Some((trained_skill, rank)) = app.recent_training_level_up {
                        if trained_skill == skill.kind {
                            lines.push(Line::from(""));
                            lines.push(Line::from(Span::styled(
                                format!("LEVEL UP! {} reached Rank {}", skill.kind.name(), rank),
                                Style::default()
                                    .fg(Color::Yellow)
                                    .add_modifier(Modifier::BOLD),
                            )));
                        }
                    }
                    lines.push(Line::from(""));
                    lines.push(Line::from(
                        app.status_message
                            .as_deref()
                            .unwrap_or("Choose a proficiency and train to improve it."),
                    ));
                    lines
                })
                .unwrap_or_else(|| vec![Line::from("No proficiency selected.")]);
            Paragraph::new(detail)
                .block(
                    Block::bordered()
                        .title(" Study ")
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
        if app.active_training.is_some() {
            "← → / Tab change tab    ↑ ↓ browse    Training in progress...    Esc return"
        } else {
            "← → / Tab change tab    ↑ ↓ browse    Enter or t train    Esc return"
        }
    } else {
        "← → / Tab change tab    ↑ ↓ browse    Esc return"
    };
    hint_bar(hint, chunks[3], buf);
}

fn render_inventory(app: &App, area: Rect, buf: &mut Buffer) {
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
                    "Equip slot: {}",
                    def.equip_slot.map(|slot| slot.label()).unwrap_or("None")
                )));
            }
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
        .wrap(ratatui::widgets::Wrap { trim: false })
        .render(panels[1], buf);
    if let Some(msg) = &app.inventory.last_use_message {
        Paragraph::new(Span::styled(msg, Style::default().fg(Color::Green)))
            .alignment(Alignment::Center)
            .render(chunks[1], buf);
    }
    hint_bar(
        "↑ ↓ choose    Enter use    Right/e equip    Esc return",
        chunks[2],
        buf,
    );
}

fn render_equipment(app: &App, area: Rect, buf: &mut Buffer) {
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
        Some(def) => vec![
            Line::from(Span::styled(
                def.name,
                Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(def.description),
            Line::from(""),
            Line::from(format!(
                "Armor {}  Attack {}  Spell {}",
                def.equipment_stats.armor,
                def.equipment_stats.attack_bonus,
                def.equipment_stats.spell_power
            )),
            Line::from(""),
            Line::from("Enter: unequip"),
        ],
    };
    Paragraph::new(detail)
        .block(
            Block::bordered()
                .title(" Details ")
                .border_type(BorderType::Rounded)
                .style(dim_style()),
        )
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
        "↑ ↓ choose slot    Enter unequip    Esc return",
        chunks[2],
        buf,
    );
}

fn render_quests(app: &App, area: Rect, buf: &mut Buffer) {
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

fn render_achievements(app: &App, area: Rect, buf: &mut Buffer) {
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

fn render_shop(app: &App, area: Rect, buf: &mut Buffer) {
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
        Constraint::Length(3),
        Constraint::Min(1),
        Constraint::Length(2),
    ])
    .split(inner);
    Paragraph::new(Line::from(vec![
        Span::styled(
            format!(
                "Vendor: {}  ",
                vendor_def(VendorId::ALL[app.vendor_cursor]).name
            ),
            Style::default().fg(GOLD),
        ),
        Span::styled(
            if app.shop_buy_mode { "[Buy]" } else { "[Sell]" },
            selected_style(),
        ),
        Span::styled(
            format!("  Gold {}", ch.gold),
            Style::default().fg(Color::Cyan),
        ),
    ]))
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
        .wrap(ratatui::widgets::Wrap { trim: true })
        .render(panels[1], buf);

    if let Some(msg) = &app.status_message {
        Paragraph::new(Span::styled(msg, Style::default().fg(Color::Green)))
            .alignment(Alignment::Center)
            .render(chunks[2], buf);
    } else {
        hint_bar(
            "← → change vendor    Tab buy/sell    ↑ ↓ choose item    Enter transact    Esc return",
            chunks[2],
            buf,
        );
    }
}

fn render_dialogue(app: &App, area: Rect, buf: &mut Buffer) {
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

fn render_combat(app: &App, area: Rect, buf: &mut Buffer) {
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
                actions.push(Line::from(Span::styled(
                    format!(
                        "{}  Hit +{}  Dmg {}",
                        attack.name,
                        attack.accuracy_bonus,
                        attack.damage_range_label()
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
                actions.push(Line::from(Span::styled(ability_name.to_string(), style)));
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
        "1 weapon  2 ability  3 item    ↑ ↓ choose    Tab target    Enter use    d defend    f flee",
        footer[1],
        buf,
    );
}

fn render_status_bar(app: &App, area: Rect, buf: &mut Buffer) {
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

fn render_item_detail(def: &crate::inventory::ItemDef) -> Vec<Line<'static>> {
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

fn render_placeholder(title: &str, message: &str, area: Rect, buf: &mut Buffer) {
    let outer = Block::bordered()
        .title(format!(" {title} "))
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(GOLD));
    let inner = outer.inner(area);
    outer.render(area, buf);
    render_centered(inner, buf, message);
}

fn render_centered(area: Rect, buf: &mut Buffer, message: &str) {
    Paragraph::new(message)
        .alignment(Alignment::Center)
        .render(area, buf);
}

fn hint_bar(text: &str, area: Rect, buf: &mut Buffer) {
    Paragraph::new(text)
        .style(dim_style())
        .alignment(Alignment::Center)
        .render(area, buf);
}

fn stat_bar(filled: i32, max: i32) -> String {
    let filled = filled.max(0) as usize;
    let empty = (max - filled as i32).max(0) as usize;
    format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
}

fn progress_bar(pct: f64, width: usize) -> String {
    let filled = (pct * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!(
        "[{}{}] {:>3}%",
        "█".repeat(filled),
        "░".repeat(empty),
        (pct * 100.0) as i32
    )
}

fn resource_bar(
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
