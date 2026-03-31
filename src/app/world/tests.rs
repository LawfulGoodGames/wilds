use super::*;
use crate::character::CharacterCreation;
use crate::combat::PlayerAction;
use crate::db;
use crate::settings::UserSettings;
use crate::world::{AreaId, NpcId, ObjectiveKind, QuestId, QuestProgress, VendorId, quest_def};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

async fn migrated_pool() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("sqlite memory pool");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrations run");
    pool
}

async fn persisted_test_app() -> App {
    let pool = migrated_pool().await;
    let mut creation = CharacterCreation::default();
    creation.name = "Tester".to_string();
    let character_id = db::save_character(&pool, &creation)
        .await
        .expect("character saved");
    let mut app = App::new(pool.clone(), UserSettings::default());
    let mut character = db::load_character_by_id(&pool, character_id)
        .await
        .expect("character loaded");
    character.gold = 999;
    db::save_character_state(&pool, &character)
        .await
        .expect("character state saved");
    app.active_character = Some(character);
    app.world_state = db::load_world_state(&pool, character_id)
        .await
        .expect("world state loaded");
    app.equipment = db::load_equipment(&pool, character_id)
        .await
        .expect("equipment loaded");
    app.inventory.items = db::load_inventory(&pool, character_id)
        .await
        .expect("inventory loaded");
    app.achievements = db::load_achievement_state(&pool, character_id)
        .await
        .expect("achievement state loaded");
    app
}

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

#[tokio::test]
async fn entering_dialogue_marks_audio_active_and_back_stops_it() {
    let pool = SqlitePool::connect_lazy("sqlite::memory:").expect("lazy sqlite pool");
    let mut app = App::new(pool, UserSettings::default());
    app.npc_cursor = NpcId::ALL
        .iter()
        .position(|npc| *npc == NpcId::CaptainHedd)
        .expect("hedd exists");

    app.talk_to_selected_npc().await.expect("talk succeeds");

    assert_eq!(
        app.dialogue_audio.active_clip_id.as_deref(),
        Some("hedd.first_meeting")
    );
    assert!(app.dialogue_audio.is_playing());

    app.go_back().await.expect("back succeeds");

    assert_eq!(app.dialogue_audio.active_clip_id, None);
    assert!(!app.dialogue_audio.is_playing());
}

#[tokio::test]
async fn choosing_dialogue_response_replaces_scene_audio() {
    let pool = SqlitePool::connect_lazy("sqlite::memory:").expect("lazy sqlite pool");
    let mut app = App::new(pool, UserSettings::default());
    app.npc_cursor = NpcId::ALL
        .iter()
        .position(|npc| *npc == NpcId::CaptainHedd)
        .expect("hedd exists");

    app.talk_to_selected_npc().await.expect("talk succeeds");
    app.dialogue_cursor = 1;

    app.resolve_dialogue_choice()
        .await
        .expect("choice resolution succeeds");

    assert_eq!(
        app.dialogue_audio.active_clip_id.as_deref(),
        Some("hedd.response.coin")
    );
    assert!(app.dialogue_audio.is_playing());
}

#[tokio::test]
async fn quit_stops_dialogue_audio() {
    let pool = SqlitePool::connect_lazy("sqlite::memory:").expect("lazy sqlite pool");
    let mut app = App::new(pool, UserSettings::default());
    app.npc_cursor = NpcId::ALL
        .iter()
        .position(|npc| *npc == NpcId::CaptainHedd)
        .expect("hedd exists");

    app.talk_to_selected_npc().await.expect("talk succeeds");
    assert!(app.dialogue_audio.is_playing());

    app.quit();

    assert!(!app.running);
    assert_eq!(app.dialogue_audio.active_clip_id, None);
    assert!(!app.dialogue_audio.is_playing());
}

