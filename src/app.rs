mod combat;
mod input;
mod navigation;
mod progression;
mod training;
mod world;

use crate::achievements::AchievementState;
use crate::audio::DialogueAudioState;
use crate::character::{
    CharacterCreation, CreationStep, MajorSkill, MinorSkill, SavedCharacter, TrainingSessionPlan,
};
use crate::combat::{ActionTab, CombatState, PlayerAction};
use crate::db;
use crate::event::{AppEvent, Event, EventHandler};
use crate::inventory::{Equipment, InventoryState};
use crate::settings::UserSettings;
use crate::world::{NpcId, WorldState};
pub use progression::{active_level_progress, active_xp_to_next};
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
    People,
    Explore,
    CharacterSheet,
    Inventory,
    Equipment,
    Achievements,
    Quests,
    Shop,
    Training,
    Dialogue,
    Combat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TownAction {
    Explore,
    Rest,
    People,
    Train,
    Shop,
    Inventory,
    Equipment,
    Character,
    Quests,
    Achievements,
    LeaveTown,
}

impl TownAction {
    pub const ALL: [TownAction; 11] = [
        TownAction::Explore,
        TownAction::Rest,
        TownAction::People,
        TownAction::Quests,
        TownAction::Train,
        TownAction::Shop,
        TownAction::Inventory,
        TownAction::Equipment,
        TownAction::Character,
        TownAction::Achievements,
        TownAction::LeaveTown,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Explore => "Explore the Wilds",
            Self::Rest => "Rest at the Inn",
            Self::People => "Speak with Townsfolk",
            Self::Train => "Train Proficiencies",
            Self::Shop => "Visit Vendors",
            Self::Inventory => "Inventory",
            Self::Equipment => "Equipment",
            Self::Character => "Character Sheet",
            Self::Quests => "Quests",
            Self::Achievements => "Achievements",
            Self::LeaveTown => "Return to Main Menu",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterTab {
    Abilities,
    Proficiencies,
    Equipment,
}

impl CharacterTab {
    pub const ALL: [CharacterTab; 3] = [
        CharacterTab::Abilities,
        CharacterTab::Proficiencies,
        CharacterTab::Equipment,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Abilities => "Abilities",
            Self::Proficiencies => "Proficiencies",
            Self::Equipment => "Equipment",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProficiencyTarget {
    Major(MajorSkill),
    Minor(MinorSkill),
}

impl ProficiencyTarget {
    pub fn name(self) -> &'static str {
        match self {
            Self::Major(skill) => skill.full_name(),
            Self::Minor(skill) => skill.name(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrainingTier {
    Poor,
    Solid,
    Great,
}

impl TrainingTier {
    pub fn label(self) -> &'static str {
        match self {
            Self::Poor => "Poor",
            Self::Solid => "Solid",
            Self::Great => "Great",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrainingPhase {
    Showing,
    Input,
}

#[derive(Debug, Clone)]
pub struct TrainingSession {
    pub target: ProficiencyTarget,
    pub plan: TrainingSessionPlan,
    pub phase: TrainingPhase,
    pub sequence: Vec<char>,
    pub reveal_ticks_remaining: u32,
    pub input_index: usize,
    pub score: i32,
    pub max_score: i32,
    pub hits: usize,
    pub misses: usize,
}

impl TrainingSession {
    pub fn progress(&self) -> f64 {
        if self.max_score == 0 {
            1.0
        } else {
            (self.score as f64 / self.max_score as f64).clamp(0.0, 1.0)
        }
    }
}

#[derive(Debug, Clone)]
pub struct TrainingResult {
    pub target: ProficiencyTarget,
    pub tier: TrainingTier,
    pub gained_xp: i32,
    pub hours: i32,
    pub hits: usize,
    pub misses: usize,
    pub score: i32,
    pub max_score: i32,
    pub level_up_rank: Option<i32>,
    pub achievement_lines: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct TrainingState {
    pub cursor: usize,
    pub session: Option<TrainingSession>,
    pub result: Option<TrainingResult>,
}

#[derive(Debug, Clone)]
pub struct DialogueChoice {
    pub label: String,
    pub response_lines: Vec<String>,
    pub memory_flag: Option<String>,
    pub status_message: Option<String>,
    pub audio_id: Option<&'static str>,
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
    pub npc_cursor: usize,
    pub quest_cursor: usize,
    pub quest_show_completed: bool,
    pub quest_show_locked: bool,
    pub shop_cursor: usize,
    pub vendor_cursor: usize,
    pub shop_buy_mode: bool,
    pub detail_scroll: u16,
    pub equipment_cursor: usize,
    pub achievement_cursor: usize,
    pub character_cursor: usize,
    pub training: TrainingState,
    pub character_tab: CharacterTab,
    pub dialogue_title: String,
    pub dialogue_lines: Vec<String>,
    pub dialogue_choices: Vec<DialogueChoice>,
    pub dialogue_cursor: usize,
    pub dialogue_npc: Option<NpcId>,
    pub dialogue_return: Screen,
    pub dialogue_audio: DialogueAudioState,
    pub audio_wave_phase: u8,
    pub audio_wave_tick: u8,
    pub status_message: Option<String>,
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
            npc_cursor: 0,
            quest_cursor: 0,
            quest_show_completed: false,
            quest_show_locked: false,
            shop_cursor: 0,
            vendor_cursor: 0,
            shop_buy_mode: true,
            detail_scroll: 0,
            equipment_cursor: 0,
            achievement_cursor: 0,
            character_cursor: 0,
            training: TrainingState::default(),
            character_tab: CharacterTab::Proficiencies,
            dialogue_title: String::new(),
            dialogue_lines: vec![],
            dialogue_choices: vec![],
            dialogue_cursor: 0,
            dialogue_npc: None,
            dialogue_return: Screen::Town,
            dialogue_audio: DialogueAudioState::default(),
            audio_wave_phase: 0,
            audio_wave_tick: 0,
            status_message: None,
            achievements: AchievementState::default(),
            pool,
            events: EventHandler::new(),
        }
    }

    async fn save_world_state_for_active_character(&self) -> color_eyre::Result<()> {
        let Some(character_id) = self.active_character.as_ref().map(|character| character.id)
        else {
            return Ok(());
        };
        db::save_world_state(&self.pool, character_id, &self.world_state).await
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
        if self.screen == Screen::Training {
            self.tick_training_session().await?;
        }

        if self.dialogue_audio.is_playing() {
            self.audio_wave_tick = self.audio_wave_tick.wrapping_add(1) % 3;
            if self.audio_wave_tick == 0 {
                self.audio_wave_phase = self.audio_wave_phase.wrapping_add(1) % 8;
            }
        } else {
            self.audio_wave_phase = 0;
            self.audio_wave_tick = 0;
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
            AppEvent::OpenPeople => self.open_people(),
            AppEvent::OpenCharacter => self.open_character_sheet(),
            AppEvent::OpenInventory => self.open_inventory().await?,
            AppEvent::OpenEquipment => self.open_equipment().await?,
            AppEvent::OpenQuests => self.open_quests(),
            AppEvent::OpenAchievements => self.open_achievements(),
            AppEvent::OpenShop => self.open_shop().await?,
            AppEvent::OpenTraining => self.open_training(),
            AppEvent::RestAtInn => self.rest_at_inn().await?,
            AppEvent::ExploreSelected => self.explore_selected().await?,
            AppEvent::ShopToggleMode => self.toggle_shop_mode(),
            AppEvent::ShopNextVendor => self.shop_cycle_vendor(1),
            AppEvent::ShopPreviousVendor => self.shop_cycle_vendor(-1),
            AppEvent::ShopTransaction => self.shop_transaction().await?,
            AppEvent::QuestAccept => self.accept_selected_quest().await?,
            AppEvent::QuestToggleCompleted => self.toggle_quest_completed_filter(),
            AppEvent::QuestToggleLocked => self.toggle_quest_locked_filter(),
            AppEvent::CombatTabWeapon => self.set_combat_tab(ActionTab::Weapon),
            AppEvent::CombatTabAbility => self.set_combat_tab(ActionTab::Ability),
            AppEvent::CombatTabItem => self.set_combat_tab(ActionTab::Item),
            AppEvent::CombatNextTab => self.cycle_combat_tab(1),
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
            AppEvent::DetailScrollUp => self.scroll_detail(-4),
            AppEvent::DetailScrollDown => self.scroll_detail(4),
            AppEvent::TrainingInput(key) => self.handle_training_input(key).await?,
            AppEvent::Quit => self.quit(),
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
                TownAction::People => self.open_people(),
                TownAction::Train => self.open_training(),
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
            Screen::People => self.talk_to_selected_npc().await?,
            Screen::Training => self.start_training_session().await?,
            Screen::Dialogue if !self.dialogue_choices.is_empty() => {
                self.resolve_dialogue_choice().await?
            }
            Screen::Dialogue => self.go_back().await?,
            _ => {}
        }
        Ok(())
    }
}
