use super::{App, DialogueChoice, Screen};
use crate::combat::CombatReward;
use crate::combat::ability_def;
use crate::db;
use crate::inventory::{EquipSlot, ItemEffect, find_def};
use crate::world::{
    AreaId, NpcId, ObjectiveKind, QuestId, VendorId, area_def, quest_completion_story_lines,
    quest_def, quest_item_miss_text, vendor_def,
};
use rand::RngExt;

impl App {
    fn objective_lead_text(&self, objective: &ObjectiveKind) -> String {
        match objective {
            ObjectiveKind::TalkToNpc { npc } => format!("Speak with {}.", npc.name()),
            ObjectiveKind::VisitArea { area } => format!("Travel to {}.", area.label()),
            ObjectiveKind::KillFamily { family, count } => {
                format!("Defeat {count} {family} encounter(s).")
            }
            ObjectiveKind::OwnItem { item_type, count } => {
                let item_name = find_def(item_type)
                    .map(|item| item.name)
                    .unwrap_or(item_type);
                format!("Recover {count} {item_name}.")
            }
        }
    }

    fn current_story_lead_line(&self) -> Option<String> {
        let quest_id = self.world_state.current_story_lead()?;
        let def = quest_def(quest_id.id())?;
        let lead = self
            .world_state
            .active_quest(quest_id)
            .and_then(|progress| def.objectives.get(progress.objective_index))
            .map(|objective| self.objective_lead_text(&objective.kind))
            .or_else(|| {
                def.objectives
                    .first()
                    .map(|objective| self.objective_lead_text(&objective.kind))
            })?;
        Some(format!("Next lead: {}. {}", def.name, lead))
    }

    pub fn visible_quest_ids(&self) -> Vec<QuestId> {
        QuestId::ALL
            .iter()
            .copied()
            .filter(|quest_id| {
                let is_complete = self.world_state.has_completed(*quest_id);
                let is_active = self.world_state.active_quest(*quest_id).is_some();
                let is_available = self.world_state.can_accept_quest(*quest_id);
                let is_locked = !is_complete && !is_active && !is_available;
                (!is_complete || self.quest_show_completed)
                    && (!is_locked || self.quest_show_locked)
            })
            .collect()
    }

    pub fn selected_visible_quest_id(&self) -> Option<QuestId> {
        let visible = self.visible_quest_ids();
        visible
            .get(self.quest_cursor.min(visible.len().saturating_sub(1)))
            .copied()
    }

    pub fn toggle_quest_completed_filter(&mut self) {
        self.quest_show_completed = !self.quest_show_completed;
        self.quest_cursor = 0;
    }