#[tokio::test]
async fn completed_story_quest_does_not_retrigger_from_stale_active_entry() {
    let pool = SqlitePool::connect_lazy("sqlite::memory:").expect("lazy sqlite pool");
    let mut app = App::new(pool, UserSettings::default());
    app.npc_cursor = NpcId::ALL
        .iter()
        .position(|npc| *npc == NpcId::CaptainHedd)
        .expect("hedd exists");

    app.world_state
        .completed_quests
        .push(QuestId::LanternsInTheRain.id().to_string());
    app.world_state.active_quests.push(QuestProgress {
        quest_id: QuestId::LanternsInTheRain.id().to_string(),
        accepted: true,
        completed: false,
        objective_index: 0,
        progress: 0,
    });

    app.talk_to_selected_npc().await.expect("talk succeeds");

    assert!(
        app.world_state
            .active_quest(QuestId::LanternsInTheRain)
            .is_none(),
        "stale active quest should be discarded"
    );
    assert_eq!(
        app.world_state
            .completed_quests
            .iter()
            .filter(|quest_id| *quest_id == QuestId::LanternsInTheRain.id())
            .count(),
        1,
        "completed quest should not be duplicated"
    );
    assert!(
        app.dialogue_lines
            .iter()
            .all(|line| !line.contains("Quest complete: Lanterns in the Rain.")),
        "completed quest should not pay out or complete again"
    );
}

#[tokio::test]
async fn exploration_departure_state_persists_after_reload() {
    let mut app = persisted_test_app().await;
    app.world_state.unlock_area(AreaId::SunkenRoad);
    app.world_state.active_quests.push(QuestProgress {
        quest_id: QuestId::TheBrokenPatrol.id().to_string(),
        accepted: true,
        completed: false,
        objective_index: 0,
        progress: 0,
    });

    app.begin_exploration(AreaId::SunkenRoad)
        .await
        .expect("exploration departure succeeds");

    let character_id = app.active_character.as_ref().expect("character").id;
    let reloaded = db::load_world_state(&app.pool, character_id)
        .await
        .expect("world reloaded");

    assert_eq!(
        reloaded.current_area.as_deref(),
        Some(AreaId::SunkenRoad.id())
    );
    assert!(reloaded.is_area_unlocked(AreaId::AshenBarrow));
    assert!(reloaded.has_completed(QuestId::TheBrokenPatrol));
    assert_eq!(reloaded.hour_of_day, 14);
}

#[tokio::test]
async fn exploration_departure_state_survives_successful_flee() {
    let mut app = persisted_test_app().await;
    app.world_state.unlock_area(AreaId::SunkenRoad);

    app.begin_exploration(AreaId::SunkenRoad)
        .await
        .expect("exploration departure succeeds");
    app.start_combat("beast_hunt").await.expect("combat starts");
    let combat = app.combat.as_mut().expect("combat active");
    combat.player.initiative = 99;
    combat.turn_index = combat
        .initiative
        .iter()
        .position(|turn| *turn == crate::combat::TurnRef::Player)
        .expect("player turn exists");

    app.handle_explicit_combat_action(PlayerAction::Flee)
        .await
        .expect("flee resolves");

    let character_id = app.active_character.as_ref().expect("character").id;
    let reloaded = db::load_world_state(&app.pool, character_id)
        .await
        .expect("world reloaded");

    assert_eq!(
        reloaded.current_area.as_deref(),
        Some(AreaId::SunkenRoad.id())
    );
    assert!(reloaded.is_area_unlocked(AreaId::AshenBarrow));
    assert_eq!(reloaded.hour_of_day, 14);
}

#[tokio::test]
async fn stale_visit_area_quest_is_pruned_without_retriggering_rewards() {
    let mut app = persisted_test_app().await;
    app.world_state.unlock_area(AreaId::AshenBarrow);
    app.world_state
        .completed_quests
        .push(QuestId::Gravewind.id().to_string());
    app.world_state.active_quests.push(QuestProgress {
        quest_id: QuestId::Gravewind.id().to_string(),
        accepted: true,
        completed: false,
        objective_index: 0,
        progress: 0,
    });

    app.begin_exploration(AreaId::AshenBarrow)
        .await
        .expect("exploration departure succeeds");

    assert!(app.world_state.active_quest(QuestId::Gravewind).is_none());
    assert_eq!(
        app.world_state
            .completed_quests
            .iter()
            .filter(|quest_id| *quest_id == QuestId::Gravewind.id())
            .count(),
        1
    );
    assert!(
        app.status_message
            .as_deref()
            .map(|message| !message.contains("Quest complete: Gravewind."))
            .unwrap_or(true)
    );
}

