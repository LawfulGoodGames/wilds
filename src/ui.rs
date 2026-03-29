use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph, Widget},
};

use crate::app::{App, MenuItem, Screen};
use crate::character::{Class, CreationStep, GearPackage, Race, STAT_FULL, STAT_LABELS};
use crate::combat::{AttackKind, Turn};

const TITLE: &str = r"
 __        _____ _     ____  ____
 \ \      / /_ _| |   |  _ \/ ___|
  \ \ /\ / / | || |   | | | \___ \
   \ V  V /  | || |___| |_| |___) |
    \_/\_/  |___|_____|____/|____/
";

// ── Palette ───────────────────────────────────────────────────────────────────

const GOLD:      Color = Color::Yellow;
const DIM:       Color = Color::Gray;
const TEXT:      Color = Color::White;
const HIGHLIGHT: Color = Color::Black;

fn selected_style() -> Style {
    Style::default().fg(HIGHLIGHT).bg(GOLD).add_modifier(Modifier::BOLD)
}
fn normal_style() -> Style {
    Style::default().fg(TEXT)
}
fn dim_style() -> Style {
    Style::default().fg(DIM)
}

// ── Widget impl ───────────────────────────────────────────────────────────────

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.screen {
            Screen::MainMenu         => render_main_menu(self, area, buf),
            Screen::CharacterCreation => render_character_creation(self, area, buf),
            Screen::Options          => render_options(self, area, buf),
            Screen::LoadGame         => render_load_game(self, area, buf),
            Screen::Skills           => render_skills(self, area, buf),
            Screen::InGame           => render_in_game(self, area, buf),
            Screen::Combat           => render_combat(self, area, buf),
        }
    }
}

// ── Main Menu ─────────────────────────────────────────────────────────────────

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
    ]).split(inner);

    Paragraph::new(TITLE)
        .style(Style::default().fg(GOLD))
        .alignment(Alignment::Center)
        .render(chunks[0], buf);

    let lines: Vec<Line> = MenuItem::ALL.iter().enumerate().map(|(i, item)| {
        if i == app.selected {
            Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("> {} <", item.label()), selected_style()),
                Span::raw("  "),
            ])
        } else {
            Line::from(Span::styled(item.label(), normal_style()))
        }
    }).collect();

    Paragraph::new(lines).alignment(Alignment::Center).render(chunks[2], buf);

    Paragraph::new("↑ ↓ / j k  navigate    Enter  select    q  quit")
        .style(dim_style())
        .alignment(Alignment::Center)
        .render(chunks[3], buf);
}

// ── Character Creation ────────────────────────────────────────────────────────

