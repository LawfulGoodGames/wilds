use super::App;
use crate::audio;
use crate::town_dialogue;
use crate::world::{NpcId, QuestId, quest_def};

impl App {
    pub async fn talk_to_selected_npc(&mut self) -> color_eyre::Result<()> {
        let npc = NpcId::ALL[self.npc_cursor];
        let scene = self.scene_for_npc(npc);
        self.maybe_play_audio(scene.id);

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

        let title = scene.title;
        let mut lines = scene
            .lines
            .iter()
            .map(|line| (*line).to_string())
            .collect::<Vec<_>>();
        let choices = scene
            .choices
            .iter()
            .map(|choice| super::super::DialogueChoice {
                label: choice.label.to_string(),
                response_lines: choice
                    .response_lines
                    .iter()
                    .map(|line| (*line).to_string())
                    .collect(),
                memory_flag: choice.memory_flag.map(str::to_string),
                status_message: choice.status_message.map(str::to_string),
                audio_id: choice.audio_id,
            })
            .collect::<Vec<_>>();

        if !quest_lines.is_empty() {
            lines.push(String::new());
            lines.append(&mut quest_lines);
        }
        if choices.is_empty() {
            self.open_dialog(title, lines, super::super::Screen::People);
        } else {
            self.open_choice_dialog(title, lines, choices, npc, super::super::Screen::People);
        }
        Ok(())
    }

    fn maybe_play_audio(&mut self, clip_id: &str) {
        if !self.settings.sound_effects {
            audio::stop(&mut self.dialogue_audio);
            return;
        }
        let _ = audio::play(clip_id, &mut self.dialogue_audio);
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
            self.save_world_state_for_active_character().await?;
        }

        if let Some(audio_id) = choice.audio_id {
            self.maybe_play_audio(audio_id);
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

    fn scene_for_npc(&self, npc: NpcId) -> &'static town_dialogue::DialogueSceneDef {
        match npc {
            NpcId::CaptainHedd
                if !self.world_state.has_flag("hedd_motive_duty")
                    && !self.world_state.has_flag("hedd_motive_coin")
                    && !self.world_state.has_flag("hedd_motive_truth") =>
            {
                &town_dialogue::HEDD_FIRST_MEETING_SCENE
            }
            NpcId::CaptainHedd
                if self
                    .world_state
                    .active_quest(QuestId::MissingOnTheWatch)
                    .is_some() =>
            {
                &town_dialogue::HEDD_MISSING_SCENE
            }
            NpcId::CaptainHedd
                if self
                    .world_state
                    .active_quest(QuestId::WordToTheCaptain)
                    .is_some() =>
            {
                &town_dialogue::HEDD_WORD_SCENE
            }
            NpcId::CaptainHedd => &town_dialogue::HEDD_DEFAULT_SCENE,
            NpcId::ScoutMira
                if self
                    .world_state
                    .active_quest(QuestId::ReportToMira)
                    .is_some() =>
            {
                &town_dialogue::MIRA_REPORT_SCENE
            }
            NpcId::ScoutMira => &town_dialogue::MIRA_DEFAULT_SCENE,
            NpcId::QuartermasterVale => &town_dialogue::VALE_DEFAULT_SCENE,
            NpcId::ArcanistSel
                if self
                    .world_state
                    .active_quest(QuestId::AshOnTheWax)
                    .is_some() =>
            {
                &town_dialogue::SEL_ASH_ON_WAX_SCENE
            }
            NpcId::ArcanistSel
                if self
                    .world_state
                    .active_quest(QuestId::CrownInCinders)
                    .is_some() =>
            {
                &town_dialogue::SEL_CROWN_IN_CINDERS_SCENE
            }
            NpcId::ArcanistSel if self.world_state.active_quest(QuestId::Gravewind).is_some() => {
                &town_dialogue::SEL_GRAVEWIND_SCENE
            }
            NpcId::ArcanistSel => &town_dialogue::SEL_DEFAULT_SCENE,
            NpcId::InnkeeperBrin => &town_dialogue::BRIN_DEFAULT_SCENE,
        }
    }
}