#[tokio::test]
async fn vendor_stock_depletes_persists_and_does_not_restock_on_sell() {
    let mut app = persisted_test_app().await;
    let vendor_cursor = VendorId::ALL
        .iter()
        .position(|vendor| *vendor == VendorId::Quartermaster)
        .expect("quartermaster exists");
    app.vendor_cursor = vendor_cursor;
    app.shop_buy_mode = true;
    app.shop_cursor = 0;

    app.shop_transaction().await.expect("first buy succeeds");

    let bought_gold = app.active_character.as_ref().expect("character").gold;
    assert_eq!(
        app.world_state
            .vendor_stock(VendorId::Quartermaster, "iron_sword"),
        0
    );

    app.shop_transaction()
        .await
        .expect("second buy attempt resolves");

    assert_eq!(
        app.active_character.as_ref().expect("character").gold,
        bought_gold
    );
    assert_eq!(
        app.status_message.as_deref(),
        Some("Iron Sword is sold out.")
    );

    let sword_index = app
        .inventory
        .items
        .iter()
        .position(|item| item.item_type == "iron_sword")
        .expect("bought sword in inventory");
    app.shop_buy_mode = false;
    app.shop_cursor = sword_index;
    app.shop_transaction().await.expect("sell resolves");

    let character_id = app.active_character.as_ref().expect("character").id;
    let reloaded = db::load_world_state(&app.pool, character_id)
        .await
        .expect("world reloaded");

    assert_eq!(
        reloaded.vendor_stock(VendorId::Quartermaster, "iron_sword"),
        0
    );
}

#[tokio::test]
async fn town_item_use_blocks_no_op_items_and_consumes_real_restoration() {
    let mut app = persisted_test_app().await;
    let character_id = app.active_character.as_ref().expect("character").id;
    db::add_item(&app.pool, character_id, "antidote", 1)
        .await
        .expect("antidote added");
    db::add_item(&app.pool, character_id, "health_potion", 1)
        .await
        .expect("health potion added");
    app.inventory.items = db::load_inventory(&app.pool, character_id)
        .await
        .expect("inventory loaded");
    let starting_potion_qty = app
        .inventory
        .items
        .iter()
        .find(|item| item.item_type == "health_potion")
        .map(|item| item.quantity)
        .expect("health potion quantity");

    app.inventory.cursor = app
        .inventory
        .items
        .iter()
        .position(|item| item.item_type == "antidote")
        .expect("antidote in inventory");
    app.use_inventory_item().await.expect("antidote resolves");
    assert_eq!(
        app.inventory.last_use_message.as_deref(),
        Some("Antidote can only be used in combat.")
    );
    assert_eq!(
        db::load_inventory(&app.pool, character_id)
            .await
            .expect("inventory reloaded")
            .iter()
            .find(|item| item.item_type == "antidote")
            .map(|item| item.quantity),
        Some(1)
    );

    app.inventory.items = db::load_inventory(&app.pool, character_id)
        .await
        .expect("inventory loaded");
    app.inventory.cursor = app
        .inventory
        .items
        .iter()
        .position(|item| item.item_type == "health_potion")
        .expect("health potion in inventory");
    app.use_inventory_item()
        .await
        .expect("full-health potion resolves");
    assert_eq!(
        app.inventory.last_use_message.as_deref(),
        Some("Health Potion would have no effect right now.")
    );
    assert_eq!(
        db::load_inventory(&app.pool, character_id)
            .await
            .expect("inventory reloaded")
            .iter()
            .find(|item| item.item_type == "health_potion")
            .map(|item| item.quantity),
        Some(starting_potion_qty)
    );

    let character = app.active_character.as_mut().expect("character");
    character.resources.hp = (character.resources.max_hp - 20).max(1);
    db::save_character_state(&app.pool, character)
        .await
        .expect("character state saved");
    app.inventory.items = db::load_inventory(&app.pool, character_id)
        .await
        .expect("inventory loaded");
    app.inventory.cursor = app
        .inventory
        .items
        .iter()
        .position(|item| item.item_type == "health_potion")
        .expect("health potion in inventory");

    app.use_inventory_item()
        .await
        .expect("damaged potion resolves");

    assert_eq!(
        db::load_inventory(&app.pool, character_id)
            .await
            .expect("inventory reloaded")
            .iter()
            .find(|item| item.item_type == "health_potion")
            .map(|item| item.quantity),
        Some(starting_potion_qty - 1)
    );
    assert!(
        app.active_character
            .as_ref()
            .expect("character")
            .resources
            .hp
            > app
                .active_character
                .as_ref()
                .expect("character")
                .resources
                .max_hp
                - 20
    );
}
