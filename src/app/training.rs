use rand::RngExt;

use crate::app::{
    App, ProficiencyTarget, TrainingPhase, TrainingResult, TrainingSession, TrainingTier,
};
use crate::character::{
    MAX_COMBAT_PROFICIENCY_RANK, MAX_PROFICIENCY_LEVEL, MajorSkill,
    training_session_plan_for_major, training_session_plan_for_minor,
};
use crate::db;

const TRAINING_ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz";

impl App {
    pub(super) fn training_cursor(&mut self, dir: i32) {
        if self.training.session.is_some() {
            return;
        }
        let len = self.training_entry_count();
        if len == 0 {
            self.training.cursor = 0;
            return;
        }
        self.training.cursor =
            ((self.training.cursor as i32 + dir).rem_euclid(len as i32)) as usize;
        self.training.result = None;
        self.status_message = None;
    }

    pub(super) async fn start_training_session(&mut self) -> color_eyre::Result<()> {
        if self.training.session.is_some() {
            return Ok(());
        }
        let Some(ch) = &self.active_character else {
            return Ok(());
        };
        let Some(target) = self.selected_training_target() else {
            return Ok(());
        };
        let plan = match target {
            ProficiencyTarget::Major(skill) => {
                if ch.major_skill(skill) >= MAX_COMBAT_PROFICIENCY_RANK {
                    self.status_message =
                        Some(format!("{} is already mastered.", skill.full_name()));
                    return Ok(());
                }
                training_session_plan_for_major(skill, ch.major_skill_xp(skill), &ch.stats)
            }
            ProficiencyTarget::Minor(skill) => {
                let Some(data) = ch.proficiencies.iter().find(|entry| entry.kind == skill) else {
                    return Ok(());
                };
                if data.level() >= MAX_PROFICIENCY_LEVEL {
                    self.status_message = Some(format!("{} is already mastered.", skill.name()));
                    return Ok(());
                }
                training_session_plan_for_minor(skill, data.xp, &ch.stats)
            }
        };

        let sequence = random_training_sequence(plan.beats);
        let reveal_ticks = plan
            .response_ticks
            .saturating_mul(plan.beats as u32)
            .max(30);
        self.training.result = None;
        self.training.session = Some(TrainingSession {
            target,
            plan,
            phase: TrainingPhase::Showing,
            sequence,
            reveal_ticks_remaining: reveal_ticks,
            input_index: 0,
            score: 0,
            max_score: plan.beats as i32 * 3,
            hits: 0,
            misses: 0,
        });
        self.status_message = Some(format!(
            "Training {} begins. Memorize the letter code before it disappears.",
            target.name()
        ));
        Ok(())
    }

    pub(super) async fn tick_training_session(&mut self) -> color_eyre::Result<()> {
        let transition_to_input = if let Some(session) = self.training.session.as_mut() {
            if session.phase == TrainingPhase::Showing {
                if session.reveal_ticks_remaining > 0 {
                    session.reveal_ticks_remaining -= 1;
                }
                session.reveal_ticks_remaining == 0
            } else {
                false
            }
        } else {
            false
        };

        if transition_to_input {
            if let Some(session) = self.training.session.as_mut() {
                session.phase = TrainingPhase::Input;
            }
            self.status_message =
                Some("The code is hidden. Type the letters back in the same order.".to_string());
        }
        Ok(())
    }

    pub(super) async fn handle_training_input(&mut self, key: char) -> color_eyre::Result<()> {
        let Some(mut session) = self.training.session.take() else {
            return Ok(());
        };

        if session.phase != TrainingPhase::Input {
            self.training.session = Some(session);
            return Ok(());
        }

        if !key.is_ascii_alphabetic() {
            self.training.session = Some(session);
            return Ok(());
        }

        let normalized = key.to_ascii_lowercase();
        let expected = session.sequence.get(session.input_index).copied();
        match expected {
            Some(ch) if ch == normalized => {
                session.hits += 1;
                session.score += 3;
            }
            Some(_) => {
                session.misses += 1;
            }
            None => {}
        }
        session.input_index += 1;

        if session.input_index >= session.sequence.len() {
            self.resolve_training_session(session).await?;
        } else {
            self.training.session = Some(session);
        }
        Ok(())
    }

    pub(super) fn cancel_training_session(&mut self) {
        if self.training.session.take().is_some() {
            self.status_message = Some("Training session abandoned.".to_string());
        }
        self.training.result = None;
    }

