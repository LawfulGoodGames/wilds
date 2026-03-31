mod dialogue;
mod quests;
mod town;

use super::App;
use crate::inventory::find_def;
use crate::world::{ObjectiveKind, quest_def};

impl App {
    pub(super) fn objective_lead_text(&self, objective: &ObjectiveKind) -> String {
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

    pub(super) fn current_story_lead_line(&self) -> Option<String> {
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
}

#[cfg(test)]
mod tests;
