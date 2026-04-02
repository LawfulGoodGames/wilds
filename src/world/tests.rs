use super::{QUESTS, QuestId, WorldState, quest_item_drop_is_relevant, quest_item_miss_text};

#[test]
fn every_defined_quest_is_listed_in_quest_id_all() {
    let listed_ids = QuestId::ALL.iter().map(|quest| quest.id()).collect::<Vec<_>>();
    let defined_ids = QUESTS.iter().map(|quest| quest.id.id()).collect::<Vec<_>>();

    assert_eq!(
        defined_ids, listed_ids,
        "QUESTS and QuestId::ALL must stay in sync so progression tests cover every quest"
    );
}

#[test]
fn story_quests_unlock_in_order() {
    let mut state = WorldState::default();
    assert!(state.can_accept_quest(QuestId::LanternsInTheRain));
    assert!(!state.can_accept_quest(QuestId::MissingOnTheWatch));
    state
        .completed_quests
        .push(QuestId::LanternsInTheRain.id().to_string());
    assert!(state.can_accept_quest(QuestId::MissingOnTheWatch));
}

#[test]
fn world_flags_only_store_unique_values() {
    let mut state = WorldState::default();
    state.set_flag("motive_duty");
    state.set_flag("motive_duty");
    assert!(state.has_flag("motive_duty"));
    assert_eq!(state.world_flags.len(), 1);
}

#[test]
fn quest_item_miss_text_is_context_aware() {
    let text = quest_item_miss_text(
        "roadside_ledger",
        "old_map",
        "Old Map",
        "Bandit Ambush",
        &["Bandit".to_string()],
        &["road".to_string()],
    );
    assert_eq!(
        text.as_deref(),
        Some("You search the fallen raiders, but the old map is nowhere on them.")
    );
    let no_text = quest_item_miss_text(
        "roadside_ledger",
        "old_map",
        "Old Map",
        "Beast Hunt",
        &["Beast".to_string()],
        &["woods".to_string()],
    );
    assert!(no_text.is_none());
}

#[test]
fn story_item_drops_stop_after_their_quest_beat_is_done() {
    let mut state = WorldState::default();
    assert!(quest_item_drop_is_relevant(&state, "old_map"));
    assert!(quest_item_drop_is_relevant(&state, "bandit_seal"));

    state
        .completed_quests
        .push(QuestId::RoadsideLedger.id().to_string());
    assert!(!quest_item_drop_is_relevant(&state, "old_map"));
    assert!(quest_item_drop_is_relevant(&state, "bandit_seal"));

    state
        .completed_quests
        .push(QuestId::AshOnTheWax.id().to_string());
    assert!(!quest_item_drop_is_relevant(&state, "bandit_seal"));
}
