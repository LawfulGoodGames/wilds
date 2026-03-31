use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::achievements;
use crate::app::{App, CharacterTab, MenuItem, Screen, TownAction};
use crate::character::{Class, CreationStep, GearPackage, Race};
use crate::combat::ActionTab;
use crate::event::AppEvent;
use crate::inventory::EquipSlot;
use crate::settings::OPTIONS_COUNT;
use crate::world::{AreaId, QuestId};

impl App {
    pub(super) fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
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
                KeyCode::Tab => self.events.send(AppEvent::CombatNextTab),
                KeyCode::Char('t') => self.events.send(AppEvent::CombatCycleTarget),
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

    pub(super) fn select_up(&mut self) {
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
                achievements::achievement_defs().len(),
            ),
            Screen::Shop => self.shop_cursor = self.shop_cursor.saturating_sub(1),
            Screen::CharacterCreation => match self.creation.step {
                CreationStep::Race => {
                    cycle_cursor(&mut self.creation.race_cursor, -1, Race::ALL.len())
                }
                CreationStep::Class => {
                    cycle_cursor(&mut self.creation.class_cursor, -1, Class::ALL.len())
                }
                CreationStep::Stats => {
                    let count = self.creation.proficiency_count();
                    cycle_cursor(&mut self.creation.stat_cursor, -1, count);
                }
                CreationStep::Gear => {
                    cycle_cursor(&mut self.creation.gear_cursor, -1, GearPackage::ALL.len())
                }
                _ => {}
            },
            _ => {}
        }
    }

    pub(super) fn select_down(&mut self) {
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
                achievements::achievement_defs().len(),
            ),
            Screen::Shop => self.shop_cursor += 1,
            Screen::CharacterCreation => match self.creation.step {
                CreationStep::Race => {
                    cycle_cursor(&mut self.creation.race_cursor, 1, Race::ALL.len())
                }
                CreationStep::Class => {
                    cycle_cursor(&mut self.creation.class_cursor, 1, Class::ALL.len())
                }
                CreationStep::Stats => {
                    let count = self.creation.proficiency_count();
                    cycle_cursor(&mut self.creation.stat_cursor, 1, count);
                }
                CreationStep::Gear => {
                    cycle_cursor(&mut self.creation.gear_cursor, 1, GearPackage::ALL.len())
                }
                _ => {}
            },
            _ => {}
        }
    }

    pub(super) fn handle_left(&mut self) {
        match self.screen {
            Screen::Options => self.change_option(-1),
            Screen::CharacterCreation => self.creation.adjust_stat(-1),
            Screen::CharacterSheet => self.change_character_tab(-1),
            _ => {}
        }
    }

    pub(super) async fn handle_right(&mut self) -> color_eyre::Result<()> {
        match self.screen {
            Screen::Options => self.change_option(1),
            Screen::CharacterCreation => self.creation.adjust_stat(1),
            Screen::CharacterSheet => self.change_character_tab(1),
            Screen::Inventory => self.equip_selected_item().await?,
            _ => {}
        }
        Ok(())
    }

    pub(super) fn change_option(&mut self, dir: i32) {
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

    pub(super) fn set_combat_tab(&mut self, tab: ActionTab) {
        if let Some(combat) = self.combat.as_mut() {
            combat.set_tab(tab);
        }
    }

    pub(super) fn cycle_combat_tab(&mut self, dir: i32) {
        if let Some(combat) = self.combat.as_mut() {
            combat.cycle_tab(dir);
        }
    }

    pub(super) fn cycle_combat_option(&mut self, dir: i32) {
        if let Some(combat) = self.combat.as_mut() {
            combat.cycle_selection(dir);
        }
    }

    pub(super) fn cycle_combat_target(&mut self, dir: i32) {
        if let Some(combat) = self.combat.as_mut() {
            combat.cycle_target(dir);
        }
    }

    pub(super) fn change_character_tab(&mut self, dir: i32) {
        let idx = CharacterTab::ALL
            .iter()
            .position(|tab| *tab == self.character_tab)
            .unwrap_or(0);
        let next = (idx as i32 + dir).rem_euclid(CharacterTab::ALL.len() as i32) as usize;
        self.character_tab = CharacterTab::ALL[next];
        self.character_cursor = 0;
    }

    pub(super) fn next_character_tab(&mut self) {
        self.change_character_tab(1);
    }
}

pub(super) fn cycle_cursor(cursor: &mut usize, dir: i32, len: usize) {
    if len == 0 {
        *cursor = 0;
        return;
    }
    *cursor = ((*cursor as i32 + dir).rem_euclid(len as i32)) as usize;
}
