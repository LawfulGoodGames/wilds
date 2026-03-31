use super::*;
use crate::settings::UserSettings;
use crate::world::{NpcId, ObjectiveKind, QuestId, QuestProgress, quest_def};
use sqlx::SqlitePool;

#[tokio::test]
async fn main_story_chain_always_has_a_next_lead_until_the_end() {
    let pool = SqlitePool::connect_lazy("sqlite::memory:").expect("lazy sqlite pool");
    let mut app = App::new(pool, UserSettings::default());

    for (idx, expected) in QuestId::ALL.iter().copied().enumerate() {
        assert_eq!(
            app.world_state.current_story_lead(),
            Some(expected),
            "expected current lead to be {} at step {}",
            expected.id(),
            idx
        );

        assert!(
            app.world_state.accept_quest(expected),
            "expected {} to be acceptible",
            expected.id()
        );

        let objective_count = quest_def(expected.id())
            .expect("quest def exists")
            .objectives
            .len();
        let progress = app
            .world_state
            .active_quest_mut(expected)
            .expect("active quest exists");
        progress.objective_index = objective_count;

        let rewards = app
            .complete_ready_quests()
            .await
            .expect("quest completion succeeds");
        assert!(
            !rewards.is_empty(),
            "expected {} to complete with rewards",
            expected.id()
        );
        assert!(
            app.world_state.has_completed(expected),
            "expected {} to be marked complete",
            expected.id()
        );

        let next = app.world_state.current_story_lead();
        if idx + 1 < QuestId::ALL.len() {
            assert_eq!(
                next,
                Some(QuestId::ALL[idx + 1]),
                "expected next lead after {} to be {}",
                expected.id(),
                QuestId::ALL[idx + 1].id()
            );
        } else {
            assert_eq!(next, None, "expected no lead after final quest");
        }
    }
}

#[tokio::test]
async fn same_speaker_story_handoffs_auto_accept_the_next_quest() {
    let handoffs = QuestId::ALL
        .windows(2)
        .filter_map(|window| {
            let current = *window.first()?;
            let next = *window.get(1)?;
            let current_def = quest_def(current.id())?;
            let next_def = quest_def(next.id())?;
            matches!(
                current_def
                    .objectives
                    .last()
                    .map(|objective| &objective.kind),
                Some(ObjectiveKind::TalkToNpc { .. })
            )
            .then_some((
                current,
                next,
                current_def.giver == next_def.giver,
                next_def.giver,
            ))
        })
        .filter(|(_, _, same_giver, _)| *same_giver)
        .map(|(current, next, _, giver)| (current, next, giver))
        .collect::<Vec<_>>();

    assert!(
        !handoffs.is_empty(),
        "expected at least one same-speaker story handoff"
    );

    for (current, next, giver) in handoffs {
        let pool = SqlitePool::connect_lazy("sqlite::memory:").expect("lazy sqlite pool");
        let mut app = App::new(pool, UserSettings::default());
        app.npc_cursor = NpcId::ALL
            .iter()
            .position(|npc| *npc == giver)
            .expect("giver exists");
        for quest in QuestId::ALL {
            if quest == current {
                break;
            }
            app.world_state
                .completed_quests
                .push(quest.id().to_string());
        }
        app.world_state.active_quests.push(QuestProgress {
            quest_id: current.id().to_string(),
            accepted: true,
            completed: false,
            objective_index: 0,
            progress: 0,
        });

        app.talk_to_selected_npc().await.expect("talk succeeds");

        assert!(app.world_state.has_completed(current));
        assert!(
            app.world_state.active_quest(next).is_some(),
            "expected {} to hand off into {}",
            current.id(),
            next.id()
        );
        let expected = format!(
            "New quest: {}.",
            quest_def(next.id()).expect("next quest def").name
        );
        assert!(
            app.dialogue_lines
                .iter()
                .any(|line| line.contains(&expected)),
            "expected dialogue to announce {}",
            next.id()
        );
    }
}
