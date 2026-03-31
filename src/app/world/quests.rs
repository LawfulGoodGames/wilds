use super::App;
use crate::combat::CombatReward;
use crate::combat::ability_def;
use crate::db;
use crate::inventory::find_def;
use crate::world::{
    NpcId, ObjectiveKind, QuestId, quest_completion_story_lines, quest_def, quest_item_miss_text,
};

impl App {
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

    pub(super) async fn auto_accept_story_quest_for_npc(
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

    pub(super) async fn apply_talk_objective(
        &mut self,
        npc: NpcId,
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

    pub(super) async fn apply_noncombat_quest_rewards(
        &mut self,
    ) -> color_eyre::Result<Vec<String>> {
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
