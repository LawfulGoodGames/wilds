use crate::character::{CharacterCreation, Class, CreationStep, GearPackage, Race, SavedCharacter};
use crate::combat::{AttackKind, CombatOutcome, CombatState, PlayerAction};
use crate::event::{AppEvent, Event, EventHandler};
use crate::inventory::{find_def, Equipment, EquipSlot, InventoryState};
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
    Inventory,
    Equipment,
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
    pub equipment: Equipment,
    pub equipment_cursor: usize,
    pub equipment_message: Option<String>,
    pub combat: Option<CombatState>,
    pub minor_skills_cursor: usize,
    pub inventory: InventoryState,
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
            equipment: Equipment::default(),
            equipment_cursor: 0,
            equipment_message: None,
            combat: None,
            minor_skills_cursor: 0,
            inventory: InventoryState::default(),
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
                    AppEvent::CombatCycleTarget => self.cycle_combat_target(),
                    AppEvent::CombatUseSelected => self.handle_combat_action(PlayerAction::UseSelectedAttack).await?,
                    AppEvent::CombatDefend => self.handle_combat_action(PlayerAction::Defend).await?,
                    AppEvent::CombatFlee => self.handle_combat_action(PlayerAction::Flee).await?,
                    AppEvent::OpenInventory    => self.open_inventory().await?,
                    AppEvent::InventoryUse     => self.use_inventory_item().await?,
                    AppEvent::InventoryEquip   => self.equip_selected_item().await?,
                    AppEvent::OpenEquipment    => self.open_equipment().await?,
                    AppEvent::EquipmentUnequip => self.unequip_item().await?,
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
                KeyCode::Char('i') => self.events.send(AppEvent::OpenInventory),
                KeyCode::Char('e') => self.events.send(AppEvent::OpenEquipment),
                KeyCode::Char('f') => self.events.send(AppEvent::StartCombat),
                KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Back),
                _ => {}
            },

            Screen::Inventory => match key_event.code {
                KeyCode::Up   | KeyCode::Char('k') => self.events.send(AppEvent::SelectUp),
                KeyCode::Down | KeyCode::Char('j') => self.events.send(AppEvent::SelectDown),
                KeyCode::Enter | KeyCode::Char('u') => self.events.send(AppEvent::InventoryUse),
                KeyCode::Char('e') => self.events.send(AppEvent::InventoryEquip),
                KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Back),
                _ => {}
            },

            Screen::Equipment => match key_event.code {
                KeyCode::Up   | KeyCode::Char('k') => self.events.send(AppEvent::SelectUp),
                KeyCode::Down | KeyCode::Char('j') => self.events.send(AppEvent::SelectDown),
                KeyCode::Enter | KeyCode::Char('u') => self.events.send(AppEvent::EquipmentUnequip),
                KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Back),
                _ => {}
            },

            Screen::Combat => match key_event.code {
                KeyCode::Char('1') | KeyCode::Char('m') => self.events.send(AppEvent::CombatSelectMelee),
                KeyCode::Char('2') | KeyCode::Char('r') => self.events.send(AppEvent::CombatSelectRanged),
                KeyCode::Char('3') | KeyCode::Char('c') => self.events.send(AppEvent::CombatSelectSpell),
                KeyCode::Up | KeyCode::Char('k') => self.events.send(AppEvent::CombatCycleOptionUp),
                KeyCode::Down | KeyCode::Char('j') => self.events.send(AppEvent::CombatCycleOptionDown),
                KeyCode::Tab => self.events.send(AppEvent::CombatCycleTarget),
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
            Screen::Inventory  => self.inventory.cursor_up(),
            Screen::Equipment  => cycle_cursor(&mut self.equipment_cursor, -1, EquipSlot::ALL.len()),
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
            Screen::Inventory  => self.inventory.cursor_down(),
            Screen::Equipment  => cycle_cursor(&mut self.equipment_cursor, 1, EquipSlot::ALL.len()),
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
                    let id = db::save_character(&self.pool, &self.creation).await?;
                    let character = db::load_character_by_id(&self.pool, id).await?;
                    let equipment = db::load_equipment(&self.pool, id).await?;
                    self.active_character = Some(character);
                    self.equipment = equipment;
                    self.screen = Screen::InGame;
                } else {
                    self.creation.step = self.creation.step.next();
                }
            }

            Screen::LoadGame => {
                if !self.saved_characters.is_empty() {
                    let character = self.saved_characters[self.load_cursor].clone();
                    let equipment = db::load_equipment(&self.pool, character.id).await?;
                    self.active_character = Some(character);
                    self.equipment = equipment;
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
            Screen::Skills | Screen::Inventory | Screen::Equipment => {
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
        self.combat = Some(CombatState::from_character_with_equipment(character, &self.equipment));
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

    fn cycle_combat_target(&mut self) {
        if let Some(combat) = self.combat.as_mut() {
            combat.cycle_target(1);
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

    // ── Inventory ─────────────────────────────────────────────────────────────

    async fn open_inventory(&mut self) -> color_eyre::Result<()> {
        let Some(ch) = &self.active_character else { return Ok(()); };
        let items = db::load_inventory(&self.pool, ch.id).await?;
        let equipment = db::load_equipment(&self.pool, ch.id).await?;
        self.inventory.items = items;
        self.inventory.cursor = 0;
        self.inventory.last_use_message = None;
        self.equipment = equipment;
        self.screen = Screen::Inventory;
        Ok(())
    }

    async fn use_inventory_item(&mut self) -> color_eyre::Result<()> {
        let Some(item) = self.inventory.selected().cloned() else { return Ok(()); };
        let Some(def) = item.def() else { return Ok(()); };
        if !def.is_usable() {
            self.inventory.last_use_message = Some(format!("{} cannot be used.", def.name));
            return Ok(());
        }

        let Some(ch) = self.active_character.as_mut() else { return Ok(()); };

        let message = if def.heal_amount > 0 {
            let before = ch.hp;
            ch.hp = (ch.hp + def.heal_amount).min(ch.max_hp);
            let healed = ch.hp - before;
            db::update_character_progress(&self.pool, ch.id, ch.hp, ch.xp, ch.gold).await?;
            format!("Used {}. Restored {} HP.", def.name, healed)
        } else {
            format!("Used {}. No effect.", def.name)
        };

        let character_id = ch.id;
        db::decrement_item(&self.pool, character_id, &item.item_type).await?;

        let items = db::load_inventory(&self.pool, character_id).await?;
        self.inventory.items = items;
        self.inventory.clamp_cursor();
        self.inventory.last_use_message = Some(message);
        Ok(())
    }

    async fn equip_selected_item(&mut self) -> color_eyre::Result<()> {
        let Some(item) = self.inventory.selected().cloned() else { return Ok(()); };
        let Some(def) = item.def() else { return Ok(()); };
        let Some(slot) = def.equip_slot else {
            self.inventory.last_use_message = Some(format!("{} cannot be equipped.", def.name));
            return Ok(());
        };
        let Some(ch) = &self.active_character else { return Ok(()); };
        let character_id = ch.id;

        // If something is already in this slot, return it to the bag first.
        if let Some(current_type) = self.equipment.get_slot(slot).map(|s| s.to_string()) {
            db::add_item(&self.pool, character_id, &current_type, 1).await?;
        }

        db::decrement_item(&self.pool, character_id, &item.item_type).await?;
        db::equip_item(&self.pool, character_id, slot, &item.item_type).await?;

        let items = db::load_inventory(&self.pool, character_id).await?;
        self.inventory.items = items;
        self.inventory.clamp_cursor();
        self.equipment = db::load_equipment(&self.pool, character_id).await?;
        self.inventory.last_use_message = Some(format!("Equipped {}.", def.name));
        Ok(())
    }

    // ── Equipment screen ──────────────────────────────────────────────────────

    async fn open_equipment(&mut self) -> color_eyre::Result<()> {
        let Some(ch) = &self.active_character else { return Ok(()); };
        self.equipment = db::load_equipment(&self.pool, ch.id).await?;
        self.equipment_cursor = 0;
        self.equipment_message = None;
        self.screen = Screen::Equipment;
        Ok(())
    }

    async fn unequip_item(&mut self) -> color_eyre::Result<()> {
        let slot = EquipSlot::ALL[self.equipment_cursor];
        let Some(item_type) = self.equipment.get_slot(slot).map(|s| s.to_string()) else {
            self.equipment_message = Some("Nothing equipped in this slot.".to_string());
            return Ok(());
        };
        let name = find_def(&item_type).map(|d| d.name).unwrap_or(&item_type).to_string();
        let Some(ch) = &self.active_character else { return Ok(()); };
        let character_id = ch.id;

        db::unequip_item(&self.pool, character_id, slot).await?;
        db::add_item(&self.pool, character_id, &item_type, 1).await?;

        self.equipment = db::load_equipment(&self.pool, character_id).await?;
        self.equipment_message = Some(format!("Unequipped {}.", name));
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
