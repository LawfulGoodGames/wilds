use crate::achievements::AchievementState;
use crate::character::{
    CharacterCreation, Class, KnownAbility, MajorProficiencyData, MajorSkill, MinorSkill,
    ProficiencyData, Race, ResourcePool, SavedCharacter, Stats, ability_unlock_level,
    class_progression, mana_growth, stamina_growth,
};
use crate::inventory::{EquipSlot, Equipment, InventoryItem, find_def, gear_package_items};
use crate::world::{QuestProgress, VendorStockEntry, WorldState};
use sqlx::{
    Row,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions, SqliteRow},
};
use std::{collections::HashMap, str::FromStr};

pub async fn init() -> color_eyre::Result<sqlx::SqlitePool> {
    let opts = SqliteConnectOptions::from_str("sqlite:wilds.db")?.create_if_missing(true);
    let pool = SqlitePoolOptions::new().connect_with(opts).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

pub async fn save_character(
    pool: &sqlx::SqlitePool,
    creation: &CharacterCreation,
) -> color_eyre::Result<i64> {
    let class = creation.selected_class();
    let stats = creation.final_stats();
    let max_hp = 88 + stats.modifier(crate::character::MajorSkill::Constitution) * 5;
    let max_mana = 20
        + mana_growth(class) * 2
        + stats
            .modifier(crate::character::MajorSkill::Intelligence)
            .max(0)
            * 3;
    let max_stamina = 24
        + stamina_growth(class) * 2
        + stats
            .modifier(crate::character::MajorSkill::Strength)
            .max(0)
            * 3;

    let id = sqlx::query(
        "INSERT INTO characters
        (name, race, class, gear, level, xp, gold, unspent_stat_points,
         hp, max_hp, mana, max_mana, stamina, max_stamina)
         VALUES (?1, ?2, ?3, ?4, 1, 0, 30, 0, ?5, ?5, ?6, ?6, ?7, ?7)",
    )
    .bind(&creation.name)
    .bind(creation.selected_race().name())
    .bind(class.name())
    .bind(creation.selected_gear().name())
    .bind(max_hp)
    .bind(max_mana)
    .bind(max_stamina)
    .execute(pool)
    .await?
    .last_insert_rowid();

    for skill in MinorSkill::ALL {
        sqlx::query(
            "INSERT INTO character_proficiencies (character_id, skill_name, xp) VALUES (?1, ?2, ?3)",
        )
        .bind(id)
        .bind(skill.name())
        .bind(creation.starting_proficiency_xp(skill))
        .execute(pool)
        .await?;
    }
    for skill in MajorSkill::ALL {
        sqlx::query(
            "INSERT INTO character_proficiencies (character_id, skill_name, xp) VALUES (?1, ?2, ?3)",
        )
        .bind(id)
        .bind(skill.full_name())
        .bind(creation.starting_major_proficiency_xp(skill))
        .execute(pool)
        .await?;
    }

    for (_, ability_id) in class_progression(class)
        .unlocks
        .into_iter()
        .filter(|(level, _)| *level <= 1)
    {
        sqlx::query(
            "INSERT INTO character_abilities (character_id, ability_id, rank, unlocked, cooldown_remaining)
             VALUES (?1, ?2, 1, 1, 0)",
        )
        .bind(id)
        .bind(ability_id)
        .execute(pool)
        .await?;
    }

    seed_starting_inventory(pool, id).await?;
    seed_starting_gear(pool, id, creation.selected_gear().name()).await?;
    save_world_state(pool, id, &WorldState::default()).await?;
    Ok(id)
}

pub async fn load_character_by_id(
    pool: &sqlx::SqlitePool,
    id: i64,
) -> color_eyre::Result<SavedCharacter> {
    let row = sqlx::query("SELECT * FROM characters WHERE id = ?1")
        .bind(id)
        .fetch_one(pool)
        .await?;
    row_to_character(pool, &row).await
}

pub async fn load_characters(pool: &sqlx::SqlitePool) -> color_eyre::Result<Vec<SavedCharacter>> {
    let rows = sqlx::query("SELECT * FROM characters ORDER BY created_at DESC")
        .fetch_all(pool)
        .await?;
    let mut out = vec![];
    for row in rows {
        out.push(row_to_character(pool, &row).await?);
    }
    Ok(out)
}

pub async fn rename_character(
    pool: &sqlx::SqlitePool,
    character_id: i64,
    new_name: &str,
) -> color_eyre::Result<()> {
    sqlx::query("UPDATE characters SET name = ?1 WHERE id = ?2")
        .bind(new_name.trim())
        .bind(character_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_character(
    pool: &sqlx::SqlitePool,
    character_id: i64,
) -> color_eyre::Result<()> {
    let mut tx = pool.begin().await?;
    for table in [
        "character_proficiencies",
        "character_abilities",
        "inventory",
        "equipment",
        "quests",
        "world_state",
        "vendor_stock",
        "achievement_metrics",
    ] {
        sqlx::query(&format!("DELETE FROM {table} WHERE character_id = ?1"))
            .bind(character_id)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("DELETE FROM characters WHERE id = ?1")
        .bind(character_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn save_character_state(
    pool: &sqlx::SqlitePool,
    character: &SavedCharacter,
) -> color_eyre::Result<()> {
    sqlx::query(
        "UPDATE characters SET
            level = ?1, xp = ?2, gold = ?3, unspent_stat_points = ?4,
            hp = ?5, max_hp = ?6, mana = ?7, max_mana = ?8, stamina = ?9, max_stamina = ?10
         WHERE id = ?11",
    )
    .bind(character.level)
    .bind(character.xp)
    .bind(character.gold)
    .bind(character.unspent_stat_points)
    .bind(character.resources.hp)
    .bind(character.resources.max_hp)
    .bind(character.resources.mana)
    .bind(character.resources.max_mana)
    .bind(character.resources.stamina)
    .bind(character.resources.max_stamina)
    .bind(character.id)
    .execute(pool)
    .await?;

    for ability in &character.known_abilities {
        sqlx::query(
            "INSERT INTO character_abilities (character_id, ability_id, rank, unlocked, cooldown_remaining)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(character_id, ability_id)
             DO UPDATE SET rank = excluded.rank, unlocked = excluded.unlocked, cooldown_remaining = excluded.cooldown_remaining",
        )
        .bind(character.id)
        .bind(&ability.ability_id)
        .bind(ability.rank)
        .bind(i32::from(ability.unlocked))
        .bind(ability.cooldown_remaining)
        .execute(pool)
        .await?;
    }
    for skill in &character.major_proficiencies {
        sqlx::query(
            "INSERT INTO character_proficiencies (character_id, skill_name, xp) VALUES (?1, ?2, ?3)
             ON CONFLICT(character_id, skill_name) DO UPDATE SET xp = excluded.xp",
        )
        .bind(character.id)
        .bind(skill.kind.full_name())
        .bind(skill.xp)
        .execute(pool)
        .await?;
    }
    for skill in &character.proficiencies {
        sqlx::query(
            "INSERT INTO character_proficiencies (character_id, skill_name, xp) VALUES (?1, ?2, ?3)
             ON CONFLICT(character_id, skill_name) DO UPDATE SET xp = excluded.xp",
        )
        .bind(character.id)
        .bind(skill.kind.name())
        .bind(skill.xp)
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn save_proficiency_xp(
    pool: &sqlx::SqlitePool,
    character_id: i64,
    skill: MinorSkill,
    xp: i32,
) -> color_eyre::Result<()> {
    sqlx::query(
        "INSERT INTO character_proficiencies (character_id, skill_name, xp) VALUES (?1, ?2, ?3)
         ON CONFLICT(character_id, skill_name) DO UPDATE SET xp = excluded.xp",
    )
    .bind(character_id)
    .bind(skill.name())
    .bind(xp)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn load_achievement_state(
    pool: &sqlx::SqlitePool,
    character_id: i64,
) -> color_eyre::Result<AchievementState> {
    let rows =
        sqlx::query("SELECT metric_key, value FROM achievement_metrics WHERE character_id = ?1")
            .bind(character_id)
            .fetch_all(pool)
            .await?;
    let mut state = AchievementState::default();
    for row in rows {
        state.metrics.insert(
            row.get::<String, _>("metric_key"),
            row.get::<i64, _>("value") as i32,
        );
    }
    state.recompute_unlocked();
    Ok(state)
}

pub async fn save_achievement_metric(
    pool: &sqlx::SqlitePool,
    character_id: i64,
    metric_key: &str,
    value: i32,
) -> color_eyre::Result<()> {
    sqlx::query(
        "INSERT INTO achievement_metrics (character_id, metric_key, value) VALUES (?1, ?2, ?3)
         ON CONFLICT(character_id, metric_key) DO UPDATE SET value = excluded.value",
    )
    .bind(character_id)
    .bind(metric_key)
    .bind(value)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn load_inventory(
    pool: &sqlx::SqlitePool,
    character_id: i64,
) -> color_eyre::Result<Vec<InventoryItem>> {
    let rows = sqlx::query("SELECT item_type, quantity FROM inventory WHERE character_id = ?1 AND quantity > 0 ORDER BY item_type")
        .bind(character_id)
        .fetch_all(pool)
        .await?;
    let mut items = rows
        .into_iter()
        .map(|row| InventoryItem {
            item_type: row.get("item_type"),
            quantity: row.get::<i64, _>("quantity") as i32,
        })
        .collect::<Vec<_>>();
    items.sort_by_key(|item| {
        find_def(&item.item_type)
            .map(|def| (def.kind.sort_order(), def.name.to_string()))
            .unwrap_or((u8::MAX, item.item_type.clone()))
    });
    Ok(items)
}

pub async fn add_item(
    pool: &sqlx::SqlitePool,
    character_id: i64,
    item_type: &str,
    quantity: i32,
) -> color_eyre::Result<()> {
    sqlx::query(
        "INSERT INTO inventory (character_id, item_type, quantity) VALUES (?1, ?2, ?3)
         ON CONFLICT(character_id, item_type) DO UPDATE SET quantity = quantity + excluded.quantity",
    )
    .bind(character_id)
    .bind(item_type)
    .bind(quantity)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn remove_item(
    pool: &sqlx::SqlitePool,
    character_id: i64,
    item_type: &str,
    quantity: i32,
) -> color_eyre::Result<()> {
    sqlx::query(
        "UPDATE inventory SET quantity = MAX(quantity - ?1, 0) WHERE character_id = ?2 AND item_type = ?3",
    )
    .bind(quantity)
    .bind(character_id)
    .bind(item_type)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn seed_starting_inventory(
    pool: &sqlx::SqlitePool,
    character_id: i64,
) -> color_eyre::Result<()> {
    for (item, qty) in [
        ("health_potion", 3),
        ("bandage", 2),
        ("ration", 2),
        ("mana_tonic", 1),
        ("stamina_draught", 1),
    ] {
        add_item(pool, character_id, item, qty).await?;
    }
    Ok(())
}

pub async fn seed_starting_gear(
    pool: &sqlx::SqlitePool,
    character_id: i64,
    gear_name: &str,
) -> color_eyre::Result<()> {
    for (slot, item_type) in gear_package_items(gear_name) {
        sqlx::query(
            "INSERT INTO equipment (character_id, slot, item_type) VALUES (?1, ?2, ?3)
             ON CONFLICT(character_id, slot) DO UPDATE SET item_type = excluded.item_type",
        )
        .bind(character_id)
        .bind(*slot)
        .bind(*item_type)
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn load_equipment(
    pool: &sqlx::SqlitePool,
    character_id: i64,
) -> color_eyre::Result<Equipment> {
    let rows = sqlx::query("SELECT slot, item_type FROM equipment WHERE character_id = ?1")
        .bind(character_id)
        .fetch_all(pool)
        .await?;
    let mut out = Equipment::default();
    for row in rows {
        let slot: String = row.get("slot");
        let item_type: String = row.get("item_type");
        if let Some(slot_enum) = EquipSlot::ALL.iter().find(|it| it.db_key() == slot) {
            out.set_slot(*slot_enum, Some(item_type));
        }
    }
    Ok(out)
}

pub async fn equip_item(
    pool: &sqlx::SqlitePool,
    character_id: i64,
    slot: EquipSlot,
    item_type: &str,
) -> color_eyre::Result<()> {
    sqlx::query(
        "INSERT INTO equipment (character_id, slot, item_type) VALUES (?1, ?2, ?3)
         ON CONFLICT(character_id, slot) DO UPDATE SET item_type = excluded.item_type",
    )
    .bind(character_id)
    .bind(slot.db_key())
    .bind(item_type)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn unequip_item(
    pool: &sqlx::SqlitePool,
    character_id: i64,
    slot: EquipSlot,
) -> color_eyre::Result<()> {
    sqlx::query("DELETE FROM equipment WHERE character_id = ?1 AND slot = ?2")
        .bind(character_id)
        .bind(slot.db_key())
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn load_world_state(
    pool: &sqlx::SqlitePool,
    character_id: i64,
) -> color_eyre::Result<WorldState> {
    let row = sqlx::query(
        "SELECT current_area, unlocked_areas, completed_quests, world_flags, campaign_day, hour_of_day
         FROM world_state WHERE character_id = ?1",
    )
        .bind(character_id)
        .fetch_optional(pool)
        .await?;

    let Some(row) = row else {
        return Ok(WorldState::default());
    };

    let mut state = WorldState::default();
    state.current_area = row.get::<Option<String>, _>("current_area");
    state.unlocked_areas = split_csv(row.get::<String, _>("unlocked_areas"));
    if state.unlocked_areas.is_empty() {
        state.unlocked_areas.push("whispering_woods".to_string());
    }
    state.completed_quests = split_csv(row.get::<String, _>("completed_quests"));
    state.vendor_stock = load_vendor_stock(pool, character_id).await?;
    state.world_flags = split_csv(row.get::<String, _>("world_flags"));
    state.campaign_day = row.get::<i64, _>("campaign_day") as i32;
    state.hour_of_day = row.get::<i64, _>("hour_of_day") as i32;
    state.active_quests = load_quests(pool, character_id).await?;
    Ok(state)
}

pub async fn save_world_state(
    pool: &sqlx::SqlitePool,
    character_id: i64,
    state: &WorldState,
) -> color_eyre::Result<()> {
    sqlx::query(
        "INSERT INTO world_state (character_id, current_area, unlocked_areas, completed_quests, world_flags, campaign_day, hour_of_day)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(character_id) DO UPDATE SET
             current_area = excluded.current_area,
             unlocked_areas = excluded.unlocked_areas,
             completed_quests = excluded.completed_quests,
             world_flags = excluded.world_flags,
             campaign_day = excluded.campaign_day,
             hour_of_day = excluded.hour_of_day",
    )
    .bind(character_id)
    .bind(state.current_area.as_deref())
    .bind(join_csv(&state.unlocked_areas))
    .bind(join_csv(&state.completed_quests))
    .bind(join_csv(&state.world_flags))
    .bind(state.campaign_day)
    .bind(state.hour_of_day)
    .execute(pool)
    .await?;

    sqlx::query("DELETE FROM quests WHERE character_id = ?1")
        .bind(character_id)
        .execute(pool)
        .await?;

    sqlx::query("DELETE FROM vendor_stock WHERE character_id = ?1")
        .bind(character_id)
        .execute(pool)
        .await?;

    for quest in &state.active_quests {
        sqlx::query(
            "INSERT INTO quests (character_id, quest_id, accepted, completed, objective_index, progress)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )
        .bind(character_id)
        .bind(&quest.quest_id)
        .bind(i32::from(quest.accepted))
        .bind(i32::from(quest.completed))
        .bind(quest.objective_index as i32)
        .bind(quest.progress)
        .execute(pool)
        .await?;
    }
    for stock in &state.vendor_stock {
        sqlx::query(
            "INSERT INTO vendor_stock (character_id, vendor_id, item_type, quantity)
             VALUES (?1, ?2, ?3, ?4)",
        )
        .bind(character_id)
        .bind(&stock.vendor_id)
        .bind(&stock.item_type)
        .bind(stock.quantity)
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn load_quests(
    pool: &sqlx::SqlitePool,
    character_id: i64,
) -> color_eyre::Result<Vec<QuestProgress>> {
    let rows = sqlx::query(
        "SELECT quest_id, accepted, completed, objective_index, progress FROM quests WHERE character_id = ?1",
    )
    .bind(character_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| QuestProgress {
            quest_id: row.get("quest_id"),
            accepted: row.get::<i64, _>("accepted") != 0,
            completed: row.get::<i64, _>("completed") != 0,
            objective_index: row.get::<i64, _>("objective_index") as usize,
            progress: row.get::<i64, _>("progress") as i32,
        })
        .collect())
}

async fn load_vendor_stock(
    pool: &sqlx::SqlitePool,
    character_id: i64,
) -> color_eyre::Result<Vec<VendorStockEntry>> {
    let rows = sqlx::query(
        "SELECT vendor_id, item_type, quantity FROM vendor_stock WHERE character_id = ?1",
    )
    .bind(character_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| VendorStockEntry {
            vendor_id: row.get("vendor_id"),
            item_type: row.get("item_type"),
            quantity: row.get::<i64, _>("quantity") as i32,
        })
        .collect())
}

async fn row_to_character(
    pool: &sqlx::SqlitePool,
    row: &SqliteRow,
) -> color_eyre::Result<SavedCharacter> {
    let id = row.get("id");
    let proficiency_map = load_proficiency_map(pool, id).await?;
    let class = Class::from_name(row.get::<String, _>("class").as_str());
    let mut known_abilities = load_abilities(pool, id).await?;
    for (_, ability_id) in class_progression(class)
        .unlocks
        .into_iter()
        .filter(|(level, _)| *level <= row.get::<i64, _>("level") as i32)
    {
        if !known_abilities
            .iter()
            .any(|ability| ability.ability_id == ability_id)
        {
            known_abilities.push(KnownAbility {
                ability_id: ability_id.to_string(),
                rank: 1,
                unlocked: true,
                cooldown_remaining: 0,
            });
        }
    }
    known_abilities.sort_by(|a, b| {
        ability_unlock_level(class, &a.ability_id)
            .cmp(&ability_unlock_level(class, &b.ability_id))
            .then(a.ability_id.cmp(&b.ability_id))
    });

    let major_proficiencies = load_major_proficiencies(&proficiency_map)?;
    let mut character = SavedCharacter {
        id,
        name: row.get("name"),
        race: Race::from_name(row.get::<String, _>("race").as_str()),
        class,
        gear: row.get("gear"),
        level: row.get::<i64, _>("level") as i32,
        xp: row.get::<i64, _>("xp") as i32,
        gold: row.get::<i64, _>("gold") as i32,
        unspent_stat_points: row.get::<i64, _>("unspent_stat_points") as i32,
        stats: Stats::default(),
        major_proficiencies,
        resources: ResourcePool {
            hp: row.get::<i64, _>("hp") as i32,
            max_hp: row.get::<i64, _>("max_hp") as i32,
            mana: row.get::<i64, _>("mana") as i32,
            max_mana: row.get::<i64, _>("max_mana") as i32,
            stamina: row.get::<i64, _>("stamina") as i32,
            max_stamina: row.get::<i64, _>("max_stamina") as i32,
        },
        proficiencies: load_proficiencies_from_map(&proficiency_map),
        known_abilities,
    };
    character.sync_stats_from_major_proficiencies();
    Ok(character)
}

async fn load_proficiency_map(
    pool: &sqlx::SqlitePool,
    character_id: i64,
) -> color_eyre::Result<HashMap<String, i32>> {
    let rows =
        sqlx::query("SELECT skill_name, xp FROM character_proficiencies WHERE character_id = ?1")
            .bind(character_id)
            .fetch_all(pool)
            .await?;
    Ok(rows
        .into_iter()
        .map(|row| {
            (
                row.get::<String, _>("skill_name"),
                row.get::<i64, _>("xp") as i32,
            )
        })
        .collect())
}

fn load_proficiencies_from_map(map: &HashMap<String, i32>) -> Vec<ProficiencyData> {
    MinorSkill::ALL
        .iter()
        .map(|skill| ProficiencyData {
            kind: *skill,
            xp: *map.get(skill.name()).unwrap_or(&0),
        })
        .collect()
}

fn load_major_proficiencies(
    map: &HashMap<String, i32>,
) -> color_eyre::Result<Vec<MajorProficiencyData>> {
    MajorSkill::ALL
        .iter()
        .map(|skill| {
            let xp = map.get(skill.full_name()).copied().ok_or_else(|| {
                color_eyre::eyre::eyre!("Missing major proficiency row: {}", skill.full_name())
            })?;
            Ok(MajorProficiencyData { kind: *skill, xp })
        })
        .collect()
}

async fn load_abilities(
    pool: &sqlx::SqlitePool,
    character_id: i64,
) -> color_eyre::Result<Vec<KnownAbility>> {
    let rows = sqlx::query(
        "SELECT ability_id, rank, unlocked, cooldown_remaining FROM character_abilities WHERE character_id = ?1",
    )
    .bind(character_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| KnownAbility {
            ability_id: row.get("ability_id"),
            rank: row.get::<i64, _>("rank") as i32,
            unlocked: row.get::<i64, _>("unlocked") != 0,
            cooldown_remaining: row.get::<i64, _>("cooldown_remaining") as i32,
        })
        .collect())
}

fn split_csv(value: String) -> Vec<String> {
    value
        .split(',')
        .filter(|part| !part.trim().is_empty())
        .map(|part| part.trim().to_string())
        .collect()
}

fn join_csv(values: &[String]) -> String {
    values.join(",")
}