    async fn resolve_training_session(
        &mut self,
        session: TrainingSession,
    ) -> color_eyre::Result<()> {
        let tier = training_tier(session.score, session.max_score);
        let (gained_xp, hours, success_credit) = match tier {
            TrainingTier::Poor => (session.plan.poor_xp, session.plan.poor_hours, false),
            TrainingTier::Solid => (session.plan.solid_xp, session.plan.solid_hours, true),
            TrainingTier::Great => (session.plan.great_xp, session.plan.great_hours, true),
        };

        let Some(ch) = self.active_character.as_mut() else {
            return Ok(());
        };
        let character_id = ch.id;
        let level_up_rank = match session.target {
            ProficiencyTarget::Major(skill) => {
                let before = ch.major_skill(skill);
                let after_xp = ch.major_skill_xp(skill) + gained_xp;
                ch.set_major_skill_xp(skill, after_xp);
                let after = ch.major_skill(skill);
                (after > before).then_some(after)
            }
            ProficiencyTarget::Minor(skill) => {
                let Some(entry) = ch
                    .proficiencies
                    .iter_mut()
                    .find(|entry| entry.kind == skill)
                else {
                    return Ok(());
                };
                let before = entry.level() as i32;
                entry.xp += gained_xp;
                let after = entry.level() as i32;
                (after > before).then_some(after)
            }
        };

        db::save_character_state(&self.pool, ch).await?;
        self.world_state.advance_time(hours);
        db::save_world_state(&self.pool, character_id, &self.world_state).await?;

        let mut achievement_lines = self
            .achievement_increment(character_id, "study_sessions", 1)
            .await?;
        achievement_lines.extend(
            self.achievement_increment(character_id, "study_hours", hours)
                .await?,
        );
        if success_credit {
            achievement_lines.extend(
                self.achievement_increment(character_id, "study_successes", 1)
                    .await?,
            );
        }
        achievement_lines.extend(self.refresh_meta_achievement_metrics(character_id).await?);

        self.training.result = Some(TrainingResult {
            target: session.target,
            tier,
            gained_xp,
            hours,
            hits: session.hits,
            misses: session.misses,
            score: session.score,
            max_score: session.max_score,
            level_up_rank,
            achievement_lines: achievement_lines.clone(),
        });
        self.status_message = Some(format!(
            "{} training finished: {} result, +{} XP, {}h spent.",
            session.target.name(),
            tier.label(),
            gained_xp,
            hours
        ));
        self.training.session = None;
        Ok(())
    }

    fn training_entry_count(&self) -> usize {
        self.active_character
            .as_ref()
            .map(|ch| MajorSkill::ALL.len() + ch.proficiencies.len())
            .unwrap_or(0)
    }

    fn selected_training_target(&self) -> Option<ProficiencyTarget> {
        let ch = self.active_character.as_ref()?;
        let idx = self
            .training
            .cursor
            .min(self.training_entry_count().saturating_sub(1));
        if idx < MajorSkill::ALL.len() {
            Some(ProficiencyTarget::Major(MajorSkill::ALL[idx]))
        } else {
            ch.proficiencies
                .get(idx - MajorSkill::ALL.len())
                .map(|entry| ProficiencyTarget::Minor(entry.kind))
        }
    }
}

fn random_training_sequence(len: usize) -> Vec<char> {
    let mut rng = rand::rng();
    (0..len)
        .map(|_| {
            let idx = rng.random_range(0..TRAINING_ALPHABET.len());
            TRAINING_ALPHABET[idx] as char
        })
        .collect()
}

fn training_tier(score: i32, max_score: i32) -> TrainingTier {
    if max_score <= 0 {
        return TrainingTier::Poor;
    }
    let ratio = score as f64 / max_score as f64;
    if ratio >= 0.72 {
        TrainingTier::Great
    } else if ratio >= 0.38 {
        TrainingTier::Solid
    } else {
        TrainingTier::Poor
    }
}

#[cfg(test)]
mod tests {
    use super::{random_training_sequence, training_tier};
    use crate::app::TrainingTier;

    #[test]
    fn training_tier_breakpoints_are_stable() {
        assert!(matches!(training_tier(0, 15), TrainingTier::Poor));
        assert!(matches!(training_tier(6, 15), TrainingTier::Solid));
        assert!(matches!(training_tier(11, 15), TrainingTier::Great));
    }

    #[test]
    fn generated_training_sequence_matches_requested_length() {
        let sequence = random_training_sequence(6);
        assert_eq!(sequence.len(), 6);
        assert!(sequence.iter().all(|ch| ch.is_ascii_lowercase()));
    }
}
