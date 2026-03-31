use super::App;
use crate::db;
use crate::world::{NpcId, QuestId, quest_def};

impl App {
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
            self.open_dialog(title, lines, super::super::Screen::People);
        } else {
            self.open_choice_dialog(title, lines, choices, npc, super::super::Screen::People);
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

    fn dialogue_for_npc(
        &self,
        npc: NpcId,
    ) -> (&'static str, Vec<String>, Vec<super::super::DialogueChoice>) {
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
                            super::super::DialogueChoice {
                                label: "Because someone has to hold the wall.".to_string(),
                                response_lines: vec![
                                    "Hedd gives a short nod. \"Duty keeps towns alive longer than luck does. Remember that.\"".to_string(),
                                ],
                                memory_flag: Some("hedd_motive_duty".to_string()),
                                status_message: Some("Captain Hedd remembers your sense of duty.".to_string()),
                            },
                            super::super::DialogueChoice {
                                label: "Because danger pays better than hunger.".to_string(),
                                response_lines: vec![
                                    "Hedd snorts. \"Honest enough. Stay alive and there might even be coin left when this is done.\"".to_string(),
                                ],
                                memory_flag: Some("hedd_motive_coin".to_string()),
                                status_message: Some("Captain Hedd clocks your eye for coin.".to_string()),
                            },
                            super::super::DialogueChoice {
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
                            super::super::DialogueChoice {
                                label: "Map every step. We only get one first read.".to_string(),
                                response_lines: vec![
                                    "\"Careful work wins long hunts,\" Mira says. \"Good. We'll read this trail clean.\"".to_string(),
                                ],
                                memory_flag: Some("mira_method_careful".to_string()),
                                status_message: Some("Mira notes that you favor caution.".to_string()),
                            },
                            super::super::DialogueChoice {
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
}
