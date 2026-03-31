use crate::achievements::AchievementDef;
use crate::app::App;
use crate::character::{MajorSkill, level_progress_pct, xp_to_next_level};
use crate::db;
use crate::inventory::EquipSlot;

impl App {
    pub(super) async fn achievement_increment(
        &mut self,
        character_id: i64,
        metric: &str,
        amount: i32,
    ) -> color_eyre::Result<Vec<String>> {
        let unlocked = self.achievements.record_increment(metric, amount);
        let reward_messages = self
            .grant_achievement_rewards(character_id, &unlocked)
            .await?;
        db::save_achievement_metric(
            &self.pool,
            character_id,
            metric,
            self.achievements.progress_for(metric),
        )
        .await?;
        Ok(reward_messages)
    }

    pub(super) async fn achievement_set_max(
        &mut self,
        character_id: i64,
        metric: &str,
        value: i32,
    ) -> color_eyre::Result<Vec<String>> {
        let unlocked = self.achievements.record_max(metric, value);
        let reward_messages = self
            .grant_achievement_rewards(character_id, &unlocked)
            .await?;
        db::save_achievement_metric(
            &self.pool,
            character_id,
            metric,
            self.achievements.progress_for(metric),
        )
        .await?;
        Ok(reward_messages)
    }

    async fn grant_achievement_rewards(
        &mut self,
        character_id: i64,
        unlocked: &[AchievementDef],
    ) -> color_eyre::Result<Vec<String>> {
        if unlocked.is_empty() {
            return Ok(vec![]);
        }

        let total_unlocked = self.achievements.unlocked_count();
        let start_ordinal = total_unlocked.saturating_sub(unlocked.len());
        let mut reward_messages = Vec::with_capacity(unlocked.len());
        let mut total_xp = 0;
        let mut total_gold = 0;

        for (idx, def) in unlocked.iter().enumerate() {
            let ordinal = start_ordinal + idx + 1;
            let reward_tier = ((ordinal.max(1) as f64).ln().floor() as i32 + 1).max(1);
            let xp_reward = reward_tier * 3;
            let gold_reward = reward_tier * 2;
            total_xp += xp_reward;
            total_gold += gold_reward;
            reward_messages.push(format!(
                "{} (+{} XP, +{} gold)",
                def.name, xp_reward, gold_reward
            ));
        }

        if let Some(ch) = self.active_character.as_mut() {
            ch.gold += total_gold;
            let level_up = ch.apply_xp_gain(total_xp);
            db::save_character_state(&self.pool, ch).await?;
            if level_up.levels_gained > 0 {
                reward_messages.push(format!("Level up! Reached level {}.", ch.level));
            }
        } else {
            let _ = character_id;
        }

        Ok(reward_messages)
    }

    pub(super) async fn refresh_meta_achievement_metrics(
        &mut self,
        character_id: i64,
    ) -> color_eyre::Result<Vec<String>> {
        let mut unlocked = vec![];
        let Some(ch) = &self.active_character else {
            return Ok(unlocked);
        };
        let best_prof = MajorSkill::ALL
            .iter()
            .map(|skill| ch.major_skill(*skill))
            .chain(ch.proficiencies.iter().map(|skill| skill.level() as i32))
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

    pub(super) fn append_achievement_lines(lines: &mut Vec<String>, unlocked: Vec<String>) {
        for name in unlocked {
            lines.push(format!("Achievement unlocked: {name}."));
        }
    }

    pub(super) async fn load_session(&mut self, character_id: i64) -> color_eyre::Result<()> {
        self.active_character = Some(db::load_character_by_id(&self.pool, character_id).await?);
        self.world_state = db::load_world_state(&self.pool, character_id).await?;
        self.equipment = db::load_equipment(&self.pool, character_id).await?;
        self.inventory.items = db::load_inventory(&self.pool, character_id).await?;
        self.achievements = db::load_achievement_state(&self.pool, character_id).await?;
        self.combat = None;
        self.refresh_meta_achievement_metrics(character_id).await?;
        Ok(())
    }
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