fn render_character_creation(app: &App, area: Rect, buf: &mut Buffer) {
    let outer = Block::bordered()
        .title(" Character Creation ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(GOLD));
    let inner = outer.inner(area);
    outer.render(area, buf);

    let chunks = Layout::vertical([
        Constraint::Length(1), // step progress bar
        Constraint::Length(1), // spacer
        Constraint::Min(1),    // content
        Constraint::Length(2), // hints
    ]).split(inner);

    render_step_bar(app.creation.step, chunks[0], buf);

    match app.creation.step {
        CreationStep::Name    => render_creation_name(app, chunks[2], chunks[3], buf),
        CreationStep::Race    => render_creation_race(app, chunks[2], chunks[3], buf),
        CreationStep::Class   => render_creation_class(app, chunks[2], chunks[3], buf),
        CreationStep::Stats   => render_creation_stats(app, chunks[2], chunks[3], buf),
        CreationStep::Gear    => render_creation_gear(app, chunks[2], chunks[3], buf),
        CreationStep::Confirm => render_creation_confirm(app, chunks[2], chunks[3], buf),
    }
}

fn render_step_bar(current: CreationStep, area: Rect, buf: &mut Buffer) {
    let steps: Vec<Span> = CreationStep::ALL.iter().enumerate().flat_map(|(i, step)| {
        let label = if *step == current {
            Span::styled(format!("[{}]", step.label()), selected_style())
        } else if step.index() < current.index() {
            Span::styled(step.label().to_string(), Style::default().fg(Color::Green))
        } else {
            Span::styled(step.label().to_string(), dim_style())
        };
        if i < CreationStep::ALL.len() - 1 {
            vec![label, Span::styled(" > ", dim_style())]
        } else {
            vec![label]
        }
    }).collect();

    Paragraph::new(Line::from(steps))
        .alignment(Alignment::Center)
        .render(area, buf);
}

// Name ─────────────────────────────────────────────────────────────────────────

fn render_creation_name(app: &App, content: Rect, hint: Rect, buf: &mut Buffer) {
    let chunks = Layout::vertical([
        Constraint::Min(1),
        Constraint::Length(3),
        Constraint::Min(1),
    ]).split(content);

    let display = if app.creation.name.is_empty() {
        Span::styled("Enter your name...", dim_style())
    } else {
        Span::styled(format!("{}_", app.creation.name), Style::default().fg(GOLD).add_modifier(Modifier::BOLD))
    };

    let input_block = Block::bordered()
        .title(" Your Name ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(GOLD));

    Paragraph::new(Line::from(display))
        .block(input_block)
        .alignment(Alignment::Center)
        .render(chunks[1], buf);

    hint_bar("Type your character's name    Enter  continue    Esc  back", hint, buf);
}

// Race ─────────────────────────────────────────────────────────────────────────

fn render_creation_race(app: &App, content: Rect, hint: Rect, buf: &mut Buffer) {
    let chunks = Layout::horizontal([
        Constraint::Percentage(40),
        Constraint::Percentage(60),
    ]).split(content);

    // Left: race list
    let list_lines: Vec<Line> = Race::ALL.iter().enumerate().map(|(i, race)| {
        if i == app.creation.race_cursor {
            Line::from(Span::styled(format!("▶ {}", race.name()), selected_style()))
        } else {
            Line::from(Span::styled(format!("  {}", race.name()), normal_style()))
        }
    }).collect();
    Paragraph::new(list_lines)
        .block(Block::bordered().title(" Race ").border_type(BorderType::Rounded).style(dim_style()))
        .render(chunks[0], buf);

    // Right: description + bonuses
    let race = Race::ALL[app.creation.race_cursor];
    let detail = vec![
        Line::from(Span::styled(race.name(), Style::default().fg(GOLD).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled(race.description(), normal_style())),
        Line::from(""),
        Line::from(vec![
            Span::styled("Bonuses: ", dim_style()),
            Span::styled(race.bonus_label(), Style::default().fg(Color::Green)),
        ]),
    ];
    Paragraph::new(detail)
        .block(Block::bordered().title(" Details ").border_type(BorderType::Rounded).style(dim_style()))
        .wrap(ratatui::widgets::Wrap { trim: true })
        .render(chunks[1], buf);

    hint_bar("↑ ↓ / j k  navigate    Enter  confirm    Esc  back", hint, buf);
}

// Class ────────────────────────────────────────────────────────────────────────

fn render_creation_class(app: &App, content: Rect, hint: Rect, buf: &mut Buffer) {
    let chunks = Layout::horizontal([
        Constraint::Percentage(40),
        Constraint::Percentage(60),
    ]).split(content);

    let list_lines: Vec<Line> = Class::ALL.iter().enumerate().map(|(i, class)| {
        if i == app.creation.class_cursor {
            Line::from(Span::styled(format!("▶ {}", class.name()), selected_style()))
        } else {
            Line::from(Span::styled(format!("  {}", class.name()), normal_style()))
        }
    }).collect();
    Paragraph::new(list_lines)
        .block(Block::bordered().title(" Class ").border_type(BorderType::Rounded).style(dim_style()))
        .render(chunks[0], buf);

    let class = Class::ALL[app.creation.class_cursor];
    let detail = vec![
        Line::from(Span::styled(class.name(), Style::default().fg(GOLD).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled(class.description(), normal_style())),
        Line::from(""),
        Line::from(vec![
            Span::styled("Primary stats: ", dim_style()),
            Span::styled(class.primary_stats(), Style::default().fg(Color::Cyan)),
        ]),
    ];
    Paragraph::new(detail)
        .block(Block::bordered().title(" Details ").border_type(BorderType::Rounded).style(dim_style()))
        .wrap(ratatui::widgets::Wrap { trim: true })
        .render(chunks[1], buf);

    hint_bar("↑ ↓ / j k  navigate    Enter  confirm    Esc  back", hint, buf);
}

// Stats ────────────────────────────────────────────────────────────────────────

fn render_creation_stats(app: &App, content: Rect, hint: Rect, buf: &mut Buffer) {
    let c = &app.creation;
    let race_bonuses = c.selected_race().stat_bonuses();

    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(1),
    ]).split(content);

    // Points remaining
    let pts_color = if c.points_remaining == 0 { Color::Green } else { GOLD };
    Paragraph::new(Line::from(vec![
        Span::styled("Points remaining: ", dim_style()),
        Span::styled(c.points_remaining.to_string(), Style::default().fg(pts_color).add_modifier(Modifier::BOLD)),
        Span::styled("  (max 10 per stat)", dim_style()),
    ])).alignment(Alignment::Center).render(chunks[0], buf);

    // Stat rows
    let stat_lines: Vec<Line> = (0..6).map(|i| {
        let base  = c.base_stats.get(i);
        let bonus = race_bonuses.get(i);
        let final_val = base + bonus;
        let selected = i == c.stat_cursor;

        let label_style = if selected { selected_style() } else { normal_style() };
        let bar = stat_bar(base - 5, 5); // allocated points (0-5)

        let mut spans = vec![
            Span::styled(format!("  {:3}  {:<14}", STAT_LABELS[i], STAT_FULL[i]), label_style),
            Span::styled(format!(" {:2} ", base), label_style),
            Span::styled(bar, Style::default().fg(Color::Cyan)),
        ];

        if bonus > 0 {
            spans.push(Span::styled(format!(" +{bonus} "), Style::default().fg(Color::Green)));
            spans.push(Span::styled(format!("= {final_val}"), Style::default().fg(GOLD).add_modifier(Modifier::BOLD)));
        }

        if selected {
            let can_inc = c.points_remaining > 0 && base < 10;
            let can_dec = base > 5;
            spans.push(Span::raw("  "));
            if can_dec { spans.push(Span::styled("◄", dim_style())); } else { spans.push(Span::raw(" ")); }
            if can_inc { spans.push(Span::styled("►", dim_style())); }
        }

        Line::from(spans)
    }).collect();

    Paragraph::new(stat_lines)
        .block(Block::bordered().title(" Allocate Stats ").border_type(BorderType::Rounded).style(dim_style()))
        .render(chunks[2], buf);

    hint_bar("↑ ↓  navigate    ◄ ► / h l  adjust    Enter  continue    Esc  back", hint, buf);
}

fn stat_bar(allocated: i32, max: i32) -> String {
    let filled = allocated.max(0) as usize;
    let empty  = (max - allocated.max(0)) as usize;
    format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
}

// Gear ─────────────────────────────────────────────────────────────────────────

fn render_creation_gear(app: &App, content: Rect, hint: Rect, buf: &mut Buffer) {
    let chunks = Layout::horizontal([
        Constraint::Percentage(40),
        Constraint::Percentage(60),
    ]).split(content);

    let list_lines: Vec<Line> = GearPackage::ALL.iter().enumerate().map(|(i, pkg)| {
        if i == app.creation.gear_cursor {
            Line::from(Span::styled(format!("▶ {}", pkg.name()), selected_style()))
        } else {
            Line::from(Span::styled(format!("  {}", pkg.name()), normal_style()))
        }
    }).collect();
    Paragraph::new(list_lines)
        .block(Block::bordered().title(" Starting Gear ").border_type(BorderType::Rounded).style(dim_style()))
        .render(chunks[0], buf);

    let pkg = GearPackage::ALL[app.creation.gear_cursor];
    let detail = vec![
        Line::from(Span::styled(pkg.name(), Style::default().fg(GOLD).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled(pkg.description(), normal_style())),
        Line::from(""),
        Line::from(Span::styled("Items:", dim_style())),
        Line::from(Span::styled(pkg.items(), Style::default().fg(Color::Cyan))),
    ];
    Paragraph::new(detail)
        .block(Block::bordered().title(" Contents ").border_type(BorderType::Rounded).style(dim_style()))
        .wrap(ratatui::widgets::Wrap { trim: true })
        .render(chunks[1], buf);

    hint_bar("↑ ↓ / j k  navigate    Enter  confirm    Esc  back", hint, buf);
}

// Confirm ──────────────────────────────────────────────────────────────────────

fn render_creation_confirm(app: &App, content: Rect, hint: Rect, buf: &mut Buffer) {
    let c = &app.creation;
    let final_stats = c.final_stats();
    let race = c.selected_race();
    let class = c.selected_class();
    let gear = c.selected_gear();

    let lines = vec![
        Line::from(vec![
            Span::styled("  Name:   ", dim_style()),
            Span::styled(&c.name, Style::default().fg(GOLD).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Race:   ", dim_style()),
            Span::styled(race.name(), normal_style()),
            Span::styled(format!("  ({})", race.bonus_label()), Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled("  Class:  ", dim_style()),
            Span::styled(class.name(), normal_style()),
            Span::styled(format!("  ({})", class.primary_stats()), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(Span::styled("  Final Stats:", dim_style())),
        Line::from(vec![
            Span::raw("  "),
            stat_chip("STR", final_stats.strength),
            Span::raw("  "),
            stat_chip("DEX", final_stats.dexterity),
            Span::raw("  "),
            stat_chip("CON", final_stats.constitution),
            Span::raw("  "),
            stat_chip("INT", final_stats.intelligence),
            Span::raw("  "),
            stat_chip("WIS", final_stats.wisdom),
            Span::raw("  "),
            stat_chip("CHA", final_stats.charisma),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Gear:   ", dim_style()),
            Span::styled(gear.name(), normal_style()),
        ]),
        Line::from(vec![
            Span::styled("           ", dim_style()),
            Span::styled(gear.items(), Style::default().fg(Color::Cyan)),
        ]),
    ];

    Paragraph::new(lines)
        .block(Block::bordered().title(" Confirm Character ").border_type(BorderType::Rounded).style(Style::default().fg(GOLD)))
        .wrap(ratatui::widgets::Wrap { trim: false })
        .render(content, buf);

    hint_bar("Enter  begin your journey    Esc  go back", hint, buf);
}

fn stat_chip(label: &str, val: i32) -> Span<'static> {
    Span::styled(
        format!("{label}:{val:2}"),
        Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
    )
}

// ── Options ───────────────────────────────────────────────────────────────────

fn render_options(app: &App, area: Rect, buf: &mut Buffer) {
    let block = Block::bordered()
        .title(" Options ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(GOLD));
    let inner = block.inner(area);
    block.render(area, buf);

    let chunks = Layout::vertical([Constraint::Min(1), Constraint::Length(2)]).split(inner);

    let s = &app.settings;
    let entries: [(&str, String); 6] = [
        ("Sound Effects", bool_label(s.sound_effects).to_string()),
        ("Music Volume",  format!("{}%", s.music_volume)),
        ("Font Size",     s.font_size.label().to_string()),
        ("Color Theme",   s.color_theme.label().to_string()),
        ("Show Hints",    bool_label(s.show_hints).to_string()),
        ("Difficulty",    s.difficulty.label().to_string()),
    ];

    let lines: Vec<Line> = entries.iter().enumerate().map(|(i, (label, value))| {
        let sel = i == app.options_cursor;
        let row = if sel { selected_style() } else { normal_style() };
        let val_span = if sel {
            Span::styled(format!(" ◄ {value} ► "), row)
        } else {
            Span::styled(format!("   {value}   "), row)
        };
        Line::from(vec![
            Span::styled(format!("  {label:<20}", label = label), row),
            val_span,
        ])
    }).collect();

    Paragraph::new(lines).render(chunks[0], buf);
    hint_bar("↑ ↓ / j k  navigate    ◄ ► / h l  change    Esc  save & back", chunks[1], buf);
}

// ── Load Game ─────────────────────────────────────────────────────────────────

fn render_load_game(app: &App, area: Rect, buf: &mut Buffer) {
    let outer = Block::bordered()
        .title(" Load Game ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(GOLD));
    let inner = outer.inner(area);
    outer.render(area, buf);

    if app.saved_characters.is_empty() {
        let chunks = Layout::vertical([Constraint::Min(1), Constraint::Length(2)]).split(inner);
        Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled("No saved characters found.", Style::default().fg(GOLD))),
            Line::from(""),
            Line::from(Span::styled("Start a New Game to create your first character.", dim_style())),
        ])
        .alignment(Alignment::Center)
        .render(chunks[0], buf);
        hint_bar("Esc  back", chunks[1], buf);
        return;
    }

    let chunks = Layout::horizontal([
        Constraint::Percentage(45),
        Constraint::Percentage(55),
    ]).split(inner);

    // Left: character list
    let list_lines: Vec<Line> = app.saved_characters.iter().enumerate().map(|(i, ch)| {
        let tag = format!(
            "  {:<16} {:<10} {:<10} Lv.{}",
            ch.name, ch.race, ch.class, ch.level
        );
        if i == app.load_cursor {
            Line::from(Span::styled(format!("▶{}", &tag[1..]), selected_style()))
        } else {
            Line::from(Span::styled(tag, normal_style()))
        }
    }).collect();

    Paragraph::new(list_lines)
        .block(Block::bordered().title(" Saves ").border_type(BorderType::Rounded).style(dim_style()))
        .render(chunks[0], buf);

    // Right: stats detail for highlighted character
    let ch = &app.saved_characters[app.load_cursor];
    let detail = vec![
        Line::from(Span::styled(&ch.name, Style::default().fg(GOLD).add_modifier(Modifier::BOLD))),
        Line::from(vec![
            Span::styled(format!("{} ", ch.race), normal_style()),
            Span::styled(&ch.class, Style::default().fg(Color::Cyan)),
            Span::styled(format!("  •  Level {}", ch.level), dim_style()),
        ]),
        Line::from(""),
        Line::from(vec![
            stat_chip(ch.major_skills[0].kind.short_name(), ch.major_skills[0].points), Span::raw("  "),
            stat_chip(ch.major_skills[1].kind.short_name(), ch.major_skills[1].points), Span::raw("  "),
            stat_chip(ch.major_skills[2].kind.short_name(), ch.major_skills[2].points),
        ]),
        Line::from(vec![
            stat_chip(ch.major_skills[3].kind.short_name(), ch.major_skills[3].points), Span::raw("  "),
            stat_chip(ch.major_skills[4].kind.short_name(), ch.major_skills[4].points), Span::raw("  "),
            stat_chip(ch.major_skills[5].kind.short_name(), ch.major_skills[5].points),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Gear:  ", dim_style()),
            Span::styled(&ch.gear, normal_style()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("HP {}/{}", ch.hp, ch.max_hp), Style::default().fg(Color::Red)),
            Span::raw("  "),
            Span::styled(format!("XP {}", ch.xp), Style::default().fg(Color::Cyan)),
            Span::raw("  "),
            Span::styled(format!("Gold {}", ch.gold), Style::default().fg(GOLD)),
        ]),
    ];

    let hint_area = Layout::vertical([Constraint::Min(1), Constraint::Length(2)]).split(inner);
    Paragraph::new(detail)
        .block(Block::bordered().title(" Details ").border_type(BorderType::Rounded).style(dim_style()))
        .render(chunks[1], buf);
    hint_bar("↑ ↓ / j k  navigate    Enter  load    Esc  back", hint_area[1], buf);
}

// ── In-Game (placeholder) ─────────────────────────────────────────────────────

fn render_in_game(app: &App, area: Rect, buf: &mut Buffer) {
    let Some(ch) = &app.active_character else {
        render_placeholder("Error", "No active character.", area, buf);
        return;
    };

    let outer = Block::bordered()
        .title(format!(" The Wilds — {} ", ch.name))
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(GOLD));
    let inner = outer.inner(area);
    outer.render(area, buf);

    let chunks = Layout::vertical([
        Constraint::Length(4), // character header bar
        Constraint::Min(1),    // world area (placeholder)
        Constraint::Length(2), // hint
    ]).split(inner);

    // Character status bar
    let status = vec![
        Line::from(vec![
            Span::styled(format!("  {} ", ch.name), Style::default().fg(GOLD).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{}  {}  ", ch.race, ch.class), normal_style()),
            Span::styled(format!("Lv.{}  ", ch.level), Style::default().fg(Color::Cyan)),
            Span::styled(format!("HP {}/{}  ", ch.hp, ch.max_hp), Style::default().fg(Color::Red)),
            Span::styled(format!("XP {}  ", ch.xp), Style::default().fg(Color::Cyan)),
            Span::styled(format!("Gold {}", ch.gold), Style::default().fg(GOLD)),
        ]),
        Line::from(vec![
            Span::raw("  "),
            stat_chip(ch.major_skills[0].kind.short_name(), ch.major_skills[0].points), Span::raw("  "),
            stat_chip(ch.major_skills[1].kind.short_name(), ch.major_skills[1].points), Span::raw("  "),
            stat_chip(ch.major_skills[2].kind.short_name(), ch.major_skills[2].points), Span::raw("  "),
            stat_chip(ch.major_skills[3].kind.short_name(), ch.major_skills[3].points), Span::raw("  "),
            stat_chip(ch.major_skills[4].kind.short_name(), ch.major_skills[4].points), Span::raw("  "),
            stat_chip(ch.major_skills[5].kind.short_name(), ch.major_skills[5].points),
        ]),
    ];
    Paragraph::new(status)
        .block(Block::bordered().border_type(BorderType::Rounded).style(dim_style()))
        .render(chunks[0], buf);

    // World placeholder
    let world_lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "You stand at the edge of the Wilds.",
            Style::default().fg(TEXT),
        )),
        Line::from(""),
        Line::from(Span::styled("The adventure begins here...", dim_style())),
        Line::from(""),
        Line::from(Span::styled("(Game world coming soon)", dim_style())),
    ];
    Paragraph::new(world_lines)
        .alignment(Alignment::Center)
        .render(chunks[1], buf);

    hint_bar("f  fight test encounter    s  skills    Esc  main menu", chunks[2], buf);
}

// ── Combat ────────────────────────────────────────────────────────────────────

fn render_combat(app: &App, area: Rect, buf: &mut Buffer) {
    let Some(combat) = &app.combat else {
        render_placeholder("Combat", "No active combat.", area, buf);
        return;
    };

    let outer = Block::bordered()
        .title(" Combat ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(GOLD));
    let inner = outer.inner(area);
    outer.render(area, buf);

    let chunks = Layout::vertical([
        Constraint::Length(6),
        Constraint::Min(1),
        Constraint::Length(2),
    ])
    .split(inner);

    let header = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(chunks[0]);
    let player_turn = if combat.turn == Turn::Player { "Player turn" } else { "Enemy turn" };

    let player_lines = vec![
        Line::from(Span::styled("You", Style::default().fg(GOLD).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(
            hp_bar("HP", combat.player_hp, combat.player_max_hp, 24),
            Style::default().fg(Color::Red),
        )),
        Line::from(Span::styled(player_turn, dim_style())),
    ];
    Paragraph::new(player_lines)
        .block(Block::bordered().title(" Player ").border_type(BorderType::Rounded).style(dim_style()))
        .render(header[0], buf);

    let enemy_lines = vec![
        Line::from(Span::styled(
            &combat.enemy.name,
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            hp_bar("HP", combat.enemy.hp, combat.enemy.max_hp, 24),
            Style::default().fg(Color::Red),
        )),
        Line::from(Span::styled(
            format!("Armor Class: {}", combat.enemy.armor_class),
            Style::default().fg(Color::Yellow),
        )),
        Line::from(Span::styled(
            format!("Reward: {} XP, {} Gold", combat.enemy.reward_xp, combat.enemy.reward_gold),
            dim_style(),
        )),
    ];
    Paragraph::new(enemy_lines)
        .block(Block::bordered().title(" Enemy ").border_type(BorderType::Rounded).style(dim_style()))
        .render(header[1], buf);

    let (options, selected_idx) = combat.selected_options();
    let selected_label = combat.selected_option_name();
    let mode_tabs = vec![
        tab_chip("1:Melee", combat.active_attack_kind == AttackKind::Melee),
        Span::raw(" "),
        tab_chip("2:Ranged", combat.active_attack_kind == AttackKind::Ranged),
        Span::raw(" "),
        tab_chip("3:Spell", combat.active_attack_kind == AttackKind::Spell),
    ];

    let mut selection_lines = vec![
        Line::from(mode_tabs),
        Line::from(""),
        Line::from(Span::styled(
            format!("Selected {}: {}", combat.active_attack_kind.label(), selected_label),
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
        )),
    ];
    selection_lines.extend(options.iter().enumerate().map(|(idx, option)| {
        if idx == selected_idx {
            Line::from(Span::styled(format!("▶ {}", option.name), selected_style()))
        } else {
            Line::from(Span::styled(format!("  {}", option.name), normal_style()))
        }
    }));

    let visible_start = combat.log.len().saturating_sub(5);
    let new_start = combat.log.len().saturating_sub(combat.new_log_entries);
    let log_lines: Vec<Line> = combat
        .log
        .iter()
        .enumerate()
        .skip(visible_start)
        .map(|(idx, entry)| {
            if idx >= new_start {
                Line::from(Span::styled(
                    format!("▶ {entry}"),
                    Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
                ))
            } else {
                Line::from(Span::styled(format!("• {entry}"), normal_style()))
            }
        })
        .collect();

    let lower = Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)]).split(chunks[1]);
    Paragraph::new(selection_lines)
        .block(Block::bordered().title(" Attack Select ").border_type(BorderType::Rounded).style(dim_style()))
        .wrap(ratatui::widgets::Wrap { trim: true })
        .render(lower[0], buf);

    Paragraph::new(log_lines)
        .block(Block::bordered().title(" Battle Log ").border_type(BorderType::Rounded).style(dim_style()))
        .wrap(ratatui::widgets::Wrap { trim: true })
        .render(lower[1], buf);

    let footer = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).split(chunks[2]);
    if let Some(summary) = &combat.last_roll_summary {
        let roll_line = Line::from(vec![
            Span::styled("Last Roll: ", dim_style()),
            Span::styled(summary, Style::default().fg(Color::Cyan)),
        ]);
        Paragraph::new(roll_line)
            .alignment(Alignment::Center)
            .render(footer[0], buf);
    }

    hint_bar(
        "1/2/3 select type    ↑ ↓ choose option    Enter/a attack    d defend    f flee",
        footer[1],
        buf,
    );
}

// ── Skills ────────────────────────────────────────────────────────────────────

fn render_skills(app: &App, area: Rect, buf: &mut Buffer) {
    let Some(ch) = &app.active_character else {
        render_placeholder("Skills", "No active character.", area, buf);
        return;
    };

    let outer = Block::bordered()
        .title(" Minor Skills ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(GOLD));
    let inner = outer.inner(area);
    outer.render(area, buf);

    let chunks = Layout::vertical([Constraint::Min(1), Constraint::Length(2)]).split(inner);

    let panels = Layout::horizontal([
        Constraint::Percentage(38),
        Constraint::Percentage(62),
    ]).split(chunks[0]);

    // Left: skill list
    let list_lines: Vec<Line> = ch.minor_skills.iter().enumerate().map(|(i, skill)| {
        let level = skill.level();
        let level_color = match level {
            99         => GOLD,
            71..=98    => Color::Green,
            31..=70    => Color::Cyan,
            _          => TEXT,
        };
        if i == app.minor_skills_cursor {
            Line::from(Span::styled(
                format!("▶ {:<15} Lv.{:>2}", skill.kind.name(), level),
                selected_style(),
            ))
        } else {
            Line::from(vec![
                Span::styled(format!("  {:<15} ", skill.kind.name()), normal_style()),
                Span::styled(format!("Lv.{:>2}", level), Style::default().fg(level_color)),
            ])
        }
    }).collect();

    Paragraph::new(list_lines)
        .block(Block::bordered().title(" Skill ").border_type(BorderType::Rounded).style(dim_style()))
        .render(panels[0], buf);

    // Right: detail for highlighted skill
    let skill = &ch.minor_skills[app.minor_skills_cursor];
    let level = skill.level();
    let detail_width = (panels[1].width as usize).saturating_sub(6); // inner width for bar

    let mut detail = vec![
        Line::from(Span::styled(skill.kind.name(), Style::default().fg(GOLD).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled(skill.kind.description(), dim_style())),
        Line::from(""),
    ];

    if level >= 99 {
        detail.push(Line::from(Span::styled(
            "MAX LEVEL",
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
        )));
    } else {
        let bar_width = detail_width.min(30);
        let bar = skill_progress_bar(skill.progress(), bar_width);
        detail.extend([
            Line::from(vec![
                Span::styled("Level     ", dim_style()),
                Span::styled(level.to_string(), Style::default().fg(GOLD).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("XP        ", dim_style()),
                Span::styled(format_xp(skill.xp), normal_style()),
            ]),
            Line::from(vec![
                Span::styled("Next lvl  ", dim_style()),
                Span::styled(format!("+{} XP", format_xp(skill.xp_to_next() as i32)), Style::default().fg(Color::Cyan)),
            ]),
            Line::from(""),
            Line::from(Span::styled(bar, Style::default().fg(Color::Cyan))),
        ]);
    }

    Paragraph::new(detail)
        .block(Block::bordered().title(" Detail ").border_type(BorderType::Rounded).style(dim_style()))
        .wrap(ratatui::widgets::Wrap { trim: false })
        .render(panels[1], buf);

    hint_bar("↑ ↓ / j k  navigate    Esc  back to game", chunks[1], buf);
}

fn skill_progress_bar(pct: f64, width: usize) -> String {
    let filled = (pct * width as f64).round() as usize;
    let empty  = width.saturating_sub(filled);
    format!("[{}{}] {:>3}%", "█".repeat(filled), "░".repeat(empty), (pct * 100.0) as u32)
}

fn format_xp(n: i32) -> String {
    // Formats numbers with thousands separators: 1_234_567 → "1,234,567"
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 { out.push(','); }
        out.push(ch);
    }
    out.chars().rev().collect()
}

// ── Shared helpers ────────────────────────────────────────────────────────────

fn render_placeholder(title: &str, message: &str, area: Rect, buf: &mut Buffer) {
    let block = Block::bordered()
        .title(format!(" {title} "))
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(GOLD));
    let inner = block.inner(area);
    block.render(area, buf);

    let chunks = Layout::vertical([Constraint::Min(1), Constraint::Length(2)]).split(inner);
    Paragraph::new(message).style(normal_style()).alignment(Alignment::Center).render(chunks[0], buf);
    hint_bar("Esc / q  back to menu", chunks[1], buf);
}

fn hint_bar(text: &str, area: Rect, buf: &mut Buffer) {
    Paragraph::new(text).style(dim_style()).alignment(Alignment::Center).render(area, buf);
}

fn hp_bar(label: &str, current: i32, max: i32, width: usize) -> String {
    let max = max.max(1);
    let current = current.clamp(0, max);
    let ratio = current as f64 / max as f64;
    let filled = (ratio * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("{label} [{}/{}] {}{}", current, max, "█".repeat(filled), "░".repeat(empty))
}

fn bool_label(v: bool) -> &'static str {
    if v { "On" } else { "Off" }
}

fn tab_chip(label: &str, selected: bool) -> Span<'_> {
    if selected {
        Span::styled(format!("[{label}]"), selected_style())
    } else {
        Span::styled(label.to_string(), dim_style())
    }
}