    pub fn toggle_quest_locked_filter(&mut self) {
        self.quest_show_locked = !self.quest_show_locked;
        self.quest_cursor = 0;
    }

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
        self.detail_scroll = 0;
    }

    pub fn shop_cycle_vendor(&mut self, dir: i32) {
        super::input::cycle_cursor(&mut self.vendor_cursor, dir, VendorId::ALL.len());
        self.shop_cursor = 0;
        self.detail_scroll = 0;
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

    pub async fn talk_to_selected_npc(&mut self) -> color_eyre::Result<()> {
        let npc = NpcId::ALL[self.npc_cursor];
        let accepted_line = self.auto_accept_story_quest_for_npc(npc).await?;
        let mut quest_lines = self.apply_talk_objective(npc).await?;
        let followup_line = self.auto_accept_story_quest_for_npc(npc).await?;
        if let Some(line) = accepted_line {
            quest_lines.insert(0, line);
        }
        if let Some(line) = followup_line {
            quest_lines.push(line);
            if let Some(lead) = self.current_story_lead_line() {
                quest_lines.push(lead);
            }
        }
        let (title, mut lines, choices) = self.dialogue_for_npc(npc);
        if !quest_lines.is_empty() {
            lines.push(String::new());
            lines.append(&mut quest_lines);
        }
        if choices.is_empty() {
            self.open_dialog(title, lines, Screen::People);
        } else {
            self.open_choice_dialog(title, lines, choices, npc, Screen::People);
        }
        Ok(())
    }

    pub async fn resolve_dialogue_choice(&mut self) -> color_eyre::Result<()> {
        let Some(choice) = self
            .dialogue_choices
            .get(
                self.dialogue_cursor
                    .min(self.dialogue_choices.len().saturating_sub(1)),
            )
            .cloned()
        else {
            self.go_back().await?;
            return Ok(());
        };
        if let Some(flag) = &choice.memory_flag {
            self.world_state.set_flag(flag);
            if let Some(ch) = &self.active_character {
                db::save_world_state(&self.pool, ch.id, &self.world_state).await?;
            }
        }
        self.dialogue_lines.extend(choice.response_lines);
        self.dialogue_choices.clear();
        self.dialogue_cursor = 0;
        let mut combined_status = None;
        if let Some(npc) = self.dialogue_npc {
            if let Some(line) = self.auto_accept_story_quest_for_npc(npc).await? {
                self.dialogue_lines.push(String::new());
                self.dialogue_lines.push(line.clone());
                if let Some(quest_id) = self.world_state.current_story_lead() {
                    if let Some(def) = quest_def(quest_id.id()) {
                        self.dialogue_lines.push(def.summary.to_string());
                    }
                }
                combined_status = Some(format!("Story updated: {line}"));
            }
        }
        if let Some(message) = choice.status_message {
            combined_status = Some(match combined_status {
                Some(prefix) => format!("{prefix} {message}"),
                None => message,
            });
        }
        self.status_message = combined_status;
        Ok(())
    }

    async fn auto_accept_story_quest_for_npc(
        &mut self,
        npc: NpcId,
    ) -> color_eyre::Result<Option<String>> {
        let Some(quest_id) = self.world_state.current_story_lead() else {
            return Ok(None);
        };
        let Some(def) = quest_def(quest_id.id()) else {
            return Ok(None);
        };
        if def.giver != npc || !self.world_state.can_accept_quest(quest_id) {
            return Ok(None);
        }
        if !self.world_state.accept_quest(quest_id) {
            return Ok(None);
        }
        if let Some(ch) = &self.active_character {
            db::save_world_state(&self.pool, ch.id, &self.world_state).await?;
        }
        Ok(Some(format!("New quest: {}.", def.name)))
    }

    pub async fn accept_selected_quest(&mut self) -> color_eyre::Result<()> {
        let Some(ch) = &self.active_character else {
            return Ok(());
        };
        let Some(quest_id) = self.selected_visible_quest_id() else {
            self.status_message = Some("No quests match the current filters.".to_string());
            return Ok(());
        };
        if !self.world_state.can_accept_quest(quest_id) {
            self.status_message = Some(
                "That story step is not ready yet. Follow the current lead first.".to_string(),
            );
            return Ok(());
        }
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
        let reward_lines = self.apply_noncombat_quest_rewards().await?;
        if let Some(line) = reward_lines.last() {
            self.status_message = Some(line.clone());
        }
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
                    let owned_before = self
                        .inventory
                        .items
                        .iter()
                        .find(|item| item.item_type == *item_type)
                        .map(|item| item.quantity)
                        .unwrap_or(0);
                    let dropped_qty = reward
                        .drops
                        .iter()
                        .filter(|(item, _)| item == item_type)
                        .map(|(_, qty)| *qty)
                        .sum::<i32>();
                    let total = owned_before + dropped_qty;
                    if total >= *count {
                        progress.objective_index += 1;
                        progress.progress = total;
                    } else if owned_before < *count && dropped_qty == 0 {
                        if let Some(item_name) = find_def(item_type).map(|item| item.name) {
                            if let Some(text) = quest_item_miss_text(
                                def.id.id(),
                                item_type,
                                item_name,
                                &reward.encounter_name,
                                &reward.defeated_families,
                                &reward.environment_tags,
                            ) {
                                lines.push(text);
                            }
                        }
                    }
                }
                ObjectiveKind::VisitArea { .. } => {}
                ObjectiveKind::TalkToNpc { .. } => {}
            }
        }

        for line in self.complete_ready_quests().await? {
            character.gold += line.1;
            character.apply_xp_gain(line.0);
            if let Some(item) = line.2.clone() {
                db::add_item(&self.pool, character.id, &item, line.3).await?;
            }
            lines.extend(line.4);
        }
        db::save_world_state(&self.pool, character.id, &self.world_state).await?;
        Ok(lines)
    }

    async fn apply_talk_objective(&mut self, npc: NpcId) -> color_eyre::Result<Vec<String>> {
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
            if let ObjectiveKind::TalkToNpc { npc: needed } =
                &def.objectives[progress.objective_index].kind
            {
                if *needed == npc {
                    progress.progress = 1;
                    progress.objective_index += 1;
                    lines.push(format!("Quest updated: {}.", def.name));
                }
            }
        }
        lines.extend(self.apply_noncombat_quest_rewards().await?);
        if let Some(ch) = &self.active_character {
            db::save_world_state(&self.pool, ch.id, &self.world_state).await?;
        }
        Ok(lines)
    }

    async fn apply_noncombat_quest_rewards(&mut self) -> color_eyre::Result<Vec<String>> {
        let rewards = self.complete_ready_quests().await?;
        if rewards.is_empty() {
            return Ok(vec![]);
        }
        let Some(character_id) = self.active_character.as_ref().map(|ch| ch.id) else {
            return Ok(vec![]);
        };
        let mut lines = vec![];
        {
            let Some(ch) = self.active_character.as_mut() else {
                return Ok(vec![]);
            };
            for reward in &rewards {
                ch.gold += reward.1;
                let level_up = ch.apply_xp_gain(reward.0);
                lines.extend(reward.4.clone());
                lines.push(format!("Received {} XP and {} gold.", reward.0, reward.1));
                if let Some(item) = reward.2.as_deref().and_then(find_def) {
                    lines.push(format!("Received {} x{}.", item.name, reward.3));
                }
                if level_up.levels_gained > 0 {
                    lines.push(format!("Level up! Reached level {}.", ch.level));
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
                }
            }
            db::save_character_state(&self.pool, ch).await?;
        }
        for reward in &rewards {
            if let Some(item) = &reward.2 {
                db::add_item(&self.pool, character_id, item, reward.3).await?;
            }
        }
        self.inventory.items = db::load_inventory(&self.pool, character_id).await?;
        db::save_world_state(&self.pool, character_id, &self.world_state).await?;
        Ok(lines)
    }

    fn dialogue_for_npc(&self, npc: NpcId) -> (&'static str, Vec<String>, Vec<DialogueChoice>) {
        match npc {
            NpcId::CaptainHedd => {
                if !self.world_state.has_flag("hedd_motive_duty")
                    && !self.world_state.has_flag("hedd_motive_coin")
                    && !self.world_state.has_flag("hedd_motive_truth")
                {
                    (
                        "Captain Hedd",
                        vec![
                            "Rain beads on Captain Hedd's coat while lanterns sway over the square.".to_string(),
                            "\"Good. A steady pair of hands.\" He taps a rough patrol map. \"One of ours is missing in the Whispering Woods. Start there and bring back the truth, not tavern fog.\"".to_string(),
                            "\"Before you go, answer me plain. Why stand the line for Hearthmere?\"".to_string(),
                        ],
                        vec![
                            DialogueChoice {
                                label: "Because someone has to hold the wall.".to_string(),
                                response_lines: vec![
                                    "Hedd gives a short nod. \"Duty keeps towns alive longer than luck does. Remember that.\"".to_string(),
                                ],
                                memory_flag: Some("hedd_motive_duty".to_string()),
                                status_message: Some("Captain Hedd remembers your sense of duty.".to_string()),
                            },
                            DialogueChoice {
                                label: "Because danger pays better than hunger.".to_string(),
                                response_lines: vec![
                                    "Hedd snorts. \"Honest enough. Stay alive and there might even be coin left when this is done.\"".to_string(),
                                ],
                                memory_flag: Some("hedd_motive_coin".to_string()),
                                status_message: Some("Captain Hedd clocks your eye for coin.".to_string()),
                            },
                            DialogueChoice {
                                label: "Because I want to be the one who sees what's coming.".to_string(),
                                response_lines: vec![
                                    "\"Then keep your eyes open wider than the last patrol did,\" Hedd says, pushing the map toward you.".to_string(),
                                ],
                                memory_flag: Some("hedd_motive_truth".to_string()),
                                status_message: Some("Captain Hedd notes your hunger for the truth.".to_string()),
                            },
                        ],
                    )
                } else if self.world_state.active_quest(QuestId::MissingOnTheWatch).is_some() {
                    (
                        "Captain Hedd",
                        vec![
                            "\"Work the western edge of the Whispering Woods and trust fresh silence less than fresh tracks,\" Hedd says.".to_string(),
                            "\"If the woods went too quiet for the patrol, they'll try the same trick on you there too.\"".to_string(),
                        ],
                        vec![],
                    )
                } else if self.world_state.active_quest(QuestId::WordToTheCaptain).is_some() {
                    (
                        "Captain Hedd",
                        vec![
                            "You lay out the story of the barrow and the marching dead. Hedd goes still.".to_string(),
                            "\"Then this isn't a bad season. It's the front edge of a war.\" He folds the report with soldier's care.".to_string(),
                            "\"Take this to Sel. If she names what raised them, I can start convincing the town before panic does it for me.\"".to_string(),
                        ],
                        vec![],
                    )
                } else {
                    (
                        "Captain Hedd",
                        vec![
                            "\"Keep your pack ready and your head low,\" Hedd says. \"Hearthmere doesn't get quiet by accident anymore.\"".to_string(),
                        ],
                        vec![],
                    )
                }
            }
            NpcId::ScoutMira => {
                if self.world_state.active_quest(QuestId::ReportToMira).is_some() {
                    (
                        "Scout Mira",
                        vec![
                            "Mira studies the mud on your boots before she studies your face.".to_string(),
                            "\"So the patrol crossed the Whispering Woods and never came back from the Sunken Road. That's not wolves.\"".to_string(),
                            "\"Do we stay close and map every step, or press hard before the trail cools?\"".to_string(),
                        ],
                        vec![
                            DialogueChoice {
                                label: "Map every step. We only get one first read.".to_string(),
                                response_lines: vec![
                                    "\"Careful work wins long hunts,\" Mira says. \"Good. We'll read this trail clean.\"".to_string(),
                                ],
                                memory_flag: Some("mira_method_careful".to_string()),
                                status_message: Some("Mira notes that you favor caution.".to_string()),
                            },
                            DialogueChoice {
                                label: "Press hard. Whoever did this is still ahead of us.".to_string(),
                                response_lines: vec![
                                    "Mira smiles without warmth. \"Fast, then. Just don't mistake speed for silence.\"".to_string(),
                                ],
                                memory_flag: Some("mira_method_bold".to_string()),
                                status_message: Some("Mira notes that you press the advantage.".to_string()),
                            },
                        ],
                    )
                } else {
                    (
                        "Scout Mira",
                        vec![
                            "\"The forest only lies to people who rush it,\" Mira says. \"Listen long enough and it names the thing that frightened it.\"".to_string(),
                        ],
                        vec![],
                    )
                }
            }
            NpcId::QuartermasterVale => (
                "Quartermaster Vale",
                vec![
                    "\"Every cart lost on the Sunken Road becomes someone else's courage problem,\" Vale says, arms full of tally slips.".to_string(),
                    "\"If you find maps, seals, or anything the raiders missed on the road, bring it back before the rain takes the ink.\"".to_string(),
                ],
                vec![],
            ),
            NpcId::ArcanistSel => {
                if self.world_state.active_quest(QuestId::AshOnTheWax).is_some() {
                    (
                        "Arcanist Sel",
                        vec![
                            "Sel turns the blackened seal under a lamp until ash glitters in its wax.".to_string(),
                            "\"This mark belonged to one court only, and it should have died with its master.\"".to_string(),
                            "\"If this ash rides with raiders on the Sunken Road, then someone is testing the roads before they test the gates.\"".to_string(),
                        ],
                        vec![],
                    )
                } else if self.world_state.active_quest(QuestId::CrownInCinders).is_some() {
                    (
                        "Arcanist Sel",
                        vec![
                            "Sel reads Hedd's report twice before speaking.".to_string(),
                            "\"The dead in the Ashen Barrow, the ash sigil, the sealed Sunken Road. I know the hand behind that pattern.\"".to_string(),
                            "\"The defamed Mage King did not die in exile. He fled the kingdom, and now he is building an army of the dead to march on the capital.\"".to_string(),
                            "\"Hearthmere is only the first place close enough to hear him testing the drum.\"".to_string(),
                        ],
                        vec![],
                    )
                } else if self.world_state.active_quest(QuestId::Gravewind).is_some() {
                    (
                        "Arcanist Sel",
                        vec![
                            "Sel wraps the seal in cloth as though even its wax should not touch bare skin.".to_string(),
                            "\"If the mark has reached the road, the answer will be waiting where the dead were first taught to stand again,\" she says.".to_string(),
                            "\"Go to the Ashen Barrow. Whatever is waking there is only the beginning.\"".to_string(),
                        ],
                        vec![],
                    )
                } else {
                    (
                        "Arcanist Sel",
                        vec![
                            "\"Old magic rarely wakes alone,\" Sel says. \"If something has stirred, something else called it.\"".to_string(),
                        ],
                        vec![],
                    )
                }
            }
            NpcId::InnkeeperBrin => (
                "Innkeeper Brin",
                vec![
                    "\"Hearthmere still eats, still sings, and still sweeps blood off the stones by dawn,\" Brin says, polishing a cup.".to_string(),
                    "\"That means we're not beaten yet. It also means you should hear every rumor twice before believing it.\"".to_string(),
                ],
                vec![],
            ),
        }
    }

    async fn complete_ready_quests(
        &mut self,
    ) -> color_eyre::Result<Vec<(i32, i32, Option<String>, i32, Vec<String>)>> {
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
            let mut lines = vec![format!("Quest complete: {}.", def.name)];
            lines.extend(
                quest_completion_story_lines(def.id.id())
                    .iter()
                    .map(|line| (*line).to_string()),
            );
            if let Some(lead) = self.current_story_lead_line() {
                lines.push(lead);
            }
            rewards.push((
                def.rewards.xp,
                def.rewards.gold,
                def.rewards.item_type.map(|item| item.to_string()),
                def.rewards.item_qty,
                lines,
            ));
        }
        Ok(rewards)
    }
}

#[cfg(test)]
mod tests;
