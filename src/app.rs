use crate::character::{CharacterCreation, Class, CreationStep, GearPackage, Race, SavedCharacter};
use crate::combat::{AttackKind, CombatOutcome, CombatState, PlayerAction};
use crate::event::{AppEvent, Event, EventHandler};
use crate::settings::{UserSettings, OPTIONS_COUNT};
use crate::db;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::DefaultTerminal;
use sqlx::SqlitePool;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MenuItem {
    NewGame,
    LoadGame,
    Options,
    Quit,
}

impl MenuItem {
    pub const ALL: [MenuItem; 4] = [
        MenuItem::NewGame,
        MenuItem::LoadGame,
        MenuItem::Options,
        MenuItem::Quit,
    ];

    pub fn label(&self) -> &str {
        match self {
            MenuItem::NewGame  => "New Game",
            MenuItem::LoadGame => "Load Game",
            MenuItem::Options  => "Options",
            MenuItem::Quit     => "Quit",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Screen {
    MainMenu,
    CharacterCreation,
    LoadGame,
    Options,
    InGame,
    Skills,
    Combat,
}

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub selected: usize,
    pub screen: Screen,
    // Options
    pub options_cursor: usize,
    pub settings: UserSettings,
    // Character creation
    pub creation: CharacterCreation,
    // Load game
    pub saved_characters: Vec<SavedCharacter>,
    pub load_cursor: usize,
    // Active game session
    pub active_character: Option<SavedCharacter>,
    pub combat: Option<CombatState>,
    pub minor_skills_cursor: usize,
    pool: SqlitePool,
    pub events: EventHandler,
}

impl App {
    pub fn new(pool: SqlitePool, settings: UserSettings) -> Self {
        Self {
            running: true,
            selected: 0,
            screen: Screen::MainMenu,
            options_cursor: 0,
            settings,
            creation: CharacterCreation::default(),
            saved_characters: Vec::new(),
            load_cursor: 0,
            active_character: None,
            combat: None,
            minor_skills_cursor: 0,
            pool,
            events: EventHandler::new(),
        }
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            match self.events.next().await? {
                Event::Tick => {}
                Event::Crossterm(event) => {
                    if let crossterm::event::Event::Key(key_event) = event {
                        if key_event.kind == crossterm::event::KeyEventKind::Press {
                            self.handle_key_events(key_event)?;
                        }
                    }
                }
                Event::App(app_event) => match app_event {
                    AppEvent::SelectUp   => self.select_up(),
                    AppEvent::SelectDown => self.select_down(),
                    AppEvent::Confirm    => self.confirm().await?,
                    AppEvent::Back       => self.go_back().await?,
                    AppEvent::Left       => self.handle_left(),
                    AppEvent::Right      => self.handle_right(),
                    AppEvent::OpenSkills => { self.minor_skills_cursor = 0; self.screen = Screen::Skills; }
                    AppEvent::StartCombat => self.start_combat(),
                    AppEvent::CombatSelectMelee => self.set_combat_attack_kind(AttackKind::Melee),
                    AppEvent::CombatSelectRanged => self.set_combat_attack_kind(AttackKind::Ranged),
                    AppEvent::CombatSelectSpell => self.set_combat_attack_kind(AttackKind::Spell),
                    AppEvent::CombatCycleOptionUp => self.cycle_combat_option(-1),
                    AppEvent::CombatCycleOptionDown => self.cycle_combat_option(1),
                    AppEvent::CombatUseSelected => self.handle_combat_action(PlayerAction::UseSelectedAttack).await?,
                    AppEvent::CombatDefend => self.handle_combat_action(PlayerAction::Defend).await?,
                    AppEvent::CombatFlee => self.handle_combat_action(PlayerAction::Flee).await?,
                    AppEvent::Quit       => self.quit(),
                },
            }
        }
        Ok(())
    }

    // ── Input routing ─────────────────────────────────────────────────────────

    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        if key_event.modifiers == KeyModifiers::CONTROL
            && matches!(key_event.code, KeyCode::Char('c' | 'C'))
        {
            self.events.send(AppEvent::Quit);
            return Ok(());
        }

        match self.screen {
            Screen::MainMenu => match key_event.code {
                KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Quit),
                KeyCode::Up   | KeyCode::Char('k') => self.events.send(AppEvent::SelectUp),
                KeyCode::Down | KeyCode::Char('j') => self.events.send(AppEvent::SelectDown),
                KeyCode::Enter => self.events.send(AppEvent::Confirm),
                _ => {}
            },

            Screen::CharacterCreation => match self.creation.step {
                CreationStep::Name => match key_event.code {
                    KeyCode::Char(c) if !key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                        if self.creation.name.len() < 24 {
                            self.creation.name.push(c);
                        }
                    }
                    KeyCode::Backspace => { self.creation.name.pop(); }
                    KeyCode::Enter if !self.creation.name.trim().is_empty() => {
                        self.events.send(AppEvent::Confirm);
                    }
                    KeyCode::Esc => self.events.send(AppEvent::Back),
                    _ => {}
                },
                _ => match key_event.code {
                    KeyCode::Up   | KeyCode::Char('k') => self.events.send(AppEvent::SelectUp),
                    KeyCode::Down | KeyCode::Char('j') => self.events.send(AppEvent::SelectDown),
                    KeyCode::Left  | KeyCode::Char('h') => self.events.send(AppEvent::Left),
                    KeyCode::Right | KeyCode::Char('l') => self.events.send(AppEvent::Right),
                    KeyCode::Enter => self.events.send(AppEvent::Confirm),
                    KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Back),
                    _ => {}
                },
            },

            Screen::LoadGame => match key_event.code {
                KeyCode::Up   | KeyCode::Char('k') => self.events.send(AppEvent::SelectUp),
                KeyCode::Down | KeyCode::Char('j') => self.events.send(AppEvent::SelectDown),
                KeyCode::Enter => self.events.send(AppEvent::Confirm),
                KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Back),
                _ => {}
            },

            Screen::Options => match key_event.code {
                KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Back),
                KeyCode::Up   | KeyCode::Char('k') => self.events.send(AppEvent::SelectUp),
                KeyCode::Down | KeyCode::Char('j') => self.events.send(AppEvent::SelectDown),
                KeyCode::Left  | KeyCode::Char('h') => self.events.send(AppEvent::Left),
                KeyCode::Right | KeyCode::Char('l') => self.events.send(AppEvent::Right),
                _ => {}
            },

            Screen::InGame => match key_event.code {
                KeyCode::Char('s') => self.events.send(AppEvent::OpenSkills),
                KeyCode::Char('f') => self.events.send(AppEvent::StartCombat),
                KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Back),
                _ => {}
            },

            Screen::Combat => match key_event.code {
                KeyCode::Char('1') | KeyCode::Char('m') => self.events.send(AppEvent::CombatSelectMelee),
                KeyCode::Char('2') | KeyCode::Char('r') => self.events.send(AppEvent::CombatSelectRanged),
                KeyCode::Char('3') | KeyCode::Char('c') => self.events.send(AppEvent::CombatSelectSpell),
                KeyCode::Up | KeyCode::Char('k') => self.events.send(AppEvent::CombatCycleOptionUp),
                KeyCode::Down | KeyCode::Char('j') => self.events.send(AppEvent::CombatCycleOptionDown),
                KeyCode::Enter | KeyCode::Char('a') => self.events.send(AppEvent::CombatUseSelected),
                KeyCode::Char('d') => self.events.send(AppEvent::CombatDefend),
                KeyCode::Char('f') => self.events.send(AppEvent::CombatFlee),
                KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::CombatFlee),
                _ => {}
            },

            Screen::Skills => match key_event.code {
                KeyCode::Up   | KeyCode::Char('k') => self.events.send(AppEvent::SelectUp),
                KeyCode::Down | KeyCode::Char('j') => self.events.send(AppEvent::SelectDown),
                KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Back),
                _ => {}
            },
        }
        Ok(())
    }

    // ── Navigation ────────────────────────────────────────────────────────────

    fn select_up(&mut self) {
        match self.screen {
            Screen::MainMenu  => cycle_cursor(&mut self.selected, -1, MenuItem::ALL.len()),
            Screen::Options   => cycle_cursor(&mut self.options_cursor, -1, OPTIONS_COUNT),
            Screen::Skills    => {
                if let Some(ch) = &self.active_character {
                    cycle_cursor(&mut self.minor_skills_cursor, -1, ch.minor_skills.len());
                }
            }
            Screen::LoadGame  => {
                if !self.saved_characters.is_empty() {
                    cycle_cursor(&mut self.load_cursor, -1, self.saved_characters.len());
                }
            }
            Screen::CharacterCreation => match self.creation.step {
                CreationStep::Race  => cycle_cursor(&mut self.creation.race_cursor,  -1, Race::ALL.len()),
                CreationStep::Class => cycle_cursor(&mut self.creation.class_cursor, -1, Class::ALL.len()),
                CreationStep::Stats => cycle_cursor(&mut self.creation.stat_cursor,  -1, 6),
                CreationStep::Gear  => cycle_cursor(&mut self.creation.gear_cursor,  -1, GearPackage::ALL.len()),
                _ => {}
            },
            _ => {}
        }
    }

    fn select_down(&mut self) {
        match self.screen {
            Screen::MainMenu  => cycle_cursor(&mut self.selected, 1, MenuItem::ALL.len()),
            Screen::Options   => cycle_cursor(&mut self.options_cursor, 1, OPTIONS_COUNT),
            Screen::Skills    => {
                if let Some(ch) = &self.active_character {
                    cycle_cursor(&mut self.minor_skills_cursor, 1, ch.minor_skills.len());
                }
            }
            Screen::LoadGame  => {
                if !self.saved_characters.is_empty() {
                    cycle_cursor(&mut self.load_cursor, 1, self.saved_characters.len());
                }
            }
            Screen::CharacterCreation => match self.creation.step {
                CreationStep::Race  => cycle_cursor(&mut self.creation.race_cursor,  1, Race::ALL.len()),
                CreationStep::Class => cycle_cursor(&mut self.creation.class_cursor, 1, Class::ALL.len()),
                CreationStep::Stats => cycle_cursor(&mut self.creation.stat_cursor,  1, 6),
                CreationStep::Gear  => cycle_cursor(&mut self.creation.gear_cursor,  1, GearPackage::ALL.len()),
                _ => {}
            },
            _ => {}
        }
    }

    fn handle_left(&mut self) {
        match self.screen {
            Screen::Options            => self.change_option(-1),
            Screen::CharacterCreation  => self.creation.adjust_stat(-1),
            _ => {}
        }
    }

    fn handle_right(&mut self) {
        match self.screen {
            Screen::Options            => self.change_option(1),
            Screen::CharacterCreation  => self.creation.adjust_stat(1),
            _ => {}
        }
    }

    // ── Confirm / Back ────────────────────────────────────────────────────────

    async fn confirm(&mut self) -> color_eyre::Result<()> {
        match self.screen {
            Screen::MainMenu => match MenuItem::ALL[self.selected] {
                MenuItem::NewGame => {
                    self.creation = CharacterCreation::default();
                    self.screen = Screen::CharacterCreation;
                }
                MenuItem::LoadGame => {
                    let chars = db::load_characters(&self.pool).await?;
                    self.saved_characters = chars;
                    self.load_cursor = 0;
                    self.screen = Screen::LoadGame;
                }
                MenuItem::Options => self.screen = Screen::Options,
                MenuItem::Quit    => self.quit(),
            },

            Screen::CharacterCreation => {
                if self.creation.step == CreationStep::Confirm {
                    // Save to DB, then load the full record back so InGame
                    // has a consistent SavedCharacter to work with.
                    let id = db::save_character(&self.pool, &self.creation).await?;
                    let character = db::load_character_by_id(&self.pool, id).await?;
                    self.active_character = Some(character);
                    self.screen = Screen::InGame;
                } else {
                    self.creation.step = self.creation.step.next();
                }
            }

            Screen::LoadGame => {
                if !self.saved_characters.is_empty() {
                    let character = self.saved_characters[self.load_cursor].clone();
                    self.active_character = Some(character);
                    self.screen = Screen::InGame;
                }
            }

            _ => {}
        }
        Ok(())
    }

    async fn go_back(&mut self) -> color_eyre::Result<()> {
        match self.screen {
            Screen::Options => {
                self.settings.save(&self.pool).await?;
                self.screen = Screen::MainMenu;
            }
            Screen::CharacterCreation => {
                if self.creation.step == CreationStep::Name {
                    self.screen = Screen::MainMenu;
                } else {
                    self.creation.step = self.creation.step.prev();
                }
            }
            Screen::Skills => {
                self.screen = Screen::InGame;
            }
            Screen::InGame => {
                self.screen = Screen::MainMenu;
            }
            Screen::Combat => {
                self.combat = None;
                self.screen = Screen::InGame;
            }
            _ => self.screen = Screen::MainMenu,
        }
        Ok(())
    }

    // ── Options value changes ─────────────────────────────────────────────────

    fn change_option(&mut self, dir: i32) {
        match self.options_cursor {
            0 => self.settings.sound_effects = !self.settings.sound_effects,
            1 => {
                if dir > 0 {
                    self.settings.music_volume = self.settings.music_volume.saturating_add(10).min(100);
                } else {
                    self.settings.music_volume = self.settings.music_volume.saturating_sub(10);
                }
            }
            2 => {
                self.settings.font_size = if dir > 0 {
                    self.settings.font_size.cycle_next()
                } else {
                    self.settings.font_size.cycle_prev()
                };
            }
            3 => {
                self.settings.color_theme = if dir > 0 {
                    self.settings.color_theme.cycle_next()
                } else {
                    self.settings.color_theme.cycle_prev()
                };
            }
            4 => self.settings.show_hints = !self.settings.show_hints,
            5 => {
                self.settings.difficulty = if dir > 0 {
                    self.settings.difficulty.cycle_next()
                } else {
                    self.settings.difficulty.cycle_prev()
                };
            }
            _ => {}
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    fn start_combat(&mut self) {
        let Some(character) = &self.active_character else {
            return;
        };
        self.combat = Some(CombatState::from_character(character));
        self.screen = Screen::Combat;
    }

    fn set_combat_attack_kind(&mut self, kind: AttackKind) {
        if let Some(combat) = self.combat.as_mut() {
            combat.set_attack_kind(kind);
        }
    }

    fn cycle_combat_option(&mut self, dir: i32) {
        if let Some(combat) = self.combat.as_mut() {
            combat.cycle_selected_option(dir);
        }
    }

    async fn handle_combat_action(&mut self, action: PlayerAction) -> color_eyre::Result<()> {
        let Some(character_snapshot) = self.active_character.clone() else {
            return Ok(());
        };
        let Some(combat) = self.combat.as_mut() else {
            return Ok(());
        };

        let outcome = combat.resolve_player_action(action, &character_snapshot);
        if !matches!(outcome, CombatOutcome::Ongoing) {
            self.finish_combat(outcome).await?;
        }
        Ok(())
    }

    async fn finish_combat(&mut self, outcome: CombatOutcome) -> color_eyre::Result<()> {
        let Some(mut character) = self.active_character.clone() else {
            self.combat = None;
            self.screen = Screen::InGame;
            return Ok(());
        };
        let player_hp = self.combat.as_ref().map(|c| c.player_hp).unwrap_or(character.hp);

        character.hp = player_hp.clamp(0, character.max_hp);

        if let CombatOutcome::Won { xp, gold } = outcome {
            character.xp += xp.max(0);
            character.gold += gold.max(0);
        }

        self.active_character = Some(character.clone());
        db::update_character_progress(&self.pool, character.id, character.hp, character.xp, character.gold)
            .await?;

        self.combat = None;
        self.screen = Screen::InGame;
        Ok(())
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn cycle_cursor(cursor: &mut usize, dir: i32, len: usize) {
    if dir > 0 {
        *cursor = (*cursor + 1) % len;
    } else if *cursor == 0 {
        *cursor = len - 1;
    } else {
        *cursor -= 1;
    }
}
