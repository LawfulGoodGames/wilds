use super::App;
use crate::combat::{CombatOutcome, CombatState, PlayerAction, ability_def};
use crate::db;
use crate::inventory::find_def;
use crate::world::quest_item_drop_is_relevant;

impl App {
    fn push_dialog_section(lines: &mut Vec<String>, title: &str, entries: Vec<String>) {
        if entries.is_empty() {
            return;
        }
        if !lines.is_empty() {
            lines.push(String::new());
        }
        lines.push(title.to_string());
        lines.extend(entries.into_iter().map(|entry| format!("• {entry}")));
    }

    pub async fn start_combat(&mut self, encounter_id: &str) -> color_eyre::Result<()> {
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
        self.screen = super::Screen::Combat;
        Ok(())
    }

    pub async fn handle_combat_action(&mut self) -> color_eyre::Result<()> {
        let action = match self
            .combat
            .as_ref()
            .map(|combat| combat.action_tab)
            .unwrap_or(crate::combat::ActionTab::Weapon)
        {
            crate::combat::ActionTab::Weapon => PlayerAction::UseWeapon,
            crate::combat::ActionTab::Ability => PlayerAction::UseAbility,
            crate::combat::ActionTab::Item => PlayerAction::UseItem,
        };
        self.handle_explicit_combat_action(action).await
    }

    pub async fn handle_explicit_combat_action(
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
            self.screen = super::Screen::Town;
            return Ok(());
        };
        match outcome {
            CombatOutcome::Won(mut reward) => {
                let character_id = character.id;
                let starting_gold = character.gold;
                character.resources = self
                    .combat
                    .as_ref()
                    .map(|combat| combat.player.resources)
                    .unwrap_or(character.resources);
                reward
                    .drops
                    .retain(|(item, _)| quest_item_drop_is_relevant(&self.world_state, item));
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
                        let mut lines = vec![format!("You win the {}.", reward.encounter_name)];
                        let reward_lines = {
                            let mut section = vec![format!("{} XP", reward.xp)];
                            section.push(format!("{} gold", reward.gold));
                            if !reward.drops.is_empty() {
                                section.extend(reward.drops.iter().map(|(item, qty)| {
                                    let item_name = find_def(item)
                                        .map(|def| def.name)
                                        .unwrap_or(item.as_str());
                                    if *qty == 1 {
                                        format!("Loot: {item_name}")
                                    } else {
                                        format!("Loot: {item_name} x{qty}")
                                    }
                                }));
                            }
                            section
                        };
                        Self::push_dialog_section(&mut lines, "Rewards", reward_lines);

                        let mut progression_lines = vec![];
                        if level_up.levels_gained > 0 {
                            progression_lines.push(format!(
                                "Level up to {}. +{} HP, +{} Mana, +{} Stamina, +{} proficiency points.",
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
                            progression_lines
                                .push(format!("New abilities unlocked: {ability_names}"));
                        }
                        Self::push_dialog_section(
                            &mut lines,
                            "Progression",
                            progression_lines,
                        );
                        Self::push_dialog_section(&mut lines, "Quests", quest_lines);
                        Self::push_dialog_section(
                            &mut lines,
                            "Achievements",
                            achievement_lines,
                        );
                        lines
                    },
                    super::Screen::Town,
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
                    super::Screen::Town,
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
                    super::Screen::Town,
                );
            }
            CombatOutcome::Ongoing => {}
        }
        Ok(())
    }
}
