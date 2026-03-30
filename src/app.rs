use crate::achievements::AchievementState;
use crate::character::{
    CharacterCreation, Class, CreationStep, GearPackage, MAX_PROFICIENCY_LEVEL, MinorSkill, Race,
    SavedCharacter, level_progress_pct, study_plan, xp_to_next_level,
};
use crate::combat::{ActionTab, CombatOutcome, CombatState, PlayerAction, ability_def};
use crate::db;
use crate::event::{AppEvent, Event, EventHandler};
use crate::inventory::{EquipSlot, Equipment, InventoryState, ItemEffect, find_def};
use crate::settings::{OPTIONS_COUNT, UserSettings};
use crate::world::{
    AreaId, ObjectiveKind, QuestId, VendorId, WorldState, area_def, quest_def, vendor_def,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rand::RngExt;
use ratatui::DefaultTerminal;
use sqlx::SqlitePool;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    pub fn label(self) -> &'static str {
        match self {
            Self::NewGame => "New Game",
            Self::LoadGame => "Load Game",
            Self::Options => "Options",
            Self::Quit => "Quit",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    MainMenu,
    CharacterCreation,
    LoadGame,
    Options,
    Town,
    Explore,
    CharacterSheet,
    Inventory,
    Equipment,
    Achievements,
    Quests,
    Shop,
    Dialogue,
    Combat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TownAction {
    Explore,
    Character,
    Inventory,
    Equipment,
    Quests,
    Achievements,
    Shop,
    Rest,
    LeaveTown,
}

impl TownAction {
    pub const ALL: [TownAction; 9] = [
        TownAction::Explore,
        TownAction::Character,
        TownAction::Inventory,
        TownAction::Equipment,
        TownAction::Quests,
        TownAction::Achievements,
        TownAction::Shop,
        TownAction::Rest,
        TownAction::LeaveTown,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Explore => "Explore the Wilds",
            Self::Character => "Character Sheet",
            Self::Inventory => "Inventory",
            Self::Equipment => "Equipment",
            Self::Quests => "Quest Log",
            Self::Achievements => "Achievements",
            Self::Shop => "Visit Vendors",
            Self::Rest => "Rest at the Inn",
            Self::LeaveTown => "Return to Main Menu",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterTab {
    Attributes,
    Abilities,
    Proficiencies,
    Equipment,
}

impl CharacterTab {
    pub const ALL: [CharacterTab; 4] = [
        CharacterTab::Attributes,
        CharacterTab::Abilities,
        CharacterTab::Proficiencies,
        CharacterTab::Equipment,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Attributes => "Attributes",
            Self::Abilities => "Abilities",
            Self::Proficiencies => "Proficiencies",
            Self::Equipment => "Equipment",
        }
    }
}

const TRAINING_TICKS_PER_HOUR: u32 = 20;

#[derive(Debug, Clone)]
pub struct ActiveTraining {
    pub skill: MinorSkill,
    pub total_ticks: u32,
    pub elapsed_ticks: u32,
    pub hours: i32,
    pub success_chance: i32,
    pub success_xp: i32,
    pub failure_xp: i32,
}

impl ActiveTraining {
    pub fn progress(&self) -> f64 {
        if self.total_ticks == 0 {
            1.0
        } else {
            (self.elapsed_ticks as f64 / self.total_ticks as f64).clamp(0.0, 1.0)
        }
    }
}

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub selected: usize,
    pub screen: Screen,
    pub options_cursor: usize,
    pub settings: UserSettings,
    pub creation: CharacterCreation,
    pub saved_characters: Vec<SavedCharacter>,
    pub load_cursor: usize,
    pub active_character: Option<SavedCharacter>,
    pub world_state: WorldState,
    pub equipment: Equipment,
    pub inventory: InventoryState,
    pub combat: Option<CombatState>,
    pub town_cursor: usize,
    pub explore_cursor: usize,
    pub quest_cursor: usize,
    pub shop_cursor: usize,
    pub vendor_cursor: usize,
    pub shop_buy_mode: bool,
    pub equipment_cursor: usize,
    pub achievement_cursor: usize,
    pub character_cursor: usize,
    pub character_tab: CharacterTab,
    pub dialogue_title: String,
    pub dialogue_lines: Vec<String>,
    pub dialogue_return: Screen,
    pub status_message: Option<String>,
    pub active_training: Option<ActiveTraining>,
    pub recent_training_level_up: Option<(MinorSkill, u32)>,
    pub achievements: AchievementState,
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
            saved_characters: vec![],
            load_cursor: 0,
            active_character: None,
            world_state: WorldState::default(),
            equipment: Equipment::default(),
            inventory: InventoryState::default(),
            combat: None,
            town_cursor: 0,
            explore_cursor: 0,
            quest_cursor: 0,
            shop_cursor: 0,
            vendor_cursor: 0,
            shop_buy_mode: true,
            equipment_cursor: 0,
            achievement_cursor: 0,
            character_cursor: 0,
            character_tab: CharacterTab::Attributes,
            dialogue_title: String::new(),
            dialogue_lines: vec![],
            dialogue_return: Screen::Town,
            status_message: None,
            active_training: None,
            recent_training_level_up: None,
            achievements: AchievementState::default(),
            pool,
            events: EventHandler::new(),
        }
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            match self.events.next().await? {
                Event::Tick => self.handle_tick().await?,
                Event::Crossterm(event) => {
                    if let crossterm::event::Event::Key(key_event) = event {
                        if key_event.kind == crossterm::event::KeyEventKind::Press {
                            self.handle_key_events(key_event)?;
                        }
                    }
                }
                Event::App(event) => self.handle_app_event(event).await?,
            }
        }
        Ok(())
    }

    async fn handle_tick(&mut self) -> color_eyre::Result<()> {
        let ready = if let Some(training) = self.active_training.as_mut() {
            training.elapsed_ticks = (training.elapsed_ticks + 1).min(training.total_ticks);
            training.elapsed_ticks >= training.total_ticks
        } else {
            false
        };

        if ready {
            self.resolve_training_completion().await?;
        }

        Ok(())
    }

    fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        if key_event.modifiers == KeyModifiers::CONTROL
            && matches!(key_event.code, KeyCode::Char('c' | 'C'))
        {
            self.events.send(AppEvent::Quit);
            return Ok(());
        }

        match self.screen {
            Screen::MainMenu => match key_event.code {
                KeyCode::Up | KeyCode::Char('k') => self.events.send(AppEvent::SelectUp),
                KeyCode::Down | KeyCode::Char('j') => self.events.send(AppEvent::SelectDown),
                KeyCode::Enter => self.events.send(AppEvent::Confirm),
                KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Quit),
                _ => {}
            },
            Screen::CharacterCreation => match self.creation.step {
                CreationStep::Name => match key_event.code {
                    KeyCode::Char(c) if !key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                        if self.creation.name.len() < 24 {
                            self.creation.name.push(c);
                        }
                    }
                    KeyCode::Backspace => {
                        self.creation.name.pop();
                    }
                    KeyCode::Enter if !self.creation.name.trim().is_empty() => {
                        self.events.send(AppEvent::Confirm)
                    }
                    KeyCode::Esc => self.events.send(AppEvent::Back),
                    _ => {}
                },
                _ => match key_event.code {
                    KeyCode::Up | KeyCode::Char('k') => self.events.send(AppEvent::SelectUp),
                    KeyCode::Down | KeyCode::Char('j') => self.events.send(AppEvent::SelectDown),
                    KeyCode::Left | KeyCode::Char('h') => self.events.send(AppEvent::Left),
                    KeyCode::Right | KeyCode::Char('l') => self.events.send(AppEvent::Right),
                    KeyCode::Enter => self.events.send(AppEvent::Confirm),
                    KeyCode::Esc => self.events.send(AppEvent::Back),
                    _ => {}
                },
            },
            Screen::LoadGame => match key_event.code {
                KeyCode::Up | KeyCode::Char('k') => self.events.send(AppEvent::SelectUp),
                KeyCode::Down | KeyCode::Char('j') => self.events.send(AppEvent::SelectDown),
                KeyCode::Enter => self.events.send(AppEvent::Confirm),
                KeyCode::Esc => self.events.send(AppEvent::Back),
                _ => {}
            },
            Screen::Options => match key_event.code {
                KeyCode::Up | KeyCode::Char('k') => self.events.send(AppEvent::SelectUp),
                KeyCode::Down | KeyCode::Char('j') => self.events.send(AppEvent::SelectDown),
                KeyCode::Left | KeyCode::Char('h') => self.events.send(AppEvent::Left),
                KeyCode::Right | KeyCode::Char('l') => self.events.send(AppEvent::Right),
                KeyCode::Esc => self.events.send(AppEvent::Back),
                _ => {}
            },
            Screen::Town => match key_event.code {
                KeyCode::Up | KeyCode::Char('k') => self.events.send(AppEvent::SelectUp),
                KeyCode::Down | KeyCode::Char('j') => self.events.send(AppEvent::SelectDown),
                KeyCode::Enter => self.events.send(AppEvent::Confirm),
                KeyCode::Char('x') => self.events.send(AppEvent::OpenExplore),
                KeyCode::Char('c') => self.events.send(AppEvent::OpenCharacter),
                KeyCode::Char('i') => self.events.send(AppEvent::OpenInventory),
                KeyCode::Char('e') => self.events.send(AppEvent::OpenEquipment),
                KeyCode::Char('q') => self.events.send(AppEvent::OpenQuests),
                KeyCode::Char('h') => self.events.send(AppEvent::OpenAchievements),
                KeyCode::Char('v') => self.events.send(AppEvent::OpenShop),
                KeyCode::Char('r') => self.events.send(AppEvent::RestAtInn),
                KeyCode::Esc => self.events.send(AppEvent::Back),
                _ => {}
            },
            Screen::Explore => match key_event.code {
                KeyCode::Up | KeyCode::Char('k') => self.events.send(AppEvent::SelectUp),
                KeyCode::Down | KeyCode::Char('j') => self.events.send(AppEvent::SelectDown),
                KeyCode::Enter => self.events.send(AppEvent::ExploreSelected),
                KeyCode::Esc => self.events.send(AppEvent::Back),
                _ => {}
            },
            Screen::CharacterSheet => match key_event.code {
                KeyCode::Up | KeyCode::Char('k') => self.events.send(AppEvent::SelectUp),
                KeyCode::Down | KeyCode::Char('j') => self.events.send(AppEvent::SelectDown),
                KeyCode::Left | KeyCode::Char('h') => self.events.send(AppEvent::Left),
                KeyCode::Right | KeyCode::Char('l') => self.events.send(AppEvent::Right),
                KeyCode::Enter | KeyCode::Char('t') => self.events.send(AppEvent::Confirm),
                KeyCode::Tab => self.events.send(AppEvent::NextTab),
                KeyCode::Esc => self.events.send(AppEvent::Back),
                _ => {}
            },
            Screen::Inventory => match key_event.code {
                KeyCode::Up | KeyCode::Char('k') => self.events.send(AppEvent::SelectUp),
                KeyCode::Down | KeyCode::Char('j') => self.events.send(AppEvent::SelectDown),
                KeyCode::Enter => self.events.send(AppEvent::Confirm),
                KeyCode::Char('e') => self.events.send(AppEvent::Right),
                KeyCode::Esc => self.events.send(AppEvent::Back),
                _ => {}
            },
            Screen::Equipment => match key_event.code {
                KeyCode::Up | KeyCode::Char('k') => self.events.send(AppEvent::SelectUp),
                KeyCode::Down | KeyCode::Char('j') => self.events.send(AppEvent::SelectDown),
                KeyCode::Enter => self.events.send(AppEvent::Confirm),
                KeyCode::Esc => self.events.send(AppEvent::Back),
                _ => {}
            },
            Screen::Quests => match key_event.code {
                KeyCode::Up | KeyCode::Char('k') => self.events.send(AppEvent::SelectUp),
                KeyCode::Down | KeyCode::Char('j') => self.events.send(AppEvent::SelectDown),
                KeyCode::Enter => self.events.send(AppEvent::QuestAccept),
                KeyCode::Esc => self.events.send(AppEvent::Back),
                _ => {}
            },
            Screen::Achievements => match key_event.code {
                KeyCode::Up | KeyCode::Char('k') => self.events.send(AppEvent::SelectUp),
                KeyCode::Down | KeyCode::Char('j') => self.events.send(AppEvent::SelectDown),
                KeyCode::Esc => self.events.send(AppEvent::Back),
                _ => {}
            },
            Screen::Shop => match key_event.code {
                KeyCode::Up | KeyCode::Char('k') => self.events.send(AppEvent::SelectUp),
                KeyCode::Down | KeyCode::Char('j') => self.events.send(AppEvent::SelectDown),
                KeyCode::Enter => self.events.send(AppEvent::ShopTransaction),
                KeyCode::Tab => self.events.send(AppEvent::ShopToggleMode),
                KeyCode::Left | KeyCode::Char('h') => {
                    self.events.send(AppEvent::ShopPreviousVendor)
                }
                KeyCode::Right | KeyCode::Char('l') => self.events.send(AppEvent::ShopNextVendor),
                KeyCode::Esc => self.events.send(AppEvent::Back),
                _ => {}
            },
            Screen::Dialogue => match key_event.code {
                KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') => {
                    self.events.send(AppEvent::Back)
                }
                _ => {}
            },
            Screen::Combat => match key_event.code {
                KeyCode::Char('1') => self.events.send(AppEvent::CombatTabWeapon),
                KeyCode::Char('2') => self.events.send(AppEvent::CombatTabAbility),
                KeyCode::Char('3') => self.events.send(AppEvent::CombatTabItem),
                KeyCode::Up | KeyCode::Char('k') => self.events.send(AppEvent::CombatCycleOptionUp),
                KeyCode::Down | KeyCode::Char('j') => {
                    self.events.send(AppEvent::CombatCycleOptionDown)
                }
                KeyCode::Tab => self.events.send(AppEvent::CombatCycleTarget),
                KeyCode::Enter | KeyCode::Char('a') => {
                    self.events.send(AppEvent::CombatUseSelected)
                }
                KeyCode::Char('d') => self.events.send(AppEvent::CombatDefend),
                KeyCode::Char('f') => self.events.send(AppEvent::CombatFlee),
                KeyCode::Esc => self.events.send(AppEvent::CombatFlee),
                _ => {}
            },
        }
        Ok(())
    }

    async fn handle_app_event(&mut self, event: AppEvent) -> color_eyre::Result<()> {
        match event {
            AppEvent::SelectUp => self.select_up(),
            AppEvent::SelectDown => self.select_down(),
            AppEvent::Confirm => self.confirm().await?,
            AppEvent::Back => self.go_back().await?,
            AppEvent::Left => self.handle_left(),
            AppEvent::Right => self.handle_right().await?,
            AppEvent::NextTab => self.next_character_tab(),
            AppEvent::OpenExplore => self.open_explore(),
            AppEvent::OpenCharacter => self.open_character_sheet(),
            AppEvent::OpenInventory => self.open_inventory().await?,
            AppEvent::OpenEquipment => self.open_equipment().await?,
            AppEvent::OpenQuests => self.open_quests(),
            AppEvent::OpenAchievements => self.open_achievements(),
            AppEvent::OpenShop => self.open_shop().await?,
            AppEvent::RestAtInn => self.rest_at_inn().await?,
            AppEvent::ExploreSelected => self.explore_selected().await?,
            AppEvent::ShopToggleMode => self.toggle_shop_mode(),
            AppEvent::ShopNextVendor => self.shop_cycle_vendor(1),
            AppEvent::ShopPreviousVendor => self.shop_cycle_vendor(-1),
            AppEvent::ShopTransaction => self.shop_transaction().await?,
            AppEvent::QuestAccept => self.accept_selected_quest().await?,
            AppEvent::CombatTabWeapon => self.set_combat_tab(ActionTab::Weapon),
            AppEvent::CombatTabAbility => self.set_combat_tab(ActionTab::Ability),
            AppEvent::CombatTabItem => self.set_combat_tab(ActionTab::Item),
            AppEvent::CombatCycleOptionUp => self.cycle_combat_option(-1),
            AppEvent::CombatCycleOptionDown => self.cycle_combat_option(1),
            AppEvent::CombatCycleTarget => self.cycle_combat_target(1),
            AppEvent::CombatUseSelected => self.handle_combat_action().await?,
            AppEvent::CombatDefend => {
                self.handle_explicit_combat_action(PlayerAction::Defend)
                    .await?
            }
            AppEvent::CombatFlee => {
                self.handle_explicit_combat_action(PlayerAction::Flee)
                    .await?
            }
            AppEvent::Quit => self.quit(),
        }
        Ok(())
    }

    fn select_up(&mut self) {
        match self.screen {
            Screen::MainMenu => cycle_cursor(&mut self.selected, -1, MenuItem::ALL.len()),
            Screen::Options => cycle_cursor(&mut self.options_cursor, -1, OPTIONS_COUNT),
            Screen::LoadGame if !self.saved_characters.is_empty() => {
                cycle_cursor(&mut self.load_cursor, -1, self.saved_characters.len())
            }
            Screen::Town => cycle_cursor(&mut self.town_cursor, -1, TownAction::ALL.len()),
            Screen::Explore => cycle_cursor(&mut self.explore_cursor, -1, AreaId::ALL.len()),
            Screen::CharacterSheet => {
                self.character_cursor = self.character_cursor.saturating_sub(1)
            }
            Screen::Inventory => self.inventory.cursor_up(),
            Screen::Equipment => cycle_cursor(&mut self.equipment_cursor, -1, EquipSlot::ALL.len()),
            Screen::Quests => cycle_cursor(&mut self.quest_cursor, -1, QuestId::ALL.len()),
            Screen::Achievements => cycle_cursor(
                &mut self.achievement_cursor,
                -1,
                crate::achievements::achievement_defs().len(),
            ),
            Screen::Shop => self.shop_cursor = self.shop_cursor.saturating_sub(1),
            Screen::CharacterCreation => match self.creation.step {
                CreationStep::Race => {
                    cycle_cursor(&mut self.creation.race_cursor, -1, Race::ALL.len())
                }
                CreationStep::Class => {
                    cycle_cursor(&mut self.creation.class_cursor, -1, Class::ALL.len())
                }
                CreationStep::Stats => cycle_cursor(&mut self.creation.stat_cursor, -1, 6),
                CreationStep::Gear => {
                    cycle_cursor(&mut self.creation.gear_cursor, -1, GearPackage::ALL.len())
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn select_down(&mut self) {
        match self.screen {
            Screen::MainMenu => cycle_cursor(&mut self.selected, 1, MenuItem::ALL.len()),
            Screen::Options => cycle_cursor(&mut self.options_cursor, 1, OPTIONS_COUNT),
            Screen::LoadGame if !self.saved_characters.is_empty() => {
                cycle_cursor(&mut self.load_cursor, 1, self.saved_characters.len())
            }
            Screen::Town => cycle_cursor(&mut self.town_cursor, 1, TownAction::ALL.len()),
            Screen::Explore => cycle_cursor(&mut self.explore_cursor, 1, AreaId::ALL.len()),
            Screen::CharacterSheet => self.character_cursor += 1,
            Screen::Inventory => self.inventory.cursor_down(),
            Screen::Equipment => cycle_cursor(&mut self.equipment_cursor, 1, EquipSlot::ALL.len()),
            Screen::Quests => cycle_cursor(&mut self.quest_cursor, 1, QuestId::ALL.len()),
            Screen::Achievements => cycle_cursor(
                &mut self.achievement_cursor,
                1,
                crate::achievements::achievement_defs().len(),
            ),
            Screen::Shop => self.shop_cursor += 1,
            Screen::CharacterCreation => match self.creation.step {
                CreationStep::Race => {
                    cycle_cursor(&mut self.creation.race_cursor, 1, Race::ALL.len())
                }
                CreationStep::Class => {
                    cycle_cursor(&mut self.creation.class_cursor, 1, Class::ALL.len())
                }
                CreationStep::Stats => cycle_cursor(&mut self.creation.stat_cursor, 1, 6),
                CreationStep::Gear => {
                    cycle_cursor(&mut self.creation.gear_cursor, 1, GearPackage::ALL.len())
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn handle_left(&mut self) {
        match self.screen {
            Screen::Options => self.change_option(-1),
            Screen::CharacterCreation => self.creation.adjust_stat(-1),
            Screen::CharacterSheet => self.change_character_tab(-1),
            _ => {}
        }
    }

    async fn handle_right(&mut self) -> color_eyre::Result<()> {
        match self.screen {
            Screen::Options => self.change_option(1),
            Screen::CharacterCreation => self.creation.adjust_stat(1),
            Screen::CharacterSheet => self.change_character_tab(1),
            Screen::Inventory => self.equip_selected_item().await?,
            _ => {}
        }
        Ok(())
    }

    async fn confirm(&mut self) -> color_eyre::Result<()> {
        match self.screen {
            Screen::MainMenu => match MenuItem::ALL[self.selected] {
                MenuItem::NewGame => {
                    self.creation = CharacterCreation::default();
                    self.screen = Screen::CharacterCreation;
                }
                MenuItem::LoadGame => {
                    self.saved_characters = db::load_characters(&self.pool).await?;
                    self.load_cursor = 0;
                    self.screen = Screen::LoadGame;
                }
                MenuItem::Options => self.screen = Screen::Options,
                MenuItem::Quit => self.quit(),
            },
            Screen::CharacterCreation => {
                if self.creation.step == CreationStep::Confirm {
                    let id = db::save_character(&self.pool, &self.creation).await?;
                    self.load_session(id).await?;
                    self.screen = Screen::Town;
                    self.status_message =
                        Some("A new adventurer arrives in Hearthmere.".to_string());
                } else {
                    self.creation.step = self.creation.step.next();
                }
            }
            Screen::LoadGame => {
                if let Some(character) = self.saved_characters.get(self.load_cursor) {
                    self.load_session(character.id).await?;
                    self.screen = Screen::Town;
                }
            }
            Screen::Town => match TownAction::ALL[self.town_cursor] {
                TownAction::Explore => self.open_explore(),
                TownAction::Character => self.open_character_sheet(),
                TownAction::Inventory => self.open_inventory().await?,
                TownAction::Equipment => self.open_equipment().await?,
                TownAction::Quests => self.open_quests(),
                TownAction::Achievements => self.open_achievements(),
                TownAction::Shop => self.open_shop().await?,
                TownAction::Rest => self.rest_at_inn().await?,
                TownAction::LeaveTown => self.screen = Screen::MainMenu,
            },
            Screen::Inventory => self.use_inventory_item().await?,
            Screen::Equipment => self.unequip_item().await?,
            Screen::CharacterSheet if self.character_tab == CharacterTab::Proficiencies => {
                self.train_selected_proficiency().await?
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
            Screen::Dialogue => self.screen = self.dialogue_return,
            Screen::Explore
            | Screen::CharacterSheet
            | Screen::Inventory
            | Screen::Equipment
            | Screen::Quests
            | Screen::Achievements
            | Screen::Shop
            | Screen::Combat => {
                self.screen = Screen::Town;
                self.combat = None;
            }
            Screen::Town => self.screen = Screen::MainMenu,
            Screen::LoadGame => self.screen = Screen::MainMenu,
            _ => self.screen = Screen::MainMenu,
        }
        Ok(())
    }

    fn change_option(&mut self, dir: i32) {
        match self.options_cursor {
            0 => self.settings.sound_effects = !self.settings.sound_effects,
            1 => {
                if dir > 0 {
                    self.settings.music_volume =
                        self.settings.music_volume.saturating_add(10).min(100);
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

    fn open_explore(&mut self) {
        self.explore_cursor = 0;
        self.screen = Screen::Explore;
        self.status_message = None;
    }

    fn open_character_sheet(&mut self) {
        self.character_tab = CharacterTab::Attributes;
        self.character_cursor = 0;
        self.screen = Screen::CharacterSheet;
    }

    fn open_quests(&mut self) {
        self.quest_cursor = 0;
        self.screen = Screen::Quests;
    }

    fn open_achievements(&mut self) {
        self.achievement_cursor = 0;
        self.screen = Screen::Achievements;
    }

    async fn open_inventory(&mut self) -> color_eyre::Result<()> {
        let Some(ch) = &self.active_character else {
            return Ok(());
        };
        self.inventory.items = db::load_inventory(&self.pool, ch.id).await?;
        self.inventory.cursor = 0;
        self.inventory.last_use_message = None;
        self.screen = Screen::Inventory;
        Ok(())
    }

    async fn open_equipment(&mut self) -> color_eyre::Result<()> {
        let Some(ch) = &self.active_character else {
            return Ok(());
        };
        self.equipment = db::load_equipment(&self.pool, ch.id).await?;
        self.equipment_cursor = 0;
        self.status_message = None;
        self.screen = Screen::Equipment;
        Ok(())
    }

    async fn open_shop(&mut self) -> color_eyre::Result<()> {
        let Some(ch) = &self.active_character else {
            return Ok(());
        };
        self.inventory.items = db::load_inventory(&self.pool, ch.id).await?;
        self.vendor_cursor = 0;
        self.shop_cursor = 0;
        self.shop_buy_mode = true;
        self.screen = Screen::Shop;
        Ok(())
    }

    async fn rest_at_inn(&mut self) -> color_eyre::Result<()> {
        let Some(ch) = self.active_character.as_mut() else {
            return Ok(());
        };
        let character_id = ch.id;
        let cost = if ch.level <= 2 { 0 } else { 12 };
        if ch.gold < cost {
            self.status_message = Some("You cannot afford a room tonight.".to_string());
            return Ok(());
        }
        ch.gold -= cost;
        ch.resources.hp = ch.resources.max_hp;
        ch.resources.mana = ch.resources.max_mana;
        ch.resources.stamina = ch.resources.max_stamina;
        db::save_character_state(&self.pool, ch).await?;
        self.world_state.advance_time(8);
        db::save_world_state(&self.pool, character_id, &self.world_state).await?;
        let unlocked = self
            .achievement_increment(character_id, "rests_taken", 1)
            .await?;
        let spent = if cost > 0 {
            self.achievement_increment(character_id, "gold_spent", cost)
                .await?
        } else {
            vec![]
        };
        let mut message = if cost == 0 {
            "The innkeeper gives you your first night free.".to_string()
        } else {
            format!("You rest, recover fully, and spend {cost} gold.")
        };
        if let Some(name) = unlocked.into_iter().chain(spent.into_iter()).last() {
            message.push_str(&format!(" Achievement unlocked: {name}."));
        }
        self.status_message = Some(message);
        Ok(())
    }

    async fn use_inventory_item(&mut self) -> color_eyre::Result<()> {
        let Some(item) = self.inventory.selected().cloned() else {
            return Ok(());
        };
        let Some(def) = item.def() else {
            return Ok(());
        };
        if !def.is_usable() {
            self.inventory.last_use_message =
                Some(format!("{} cannot be used directly.", def.name));
            return Ok(());
        }
        let Some(ch) = self.active_character.as_mut() else {
            return Ok(());
        };
        for effect in def.effects {
            match effect {
                ItemEffect::HealHp(amount) => {
                    ch.resources.hp = (ch.resources.hp + amount).min(ch.resources.max_hp)
                }
                ItemEffect::RestoreMana(amount) => {
                    ch.resources.mana = (ch.resources.mana + amount).min(ch.resources.max_mana)
                }
                ItemEffect::RestoreStamina(amount) => {
                    ch.resources.stamina =
                        (ch.resources.stamina + amount).min(ch.resources.max_stamina)
                }
                ItemEffect::CurePoison => {}
                ItemEffect::ApplyGuard(_) => {}
            }
        }
        db::remove_item(&self.pool, ch.id, &item.item_type, 1).await?;
        db::save_character_state(&self.pool, ch).await?;
        self.inventory.items = db::load_inventory(&self.pool, ch.id).await?;
        self.inventory.clamp_cursor();
        self.inventory.last_use_message = Some(format!("Used {}.", def.name));
        Ok(())
    }

    async fn equip_selected_item(&mut self) -> color_eyre::Result<()> {
        let Some(item) = self.inventory.selected().cloned() else {
            return Ok(());
        };
        let Some(def) = item.def() else {
            return Ok(());
        };
        let Some(slot) = def.equip_slot else {
            self.inventory.last_use_message = Some(format!("{} cannot be equipped.", def.name));
            return Ok(());
        };
        let Some(ch) = &self.active_character else {
            return Ok(());
        };
        let character_id = ch.id;
        if let Some(current) = self.equipment.get_slot(slot).map(|it| it.to_string()) {
            db::add_item(&self.pool, character_id, &current, 1).await?;
        }
        db::remove_item(&self.pool, character_id, &item.item_type, 1).await?;
        db::equip_item(&self.pool, character_id, slot, &item.item_type).await?;
        self.equipment = db::load_equipment(&self.pool, character_id).await?;
        self.inventory.items = db::load_inventory(&self.pool, character_id).await?;
        self.inventory.clamp_cursor();
        let mut message = format!("Equipped {}.", def.name);
        let mut unlocked = self
            .achievement_increment(character_id, "items_equipped", 1)
            .await?;
        unlocked.extend(self.refresh_meta_achievement_metrics(character_id).await?);
        if let Some(name) = unlocked.last() {
            message.push_str(&format!(" Achievement unlocked: {name}."));
        }
        self.inventory.last_use_message = Some(message);
        Ok(())
    }

    async fn unequip_item(&mut self) -> color_eyre::Result<()> {
        let Some(ch) = &self.active_character else {
            return Ok(());
        };
        let character_id = ch.id;
        let slot = EquipSlot::ALL[self.equipment_cursor];
        let Some(item_type) = self.equipment.get_slot(slot).map(|it| it.to_string()) else {
            self.status_message = Some("Nothing is equipped in that slot.".to_string());
            return Ok(());
        };
        db::unequip_item(&self.pool, character_id, slot).await?;
        db::add_item(&self.pool, character_id, &item_type, 1).await?;
        self.equipment = db::load_equipment(&self.pool, character_id).await?;
        let mut message = format!(
            "Unequipped {}.",
            find_def(&item_type).map(|def| def.name).unwrap_or("item")
        );
        let unlocked = self.refresh_meta_achievement_metrics(character_id).await?;
        if let Some(name) = unlocked.last() {
            message.push_str(&format!(" Achievement unlocked: {name}."));
        }
        self.status_message = Some(message);
        Ok(())
    }

    fn toggle_shop_mode(&mut self) {
        self.shop_buy_mode = !self.shop_buy_mode;
        self.shop_cursor = 0;
    }

    fn shop_cycle_vendor(&mut self, dir: i32) {
        cycle_cursor(&mut self.vendor_cursor, dir, VendorId::ALL.len());
        self.shop_cursor = 0;
    }

    async fn shop_transaction(&mut self) -> color_eyre::Result<()> {
        let Some(ch) = self.active_character.as_mut() else {
            return Ok(());
        };
        let character_id = ch.id;
        if self.shop_buy_mode {
            let vendor = vendor_def(VendorId::ALL[self.vendor_cursor]);
            let Some(entry) = vendor.inventory.get(
                self.shop_cursor
                    .min(vendor.inventory.len().saturating_sub(1)),
            ) else {
                return Ok(());
            };
            let Some(def) = find_def(entry.item_type) else {
                return Ok(());
            };
            if ch.gold < def.base_value {
                self.status_message = Some("You do not have enough gold.".to_string());
                return Ok(());
            }
            ch.gold -= def.base_value;
            db::add_item(&self.pool, character_id, entry.item_type, 1).await?;
            db::save_character_state(&self.pool, ch).await?;
            self.inventory.items = db::load_inventory(&self.pool, character_id).await?;
            let unlocked = self
                .achievement_increment(character_id, "gold_spent", def.base_value)
                .await?;
            let mut message = format!("Bought {} for {} gold.", def.name, def.base_value);
            if let Some(name) = unlocked.last() {
                message.push_str(&format!(" Achievement unlocked: {name}."));
            }
            self.status_message = Some(message);
        } else {
            self.inventory.items = db::load_inventory(&self.pool, character_id).await?;
            let Some(item) = self
                .inventory
                .items
                .get(
                    self.shop_cursor
                        .min(self.inventory.items.len().saturating_sub(1)),
                )
                .cloned()
            else {
                return Ok(());
            };
            let Some(def) = item.def() else {
                return Ok(());
            };
            if def.base_value <= 0 {
                self.status_message = Some("That item has no market value.".to_string());
                return Ok(());
            }
            let sell_price = (def.base_value * 40) / 100;
            ch.gold += sell_price;
            db::remove_item(&self.pool, character_id, &item.item_type, 1).await?;
            db::save_character_state(&self.pool, ch).await?;
            self.inventory.items = db::load_inventory(&self.pool, character_id).await?;
            let unlocked = self
                .achievement_increment(character_id, "gold_earned", sell_price)
                .await?;
            let mut message = format!("Sold {} for {} gold.", def.name, sell_price);
            if let Some(name) = unlocked.last() {
                message.push_str(&format!(" Achievement unlocked: {name}."));
            }
            self.status_message = Some(message);
        }
        Ok(())
    }

    async fn accept_selected_quest(&mut self) -> color_eyre::Result<()> {
        let Some(ch) = &self.active_character else {
            return Ok(());
        };
        let quest_id = QuestId::ALL[self.quest_cursor];
        if self.world_state.has_completed(quest_id) {
            self.status_message = Some("That quest is already complete.".to_string());
            return Ok(());
        }
        if !self.world_state.accept_quest(quest_id) {
            self.status_message = Some("That quest is already active.".to_string());
            return Ok(());
        }
        db::save_world_state(&self.pool, ch.id, &self.world_state).await?;
        self.status_message = Some(format!(
            "Accepted {}.",
            quest_def(quest_id.id()).map(|q| q.name).unwrap_or("quest")
        ));
        Ok(())
    }

    async fn explore_selected(&mut self) -> color_eyre::Result<()> {
        let area = AreaId::ALL[self.explore_cursor];
        if !self.world_state.is_area_unlocked(area) {
            self.status_message = Some("That route is not yet safe enough to travel.".to_string());
            return Ok(());
        }
        self.world_state.advance_time(6);
        self.world_state.current_area = Some(area.id().to_string());
        self.apply_area_visit(area).await?;

        let area = area_def(area);
        let mut rng = rand::rng();
        if rng.random_bool(0.7) {
            let encounter_id = area.encounters[rng.random_range(0..area.encounters.len())];
            self.start_combat(encounter_id).await?;
        } else {
            let Some(ch) = &self.active_character else {
                return Ok(());
            };
            db::add_item(&self.pool, ch.id, "ration", 1).await?;
            self.open_dialog(
                area.name,
                vec![
                    area.event_text.to_string(),
                    "You secure supplies and withdraw before the deeper threats close in."
                        .to_string(),
                    "Ration x1 added to your pack.".to_string(),
                ],
                Screen::Town,
            );
        }
        Ok(())
    }

    async fn train_selected_proficiency(&mut self) -> color_eyre::Result<()> {
        if let Some(training) = &self.active_training {
            self.status_message = Some(format!(
                "You are already studying {}. Wait for the current session to finish.",
                training.skill.name()
            ));
            return Ok(());
        }

        let Some(ch) = self.active_character.as_mut() else {
            return Ok(());
        };
        let skill_idx = self
            .character_cursor
            .min(ch.proficiencies.len().saturating_sub(1));
        let Some(skill) = ch.proficiencies.get_mut(skill_idx) else {
            return Ok(());
        };
        if skill.level() >= MAX_PROFICIENCY_LEVEL {
            self.status_message = Some(format!("{} is already mastered.", skill.kind.name()));
            return Ok(());
        }

        let plan = study_plan(skill.kind, skill.xp, &ch.stats);
        self.recent_training_level_up = None;
        self.active_training = Some(ActiveTraining {
            skill: skill.kind,
            total_ticks: (plan.hours.max(1) as u32) * TRAINING_TICKS_PER_HOUR,
            elapsed_ticks: 0,
            hours: plan.hours,
            success_chance: plan.success_chance,
            success_xp: plan.success_xp,
            failure_xp: plan.failure_xp,
        });
        self.status_message = Some(format!(
            "You begin studying {}. Progress will complete over time.",
            skill.kind.name(),
        ));
        Ok(())
    }

    async fn resolve_training_completion(&mut self) -> color_eyre::Result<()> {
        let Some(training) = self.active_training.take() else {
            return Ok(());
        };
        let roll = rand::rng().random_range(1..=100);
        let success = roll <= training.success_chance;
        let xp_gain = if success {
            training.success_xp
        } else {
            training.failure_xp
        };
        let Some((character_id, skill_kind, before_level, after_level, new_xp)) = ({
            let Some(ch) = self.active_character.as_mut() else {
                return Ok(());
            };
            let Some(skill) = ch
                .proficiencies
                .iter_mut()
                .find(|skill| skill.kind == training.skill)
            else {
                return Ok(());
            };
            let before_level = skill.level();
            skill.xp += xp_gain;
            let after_level = skill.level();
            Some((ch.id, skill.kind, before_level, after_level, skill.xp))
        }) else {
            return Ok(());
        };
        self.world_state.advance_time(training.hours);

        db::save_proficiency_xp(&self.pool, character_id, skill_kind, new_xp).await?;
        db::save_world_state(&self.pool, character_id, &self.world_state).await?;
        let mut unlocked = self
            .achievement_increment(character_id, "study_sessions", 1)
            .await?;
        unlocked.extend(
            self.achievement_increment(character_id, "study_hours", training.hours)
                .await?,
        );
        if success {
            unlocked.extend(
                self.achievement_increment(character_id, "study_successes", 1)
                    .await?,
            );
        }
        unlocked.extend(self.refresh_meta_achievement_metrics(character_id).await?);

        let result = if success { "Success" } else { "Setback" };
        let mut message = format!(
            "{result}: {} training finished after {}h. Roll {} vs {}%, gained {} XP.",
            skill_kind.name(),
            training.hours,
            roll,
            training.success_chance,
            xp_gain
        );
        if after_level > before_level {
            self.recent_training_level_up = Some((skill_kind, after_level));
            message.push_str(&format!(" Rank up to {}.", after_level));
        }
        if let Some(name) = unlocked.last() {
            message.push_str(&format!(" Achievement unlocked: {name}."));
        }
        self.status_message = Some(message);
        Ok(())
    }

    async fn start_combat(&mut self, encounter_id: &str) -> color_eyre::Result<()> {
        let Some(ch) = &self.active_character else {
            return Ok(());
        };
        self.inventory.items = db::load_inventory(&self.pool, ch.id).await?;
        self.equipment = db::load_equipment(&self.pool, ch.id).await?;
        let mut combat = CombatState::from_character_and_encounter(
            ch,
            &self.equipment,
            &self.inventory.items,
            encounter_id,
        );
        let opening_outcome = combat.begin_encounter();
        self.combat = Some(combat);
        if !matches!(opening_outcome, CombatOutcome::Ongoing) {
            self.finish_combat(opening_outcome).await?;
            return Ok(());
        }
        self.screen = Screen::Combat;
        Ok(())
    }

    fn set_combat_tab(&mut self, tab: ActionTab) {
        if let Some(combat) = self.combat.as_mut() {
            combat.set_tab(tab);
        }
    }

    fn cycle_combat_option(&mut self, dir: i32) {
        if let Some(combat) = self.combat.as_mut() {
            combat.cycle_selection(dir);
        }
    }

    fn cycle_combat_target(&mut self, dir: i32) {
        if let Some(combat) = self.combat.as_mut() {
            combat.cycle_target(dir);
        }
    }

    async fn handle_combat_action(&mut self) -> color_eyre::Result<()> {
        let action = match self
            .combat
            .as_ref()
            .map(|combat| combat.action_tab)
            .unwrap_or(ActionTab::Weapon)
        {
            ActionTab::Weapon => PlayerAction::UseWeapon,
            ActionTab::Ability => PlayerAction::UseAbility,
            ActionTab::Item => PlayerAction::UseItem,
        };
        self.handle_explicit_combat_action(action).await
    }

    async fn handle_explicit_combat_action(
        &mut self,
        action: PlayerAction,
    ) -> color_eyre::Result<()> {
        let Some(combat) = self.combat.as_mut() else {
            return Ok(());
        };
        let outcome = combat.resolve_player_action(action);
        if !matches!(outcome, CombatOutcome::Ongoing) {
            self.finish_combat(outcome).await?;
        }
        Ok(())
    }

    async fn finish_combat(&mut self, outcome: CombatOutcome) -> color_eyre::Result<()> {
        let Some(mut character) = self.active_character.clone() else {
            self.combat = None;
            self.screen = Screen::Town;
            return Ok(());
        };
        match outcome {
            CombatOutcome::Won(reward) => {
                let character_id = character.id;
                let starting_gold = character.gold;
                character.resources = self
                    .combat
                    .as_ref()
                    .map(|combat| combat.player.resources)
                    .unwrap_or(character.resources);
                character.gold += reward.gold;
                let level_up = character.apply_xp_gain(reward.xp);
                for (item, qty) in &reward.drops {
                    db::add_item(&self.pool, character.id, item, *qty).await?;
                }
                let quest_lines = self
                    .apply_combat_rewards_to_world(&mut character, &reward)
                    .await?;
                db::save_character_state(&self.pool, &character).await?;
                self.active_character = Some(character.clone());
                self.inventory.items = db::load_inventory(&self.pool, character.id).await?;
                let total_gold_earned = character.gold - starting_gold;
                let mut achievement_lines = vec![];
                Self::append_achievement_lines(
                    &mut achievement_lines,
                    self.achievement_increment(character_id, "combat_victories", 1)
                        .await?,
                );
                Self::append_achievement_lines(
                    &mut achievement_lines,
                    self.achievement_increment(
                        character_id,
                        "enemy_kills",
                        reward.enemies_defeated,
                    )
                    .await?,
                );
                Self::append_achievement_lines(
                    &mut achievement_lines,
                    self.achievement_increment(character_id, "beast_kills", reward.beast_kills)
                        .await?,
                );
                Self::append_achievement_lines(
                    &mut achievement_lines,
                    self.achievement_increment(character_id, "bandit_kills", reward.bandit_kills)
                        .await?,
                );
                Self::append_achievement_lines(
                    &mut achievement_lines,
                    self.achievement_increment(character_id, "undead_kills", reward.undead_kills)
                        .await?,
                );
                Self::append_achievement_lines(
                    &mut achievement_lines,
                    self.achievement_increment(character_id, "damage_dealt", reward.damage_dealt)
                        .await?,
                );
                Self::append_achievement_lines(
                    &mut achievement_lines,
                    self.achievement_increment(character_id, "ability_uses", reward.ability_uses)
                        .await?,
                );
                Self::append_achievement_lines(
                    &mut achievement_lines,
                    self.achievement_increment(
                        character_id,
                        "weapon_attacks",
                        reward.weapon_attacks,
                    )
                    .await?,
                );
                Self::append_achievement_lines(
                    &mut achievement_lines,
                    self.achievement_increment(character_id, "item_uses", reward.item_uses)
                        .await?,
                );
                Self::append_achievement_lines(
                    &mut achievement_lines,
                    self.achievement_increment(character_id, "gold_earned", total_gold_earned)
                        .await?,
                );
                Self::append_achievement_lines(
                    &mut achievement_lines,
                    self.refresh_meta_achievement_metrics(character_id).await?,
                );
                self.combat = None;
                self.open_dialog(
                    "Victory",
                    {
                        let mut lines = vec![
                            format!("You win the encounter and claim {} XP and {} gold.", reward.xp, reward.gold),
                        ];
                        if level_up.levels_gained > 0 {
                            lines.push(format!(
                                "Level up! Now level {}. +{} HP, +{} Mana, +{} Stamina, +{} stat points.",
                                character.level,
                                level_up.hp_gain,
                                level_up.mana_gain,
                                level_up.stamina_gain,
                                level_up.attribute_points_awarded
                            ));
                        }
                        if !level_up.new_ability_ids.is_empty() {
                            let ability_names = level_up
                                .new_ability_ids
                                .iter()
                                .map(|ability_id| {
                                    ability_def(ability_id)
                                        .map(|def| def.name)
                                        .unwrap_or(ability_id.as_str())
                                })
                                .collect::<Vec<_>>()
                                .join(", ");
                            lines.push(format!("New abilities unlocked: {ability_names}"));
                        }
                        lines.extend(quest_lines);
                        lines.extend(achievement_lines);
                        lines
                    },
                    Screen::Town,
                );
            }
            CombatOutcome::Lost => {
                character.resources.hp = (character.resources.max_hp / 2).max(1);
                character.resources.mana = character.resources.max_mana;
                character.resources.stamina = character.resources.max_stamina;
                db::save_character_state(&self.pool, &character).await?;
                self.active_character = Some(character);
                self.combat = None;
                self.open_dialog(
                    "Defeat",
                    vec![
                        "You are carried back to Hearthmere in rough shape.".to_string(),
                        "The healer stabilizes you, but the outing is lost.".to_string(),
                    ],
                    Screen::Town,
                );
            }
            CombatOutcome::Fled => {
                if let Some(combat) = &self.combat {
                    character.resources = combat.player.resources;
                }
                db::save_character_state(&self.pool, &character).await?;
                self.active_character = Some(character);
                self.combat = None;
                self.open_dialog(
                    "Retreat",
                    vec![
                        "You break away and return to town before the fight collapses around you."
                            .to_string(),
                    ],
                    Screen::Town,
                );
            }
            CombatOutcome::Ongoing => {}
        }
        Ok(())
    }

    async fn apply_area_visit(&mut self, area: AreaId) -> color_eyre::Result<()> {
        match area {
            AreaId::WhisperingWoods => self.world_state.unlock_area(AreaId::SunkenRoad),
            AreaId::SunkenRoad => self.world_state.unlock_area(AreaId::AshenBarrow),
            AreaId::AshenBarrow => {}
        }

        let active_ids = self
            .world_state
            .active_quests
            .iter()
            .map(|q| q.quest_id.clone())
            .collect::<Vec<_>>();
        for quest_id in active_ids {
            let Some(def) = quest_def(&quest_id) else {
                continue;
            };
            let Some(progress) = self.world_state.active_quest_mut(def.id) else {
                continue;
            };
            if progress.completed || progress.objective_index >= def.objectives.len() {
                continue;
            }
            if let ObjectiveKind::VisitArea { area: needed } =
                &def.objectives[progress.objective_index].kind
            {
                if *needed == area {
                    progress.progress = 1;
                    progress.objective_index += 1;
                    self.status_message = Some(format!("Quest updated: {}.", def.name));
                }
            }
        }
        let _ = self.complete_ready_quests().await?;
        Ok(())
    }

    async fn apply_combat_rewards_to_world(
        &mut self,
        character: &mut SavedCharacter,
        reward: &crate::combat::CombatReward,
    ) -> color_eyre::Result<Vec<String>> {
        let mut lines = vec![];
        let active_ids = self
            .world_state
            .active_quests
            .iter()
            .map(|q| q.quest_id.clone())
            .collect::<Vec<_>>();
        for quest_id in active_ids {
            let Some(def) = quest_def(&quest_id) else {
                continue;
            };
            let Some(progress) = self.world_state.active_quest_mut(def.id) else {
                continue;
            };
            if progress.completed || progress.objective_index >= def.objectives.len() {
                continue;
            }
            match &def.objectives[progress.objective_index].kind {
                ObjectiveKind::KillFamily { family, count } => {
                    let gained = reward
                        .defeated_families
                        .iter()
                        .filter(|it| it.as_str() == *family)
                        .count() as i32;
                    if gained > 0 {
                        progress.progress += gained;
                        lines.push(format!(
                            "{} progress: {}/{} {}",
                            def.name,
                            progress.progress.min(*count),
                            count,
                            family
                        ));
                        if progress.progress >= *count {
                            progress.objective_index += 1;
                            progress.progress = 0;
                        }
                    }
                }
                ObjectiveKind::OwnItem { item_type, count } => {
                    let total = self
                        .inventory
                        .items
                        .iter()
                        .find(|item| item.item_type == *item_type)
                        .map(|item| item.quantity)
                        .unwrap_or(0)
                        + reward
                            .drops
                            .iter()
                            .filter(|(item, _)| item == item_type)
                            .map(|(_, qty)| *qty)
                            .sum::<i32>();
                    if total >= *count {
                        progress.objective_index += 1;
                        progress.progress = total;
                    }
                }
                ObjectiveKind::VisitArea { .. } => {}
            }
        }

        for line in self.complete_ready_quests().await? {
            character.gold += line.1;
            character.apply_xp_gain(line.0);
            if let Some(item) = line.2 {
                db::add_item(&self.pool, character.id, &item, line.3).await?;
            }
            lines.push(line.4);
        }
        db::save_world_state(&self.pool, character.id, &self.world_state).await?;
        Ok(lines)
    }

    async fn complete_ready_quests(
        &mut self,
    ) -> color_eyre::Result<Vec<(i32, i32, Option<String>, i32, String)>> {
        let mut rewards = vec![];
        let ids = self
            .world_state
            .active_quests
            .iter()
            .map(|q| q.quest_id.clone())
            .collect::<Vec<_>>();
        for quest_id in ids {
            let Some(def) = quest_def(&quest_id) else {
                continue;
            };
            let complete_now = self
                .world_state
                .active_quest(def.id)
                .map(|progress| progress.objective_index >= def.objectives.len())
                .unwrap_or(false);
            if !complete_now {
                continue;
            }
            self.world_state
                .completed_quests
                .push(def.id.id().to_string());
            self.world_state
                .active_quests
                .retain(|progress| progress.quest_id != def.id.id());
            rewards.push((
                def.rewards.xp,
                def.rewards.gold,
                def.rewards.item_type.map(|item| item.to_string()),
                def.rewards.item_qty,
                format!("Quest complete: {}.", def.name),
            ));
        }
        Ok(rewards)
    }

    fn change_character_tab(&mut self, dir: i32) {
        let idx = CharacterTab::ALL
            .iter()
            .position(|tab| *tab == self.character_tab)
            .unwrap_or(0);
        let next = (idx as i32 + dir).rem_euclid(CharacterTab::ALL.len() as i32) as usize;
        self.character_tab = CharacterTab::ALL[next];
        self.character_cursor = 0;
    }

    fn next_character_tab(&mut self) {
        self.change_character_tab(1);
    }

    fn open_dialog(&mut self, title: &str, lines: Vec<String>, return_screen: Screen) {
        self.dialogue_title = title.to_string();
        self.dialogue_lines = lines;
        self.dialogue_return = return_screen;
        self.screen = Screen::Dialogue;
    }

    async fn achievement_increment(
        &mut self,
        character_id: i64,
        metric: &str,
        amount: i32,
    ) -> color_eyre::Result<Vec<String>> {
        let unlocked = self.achievements.record_increment(metric, amount);
        db::save_achievement_metric(
            &self.pool,
            character_id,
            metric,
            self.achievements.progress_for(metric),
        )
        .await?;
        Ok(unlocked.into_iter().map(|def| def.name).collect())
    }

    async fn achievement_set_max(
        &mut self,
        character_id: i64,
        metric: &str,
        value: i32,
    ) -> color_eyre::Result<Vec<String>> {
        let unlocked = self.achievements.record_max(metric, value);
        db::save_achievement_metric(
            &self.pool,
            character_id,
            metric,
            self.achievements.progress_for(metric),
        )
        .await?;
        Ok(unlocked.into_iter().map(|def| def.name).collect())
    }

    async fn refresh_meta_achievement_metrics(
        &mut self,
        character_id: i64,
    ) -> color_eyre::Result<Vec<String>> {
        let mut unlocked = vec![];
        let Some(ch) = &self.active_character else {
            return Ok(unlocked);
        };
        let best_prof = ch
            .proficiencies
            .iter()
            .map(|skill| skill.level() as i32)
            .max()
            .unwrap_or(1);
        let level = ch.level;
        let ability_count = ch
            .known_abilities
            .iter()
            .filter(|ability| ability.unlocked)
            .count() as i32;
        let equipment_slots_filled = EquipSlot::ALL
            .iter()
            .filter(|slot| self.equipment.get_slot(**slot).is_some())
            .count() as i32;
        let _ = ch;
        unlocked.extend(
            self.achievement_set_max(character_id, "best_proficiency_rank", best_prof)
                .await?,
        );
        unlocked.extend(
            self.achievement_set_max(character_id, "level_reached", level)
                .await?,
        );
        unlocked.extend(
            self.achievement_set_max(character_id, "abilities_unlocked", ability_count)
                .await?,
        );
        unlocked.extend(
            self.achievement_set_max(
                character_id,
                "equipment_slots_filled",
                equipment_slots_filled,
            )
            .await?,
        );
        Ok(unlocked)
    }

    fn append_achievement_lines(lines: &mut Vec<String>, unlocked: Vec<String>) {
        for name in unlocked {
            lines.push(format!("Achievement unlocked: {name}."));
        }
    }

    async fn load_session(&mut self, character_id: i64) -> color_eyre::Result<()> {
        self.active_character = Some(db::load_character_by_id(&self.pool, character_id).await?);
        self.world_state = db::load_world_state(&self.pool, character_id).await?;
        self.equipment = db::load_equipment(&self.pool, character_id).await?;
        self.inventory.items = db::load_inventory(&self.pool, character_id).await?;
        self.achievements = db::load_achievement_state(&self.pool, character_id).await?;
        self.combat = None;
        self.refresh_meta_achievement_metrics(character_id).await?;
        Ok(())
    }

    fn quit(&mut self) {
        self.running = false;
    }
}

fn cycle_cursor(cursor: &mut usize, dir: i32, len: usize) {
    if len == 0 {
        *cursor = 0;
        return;
    }
    *cursor = ((*cursor as i32 + dir).rem_euclid(len as i32)) as usize;
}

pub fn active_level_progress(app: &App) -> f64 {
    app.active_character
        .as_ref()
        .map(|character| level_progress_pct(character.xp))
        .unwrap_or(0.0)
}

pub fn active_xp_to_next(app: &App) -> i32 {
    app.active_character
        .as_ref()
        .map(|character| xp_to_next_level(character.xp))
        .unwrap_or(0)
}
