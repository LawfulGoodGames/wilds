use crate::character::{
    ability_unlock_level, class_progression, mana_growth, stamina_growth, CharacterCreation, KnownAbility,
    ProficiencyData, Race, ResourcePool, SavedCharacter, Stats, Class, MinorSkill,
};
use crate::inventory::{gear_package_items, Equipment, EquipSlot, InventoryItem};
use crate::world::{QuestProgress, WorldState};
use sqlx::{sqlite::{SqliteConnectOptions, SqlitePoolOptions, SqliteRow}, Row};
use std::{collections::HashMap, str::FromStr};

pub async fn init() -> color_eyre::Result<sqlx::SqlitePool> {
    let opts = SqliteConnectOptions::from_str("sqlite:wilds.db")?.create_if_missing(true);
    let pool = SqlitePoolOptions::new().connect_with(opts).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

pub async fn save_character(pool: &sqlx::SqlitePool, creation: &CharacterCreation) -> color_eyre::Result<i64> {
    let class = creation.selected_class();
    let stats = creation.final_stats();
    let max_hp = 72 + stats.modifier(crate::character::MajorSkill::Constitution) * 4;
    let max_mana = 16 + mana_growth(class) * 2 + stats.modifier(crate::character::MajorSkill::Wisdom).max(0) * 2;
    let max_stamina = 18 + stamina_growth(class) * 2 + stats.modifier(crate::character::MajorSkill::Constitution).max(0) * 2;

    let id = sqlx::query(
        "INSERT INTO characters
        (name, race, class, gear, level, xp, gold, unspent_stat_points,
         str_stat, dex_stat, con_stat, int_stat, wis_stat, cha_stat,
         hp, max_hp, mana, max_mana, stamina, max_stamina)
         VALUES (?1, ?2, ?3, ?4, 1, 0, 30, 0, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11, ?12, ?12, ?13, ?13)",
    )
    .bind(&creation.name)
    .bind(creation.selected_race().name())
    .bind(class.name())
    .bind(creation.selected_gear().name())
    .bind(stats.strength)
    .bind(stats.dexterity)
    .bind(stats.constitution)
    .bind(stats.intelligence)
    .bind(stats.wisdom)
    .bind(stats.charisma)
    .bind(max_hp)
    .bind(max_mana)
    .bind(max_stamina)
    .execute(pool)
    .await?
    .last_insert_rowid();

    for skill in MinorSkill::ALL {
        sqlx::query(
            "INSERT INTO character_proficiencies (character_id, skill_name, xp) VALUES (?1, ?2, 0)",
        )
        .bind(id)
        .bind(skill.name())
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

pub async fn load_character_by_id(pool: &sqlx::SqlitePool, id: i64) -> color_eyre::Result<SavedCharacter> {
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

pub async fn save_character_state(pool: &sqlx::SqlitePool, character: &SavedCharacter) -> color_eyre::Result<()> {
    sqlx::query(
        "UPDATE characters SET
            level = ?1, xp = ?2, gold = ?3, unspent_stat_points = ?4,
            str_stat = ?5, dex_stat = ?6, con_stat = ?7, int_stat = ?8, wis_stat = ?9, cha_stat = ?10,
            hp = ?11, max_hp = ?12, mana = ?13, max_mana = ?14, stamina = ?15, max_stamina = ?16
         WHERE id = ?17",
    )
    .bind(character.level)
    .bind(character.xp)
    .bind(character.gold)
    .bind(character.unspent_stat_points)
    .bind(character.stats.strength)
    .bind(character.stats.dexterity)
    .bind(character.stats.constitution)
    .bind(character.stats.intelligence)
    .bind(character.stats.wisdom)
    .bind(character.stats.charisma)
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
    Ok(())
}

pub async fn save_proficiency_xp(pool: &sqlx::SqlitePool, character_id: i64, skill: MinorSkill, xp: i32) -> color_eyre::Result<()> {
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

pub async fn load_inventory(pool: &sqlx::SqlitePool, character_id: i64) -> color_eyre::Result<Vec<InventoryItem>> {
    let rows = sqlx::query("SELECT item_type, quantity FROM inventory WHERE character_id = ?1 AND quantity > 0 ORDER BY item_type")
        .bind(character_id)
        .fetch_all(pool)
        .await?;
    Ok(rows
        .into_iter()
        .map(|row| InventoryItem {
            item_type: row.get("item_type"),
            quantity: row.get::<i64, _>("quantity") as i32,
        })
        .collect())
}

pub async fn add_item(pool: &sqlx::SqlitePool, character_id: i64, item_type: &str, quantity: i32) -> color_eyre::Result<()> {
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

pub async fn remove_item(pool: &sqlx::SqlitePool, character_id: i64, item_type: &str, quantity: i32) -> color_eyre::Result<()> {
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

pub async fn seed_starting_inventory(pool: &sqlx::SqlitePool, character_id: i64) -> color_eyre::Result<()> {
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

pub async fn seed_starting_gear(pool: &sqlx::SqlitePool, character_id: i64, gear_name: &str) -> color_eyre::Result<()> {
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

pub async fn load_equipment(pool: &sqlx::SqlitePool, character_id: i64) -> color_eyre::Result<Equipment> {
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

pub async fn equip_item(pool: &sqlx::SqlitePool, character_id: i64, slot: EquipSlot, item_type: &str) -> color_eyre::Result<()> {
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

pub async fn unequip_item(pool: &sqlx::SqlitePool, character_id: i64, slot: EquipSlot) -> color_eyre::Result<()> {
    sqlx::query("DELETE FROM equipment WHERE character_id = ?1 AND slot = ?2")
        .bind(character_id)
        .bind(slot.db_key())
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn load_world_state(pool: &sqlx::SqlitePool, character_id: i64) -> color_eyre::Result<WorldState> {
    let row = sqlx::query("SELECT current_area, unlocked_areas, completed_quests, world_flags FROM world_state WHERE character_id = ?1")
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
    state.world_flags = split_csv(row.get::<String, _>("world_flags"));
    state.active_quests = load_quests(pool, character_id).await?;
    Ok(state)
}

pub async fn save_world_state(pool: &sqlx::SqlitePool, character_id: i64, state: &WorldState) -> color_eyre::Result<()> {
    sqlx::query(
        "INSERT INTO world_state (character_id, current_area, unlocked_areas, completed_quests, world_flags)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(character_id) DO UPDATE SET
             current_area = excluded.current_area,
             unlocked_areas = excluded.unlocked_areas,
             completed_quests = excluded.completed_quests,
             world_flags = excluded.world_flags",
    )
    .bind(character_id)
    .bind(state.current_area.as_deref())
    .bind(join_csv(&state.unlocked_areas))
    .bind(join_csv(&state.completed_quests))
    .bind(join_csv(&state.world_flags))
    .execute(pool)
    .await?;

    for quest in &state.active_quests {
        sqlx::query(
            "INSERT INTO quests (character_id, quest_id, accepted, completed, objective_index, progress)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(character_id, quest_id) DO UPDATE SET
                accepted = excluded.accepted,
                completed = excluded.completed,
                objective_index = excluded.objective_index,
                progress = excluded.progress",
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
    Ok(())
}

pub async fn load_quests(pool: &sqlx::SqlitePool, character_id: i64) -> color_eyre::Result<Vec<QuestProgress>> {
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

async fn row_to_character(pool: &sqlx::SqlitePool, row: &SqliteRow) -> color_eyre::Result<SavedCharacter> {
    let id = row.get("id");
    let class = Class::from_name(row.get::<String, _>("class").as_str());
    let mut known_abilities = load_abilities(pool, id).await?;
    for (_, ability_id) in class_progression(class)
        .unlocks
        .into_iter()
        .filter(|(level, _)| *level <= row.get::<i64, _>("level") as i32)
    {
        if !known_abilities.iter().any(|ability| ability.ability_id == ability_id) {
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

    Ok(SavedCharacter {
        id,
        name: row.get("name"),
        race: Race::from_name(row.get::<String, _>("race").as_str()),
        class,
        gear: row.get("gear"),
        level: row.get::<i64, _>("level") as i32,
        xp: row.get::<i64, _>("xp") as i32,
        gold: row.get::<i64, _>("gold") as i32,
        unspent_stat_points: row.get::<i64, _>("unspent_stat_points") as i32,
        stats: Stats {
            strength: row.get::<i64, _>("str_stat") as i32,
            dexterity: row.get::<i64, _>("dex_stat") as i32,
            constitution: row.get::<i64, _>("con_stat") as i32,
            intelligence: row.get::<i64, _>("int_stat") as i32,
            wisdom: row.get::<i64, _>("wis_stat") as i32,
            charisma: row.get::<i64, _>("cha_stat") as i32,
        },
        resources: ResourcePool {
            hp: row.get::<i64, _>("hp") as i32,
            max_hp: row.get::<i64, _>("max_hp") as i32,
            mana: row.get::<i64, _>("mana") as i32,
            max_mana: row.get::<i64, _>("max_mana") as i32,
            stamina: row.get::<i64, _>("stamina") as i32,
            max_stamina: row.get::<i64, _>("max_stamina") as i32,
        },
        proficiencies: load_proficiencies(pool, id).await?,
        known_abilities,
    })
}

async fn load_proficiencies(pool: &sqlx::SqlitePool, character_id: i64) -> color_eyre::Result<Vec<ProficiencyData>> {
    let rows = sqlx::query("SELECT skill_name, xp FROM character_proficiencies WHERE character_id = ?1")
        .bind(character_id)
        .fetch_all(pool)
        .await?;
    let map: HashMap<String, i32> = rows
        .into_iter()
        .map(|row| (row.get::<String, _>("skill_name"), row.get::<i64, _>("xp") as i32))
        .collect();
    Ok(MinorSkill::ALL
        .iter()
        .map(|skill| ProficiencyData {
            kind: *skill,
            xp: *map.get(skill.name()).unwrap_or(&0),
        })
        .collect())
}

async fn load_abilities(pool: &sqlx::SqlitePool, character_id: i64) -> color_eyre::Result<Vec<KnownAbility>> {
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

