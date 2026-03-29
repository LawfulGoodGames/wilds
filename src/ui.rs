use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph, Widget},
};

use crate::app::{App, MenuItem, Screen};
use crate::character::{Class, CreationStep, GearPackage, Race, STAT_FULL, STAT_LABELS};

const TITLE: &str = r"
 __        _____ _     ____  ____
 \ \      / /_ _| |   |  _ \/ ___|
  \ \ /\ / / | || |   | | | \___ \
   \ V  V /  | || |___| |_| |___) |
    \_/\_/  |___|_____|____/|____/
";

// ── Palette ───────────────────────────────────────────────────────────────────

const GOLD:      Color = Color::Yellow;
const DIM:       Color = Color::DarkGray;
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
            Screen::InGame           => render_in_game(self, area, buf),
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
            stat_chip("STR", ch.str_stat), Span::raw("  "),
            stat_chip("DEX", ch.dex_stat), Span::raw("  "),
            stat_chip("CON", ch.con_stat),
        ]),
        Line::from(vec![
            stat_chip("INT", ch.int_stat), Span::raw("  "),
            stat_chip("WIS", ch.wis_stat), Span::raw("  "),
            stat_chip("CHA", ch.cha_stat),
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
            Span::styled(format!("{}  {}  ", ch.race, ch.class), dim_style()),
            Span::styled(format!("Lv.{}  ", ch.level), Style::default().fg(Color::Cyan)),
            Span::styled(format!("HP {}/{}  ", ch.hp, ch.max_hp), Style::default().fg(Color::Red)),
            Span::styled(format!("XP {}  ", ch.xp), Style::default().fg(Color::Cyan)),
            Span::styled(format!("Gold {}", ch.gold), Style::default().fg(GOLD)),
        ]),
        Line::from(vec![
            Span::raw("  "),
            stat_chip("STR", ch.str_stat), Span::raw("  "),
            stat_chip("DEX", ch.dex_stat), Span::raw("  "),
            stat_chip("CON", ch.con_stat), Span::raw("  "),
            stat_chip("INT", ch.int_stat), Span::raw("  "),
            stat_chip("WIS", ch.wis_stat), Span::raw("  "),
            stat_chip("CHA", ch.cha_stat),
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

    hint_bar("Esc  main menu", chunks[2], buf);
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

fn bool_label(v: bool) -> &'static str {
    if v { "On" } else { "Off" }
}
