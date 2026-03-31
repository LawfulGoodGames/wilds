use rand::RngExt;

use crate::app::{ActiveTraining, App, ProficiencyTarget, TRAINING_TICKS_PER_HOUR};
use crate::character::{
    MAX_COMBAT_PROFICIENCY_RANK, MAX_PROFICIENCY_LEVEL, MajorSkill, level_progress_pct,
    major_study_plan_for_xp, study_plan, xp_to_next_level,
};
use crate::db;
use crate::inventory::EquipSlot;

impl App {
    pub(super) async fn train_selected_proficiency(&mut self) -> color_eyre::Result<()> {
        if let Some(training) = &self.active_training {
            self.status_message = Some(format!(
                "You are already studying {}. Wait for the current session to finish.",
                training.target.name()
            ));
            return Ok(());
        }

        let Some(ch) = self.active_character.as_mut() else {
            return Ok(());
        };
        let (target, plan) = if self.character_cursor < MajorSkill::ALL.len() {
            let skill = MajorSkill::ALL[self.character_cursor];
            if ch.major_skill(skill) >= MAX_COMBAT_PROFICIENCY_RANK {
                self.status_message = Some(format!("{} is already mastered.", skill.full_name()));
                return Ok(());
            }
            (
                ProficiencyTarget::Major(skill),
                major_study_plan_for_xp(skill, ch.major_skill_xp(skill), &ch.stats),
            )
        } else {
            let skill_idx = (self.character_cursor - MajorSkill::ALL.len())
                .min(ch.proficiencies.len().saturating_sub(1));
            let Some(skill) = ch.proficiencies.get_mut(skill_idx) else {
                return Ok(());
            };
            if skill.level() >= MAX_PROFICIENCY_LEVEL {
                self.status_message = Some(format!("{} is already mastered.", skill.kind.name()));
                return Ok(());
            }
            (
                ProficiencyTarget::Minor(skill.kind),
                study_plan(skill.kind, skill.xp, &ch.stats),
            )
        };
        self.recent_training_level_up = None;
        self.active_training = Some(ActiveTraining {
            target,
            total_ticks: (plan.hours.max(1) as u32) * TRAINING_TICKS_PER_HOUR,
            elapsed_ticks: 0,
            hours: plan.hours,
            success_chance: plan.success_chance,
            success_gain: plan.success_xp,
            failure_gain: plan.failure_xp,
        });
        self.status_message = Some(format!(
            "You begin studying {}. Progress will complete over time.",
            target.name(),
        ));
        Ok(())
    }

    pub(super) async fn resolve_training_completion(&mut self) -> color_eyre::Result<()> {
        let Some(training) = self.active_training.take() else {
            return Ok(());
        };
        let roll = rand::rng().random_range(1..=100);
        let success = roll <= training.success_chance;
        let gain = if success {
            training.success_gain
        } else {
            training.failure_gain
        };
        let Some((character_id, target, before_level, after_level, maybe_xp)) = ({
            let Some(ch) = self.active_character.as_mut() else {
                return Ok(());
            };
            match training.target {
                ProficiencyTarget::Major(skill) => {
                    let before = ch.major_skill(skill);
                    let new_xp = ch.major_skill_xp(skill) + gain;
                    ch.set_major_skill_xp(skill, new_xp);
                    let after = ch.major_skill(skill);
                    Some((ch.id, ProficiencyTarget::Major(skill), before, after, None))
                }
                ProficiencyTarget::Minor(skill_kind) => {
                    let Some(skill) = ch
                        .proficiencies
                        .iter_mut()
                        .find(|skill| skill.kind == skill_kind)
                    else {
                        return Ok(());
                    };
                    let before = skill.level() as i32;
                    skill.xp += gain;
                    let after = skill.level() as i32;
                    Some((
                        ch.id,
                        ProficiencyTarget::Minor(skill.kind),
                        before,
                        after,
                        Some(skill.xp),
                    ))
                }
            }
        }) else {
            return Ok(());
        };
        self.world_state.advance_time(training.hours);

        if let (ProficiencyTarget::Minor(skill_kind), Some(new_xp)) = (target, maybe_xp) {
            db::save_proficiency_xp(&self.pool, character_id, skill_kind, new_xp).await?;
        } else if let Some(ch) = &self.active_character {
            db::save_character_state(&self.pool, ch).await?;
        }
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
            "{result}: {} training finished after {}h. Roll {} vs {}%, gained {} progress.",
            target.name(),
            training.hours,
            roll,
            training.success_chance,
            gain
        );
        if after_level > before_level {
            self.recent_training_level_up = Some((target, after_level));
            message.push_str(&format!(" Rank up to {}.", after_level));
        }
        if let Some(name) = unlocked.last() {
            message.push_str(&format!(" Achievement unlocked: {name}."));
        }
        self.status_message = Some(message);
        Ok(())
    }

    pub(super) async fn achievement_increment(
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

    pub(super) async fn achievement_set_max(
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
