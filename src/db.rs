use crate::character::{
    CharacterCreation, MajorSkill, MajorSkillData, MinorSkill, MinorSkillData, SavedCharacter,
};
use crate::inventory::{gear_package_items, Equipment, EquipSlot, InventoryItem};
use sqlx::{Row, sqlite::{SqliteConnectOptions, SqlitePoolOptions, SqliteRow}};
use std::{collections::HashMap, str::FromStr};

pub async fn init() -> color_eyre::Result<sqlx::SqlitePool> {
    let opts = SqliteConnectOptions::from_str("sqlite:wilds.db")?.create_if_missing(true);
    let pool = SqlitePoolOptions::new().connect_with(opts).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

// ── Character save/load ───────────────────────────────────────────────────────

pub async fn save_character(
    pool: &sqlx::SqlitePool,
    creation: &CharacterCreation,
) -> color_eyre::Result<i64> {
    let s = creation.final_stats();
    let result = sqlx::query(
        "INSERT INTO characters
            (name, race, class, str_stat, dex_stat, con_stat, int_stat, wis_stat, cha_stat, gear)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
    )
    .bind(&creation.name)
    .bind(creation.selected_race().name())
    .bind(creation.selected_class().name())
    .bind(s.strength     as i64)
    .bind(s.dexterity    as i64)
    .bind(s.constitution as i64)
    .bind(s.intelligence as i64)
    .bind(s.wisdom       as i64)
    .bind(s.charisma     as i64)
    .bind(creation.selected_gear().name())
    .execute(pool)
    .await?;

    let id = result.last_insert_rowid();

    // Seed all 12 minor skills at XP = 0
    for skill in MinorSkill::ALL {
        sqlx::query(
            "INSERT OR IGNORE INTO character_skills (character_id, skill_name, xp)
             VALUES (?1, ?2, 0)",
        )
        .bind(id)
        .bind(skill.name())
        .execute(pool)
        .await?;
    }

    seed_starting_inventory(pool, id).await?;
    seed_starting_gear(pool, id, creation.selected_gear().name()).await?;

    Ok(id)
}

pub async fn load_character_by_id(
    pool: &sqlx::SqlitePool,
    id: i64,
) -> color_eyre::Result<SavedCharacter> {
    let row = sqlx::query(
        "SELECT id, name, race, class, gear, level, xp, hp, max_hp, gold,
                str_stat, dex_stat, con_stat, int_stat, wis_stat, cha_stat
         FROM characters WHERE id = ?1",
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    let mut ch = row_to_character(&row);
    ch.minor_skills = load_minor_skills(pool, id).await?;
    Ok(ch)
}

pub async fn load_characters(
    pool: &sqlx::SqlitePool,
) -> color_eyre::Result<Vec<SavedCharacter>> {
    let rows = sqlx::query(
        "SELECT id, name, race, class, gear, level, xp, hp, max_hp, gold,
                str_stat, dex_stat, con_stat, int_stat, wis_stat, cha_stat
         FROM characters ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;

    let mut chars = Vec::with_capacity(rows.len());
    for row in &rows {
        let mut ch = row_to_character(row);
        ch.minor_skills = load_minor_skills(pool, ch.id).await?;
        chars.push(ch);
    }
    Ok(chars)
}

// ── Skill XP update ───────────────────────────────────────────────────────────

pub async fn update_minor_skill_xp(
    pool: &sqlx::SqlitePool,
    character_id: i64,
    skill: MinorSkill,
    new_xp: i32,
) -> color_eyre::Result<()> {
    sqlx::query(
        "UPDATE character_skills SET xp = ?1
         WHERE character_id = ?2 AND skill_name = ?3",
    )
    .bind(new_xp as i64)
    .bind(character_id)
    .bind(skill.name())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_major_skill(
    pool: &sqlx::SqlitePool,
    character_id: i64,
    skill: MajorSkill,
    new_points: i32,
) -> color_eyre::Result<()> {
    let col = skill.db_column();
    sqlx::query(&format!(
        "UPDATE characters SET {col} = ?1 WHERE id = ?2"
    ))
    .bind(new_points as i64)
    .bind(character_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_character_progress(
    pool: &sqlx::SqlitePool,
    character_id: i64,
    hp: i32,
    xp: i32,
    gold: i32,
) -> color_eyre::Result<()> {
    sqlx::query(
        "UPDATE characters
         SET hp = ?1, xp = ?2, gold = ?3
         WHERE id = ?4",
    )
    .bind(hp as i64)
    .bind(xp as i64)
    .bind(gold as i64)
    .bind(character_id)
    .execute(pool)
    .await?;
    Ok(())
}

// ── Inventory ─────────────────────────────────────────────────────────────────

pub async fn seed_starting_inventory(
    pool: &sqlx::SqlitePool,
    character_id: i64,
) -> color_eyre::Result<()> {
    let starters = [
        ("health_potion", 3i64),
        ("bandage", 2),
        ("ration", 1),
    ];
    for (item_type, qty) in starters {
        sqlx::query(
            "INSERT OR IGNORE INTO inventory (character_id, item_type, quantity)
             VALUES (?1, ?2, ?3)",
        )
        .bind(character_id)
        .bind(item_type)
        .bind(qty)
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn load_inventory(
    pool: &sqlx::SqlitePool,
    character_id: i64,
) -> color_eyre::Result<Vec<InventoryItem>> {
    let rows = sqlx::query(
        "SELECT item_type, quantity FROM inventory
         WHERE character_id = ?1 AND quantity > 0
         ORDER BY item_type",
    )
    .bind(character_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| InventoryItem {
            item_type: r.get("item_type"),
            quantity:  r.get::<i64, _>("quantity") as i32,
        })
        .collect())
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
    .bind(quantity as i64)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn decrement_item(
    pool: &sqlx::SqlitePool,
    character_id: i64,
    item_type: &str,
) -> color_eyre::Result<()> {
    sqlx::query(
        "UPDATE inventory SET quantity = quantity - 1
         WHERE character_id = ?1 AND item_type = ?2 AND quantity > 0",
    )
    .bind(character_id)
    .bind(item_type)
    .execute(pool)
    .await?;
    Ok(())
}

// ── Equipment ─────────────────────────────────────────────────────────────────

pub async fn seed_starting_gear(
    pool: &sqlx::SqlitePool,
    character_id: i64,
    gear_name: &str,
) -> color_eyre::Result<()> {
    for (slot, item_type) in gear_package_items(gear_name) {
        sqlx::query(
            "INSERT OR IGNORE INTO equipment (character_id, slot, item_type)
             VALUES (?1, ?2, ?3)",
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
    let rows = sqlx::query(
        "SELECT slot, item_type FROM equipment WHERE character_id = ?1",
    )
    .bind(character_id)
    .fetch_all(pool)
    .await?;

    let mut eq = Equipment::default();
    for row in rows {
        let slot: String    = row.get("slot");
        let item: String    = row.get("item_type");
        match slot.as_str() {
            "weapon" => eq.weapon = Some(item),
            "shield" => eq.shield = Some(item),
            "head"   => eq.head   = Some(item),
            "neck"   => eq.neck   = Some(item),
            "chest"  => eq.chest  = Some(item),
            "cape"   => eq.cape   = Some(item),
            "hands"  => eq.hands  = Some(item),
            "ring"   => eq.ring   = Some(item),
            "legs"   => eq.legs   = Some(item),
            "feet"   => eq.feet   = Some(item),
            _ => {}
        }
    }
    Ok(eq)
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
    sqlx::query(
        "DELETE FROM equipment WHERE character_id = ?1 AND slot = ?2",
    )
    .bind(character_id)
    .bind(slot.db_key())
    .execute(pool)
    .await?;
    Ok(())
}

// ── Private helpers ───────────────────────────────────────────────────────────

async fn load_minor_skills(
    pool: &sqlx::SqlitePool,
    character_id: i64,
) -> color_eyre::Result<Vec<MinorSkillData>> {
    let rows = sqlx::query(
        "SELECT skill_name, xp FROM character_skills WHERE character_id = ?1",
    )
    .bind(character_id)
    .fetch_all(pool)
    .await?;

    let mut map: HashMap<String, i32> = rows
        .iter()
        .map(|r| (r.get::<String, _>("skill_name"), r.get::<i64, _>("xp") as i32))
        .collect();

    Ok(MinorSkill::ALL
        .iter()
        .map(|&kind| MinorSkillData {
            kind,
            xp: map.remove(kind.name()).unwrap_or(0),
        })
        .collect())
}

fn row_to_character(row: &SqliteRow) -> SavedCharacter {
    SavedCharacter {
        id:     row.get("id"),
        name:   row.get("name"),
        race:   row.get("race"),
        class:  row.get("class"),
        gear:   row.get("gear"),
        level:  row.get::<i64, _>("level")  as i32,
        xp:     row.get::<i64, _>("xp")     as i32,
        hp:     row.get::<i64, _>("hp")      as i32,
        max_hp: row.get::<i64, _>("max_hp")  as i32,
        gold:   row.get::<i64, _>("gold")    as i32,
        major_skills: vec![
            MajorSkillData { kind: MajorSkill::Strength,     points: row.get::<i64,_>("str_stat") as i32 },
            MajorSkillData { kind: MajorSkill::Dexterity,    points: row.get::<i64,_>("dex_stat") as i32 },
            MajorSkillData { kind: MajorSkill::Constitution, points: row.get::<i64,_>("con_stat") as i32 },
            MajorSkillData { kind: MajorSkill::Intelligence, points: row.get::<i64,_>("int_stat") as i32 },
            MajorSkillData { kind: MajorSkill::Wisdom,       points: row.get::<i64,_>("wis_stat") as i32 },
            MajorSkillData { kind: MajorSkill::Charisma,     points: row.get::<i64,_>("cha_stat") as i32 },
        ],
        minor_skills: Vec::new(), // populated by callers via load_minor_skills
    }
}
