use super::{App, Screen};
use crate::combat::CombatReward;
use crate::db;
use crate::inventory::{EquipSlot, ItemEffect, find_def};
use crate::world::{AreaId, ObjectiveKind, QuestId, VendorId, area_def, quest_def, vendor_def};
use rand::RngExt;

impl App {
    pub async fn rest_at_inn(&mut self) -> color_eyre::Result<()> {
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

    pub async fn use_inventory_item(&mut self) -> color_eyre::Result<()> {
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
                ItemEffect::CurePoison | ItemEffect::ApplyGuard(_) => {}
            }
        }
        db::remove_item(&self.pool, ch.id, &item.item_type, 1).await?;
        db::save_character_state(&self.pool, ch).await?;
        self.inventory.items = db::load_inventory(&self.pool, ch.id).await?;
        self.inventory.clamp_cursor();
        self.inventory.last_use_message = Some(format!("Used {}.", def.name));
        Ok(())
    }

    pub async fn equip_selected_item(&mut self) -> color_eyre::Result<()> {
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

    pub async fn unequip_item(&mut self) -> color_eyre::Result<()> {
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

    pub fn toggle_shop_mode(&mut self) {
        self.shop_buy_mode = !self.shop_buy_mode;
        self.shop_cursor = 0;
    }

    pub fn shop_cycle_vendor(&mut self, dir: i32) {
        super::input::cycle_cursor(&mut self.vendor_cursor, dir, VendorId::ALL.len());
        self.shop_cursor = 0;
    }

    pub async fn shop_transaction(&mut self) -> color_eyre::Result<()> {
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

    pub async fn accept_selected_quest(&mut self) -> color_eyre::Result<()> {
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

    pub async fn explore_selected(&mut self) -> color_eyre::Result<()> {
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

    pub async fn apply_combat_rewards_to_world(
        &mut self,
        character: &mut crate::character::SavedCharacter,
        reward: &CombatReward,
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
}
