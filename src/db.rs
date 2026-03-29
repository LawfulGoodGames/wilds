use crate::character::{CharacterCreation, SavedCharacter};
use sqlx::{Row, sqlite::{SqliteConnectOptions, SqlitePoolOptions}};
use std::str::FromStr;

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
    let s = creation.final_stats();
    let result = sqlx::query(
        "INSERT INTO characters
            (name, race, class, str_stat, dex_stat, con_stat, int_stat, wis_stat, cha_stat, gear)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
    )
    .bind(&creation.name)
    .bind(creation.selected_race().name())
    .bind(creation.selected_class().name())
    .bind(s.strength    as i64)
    .bind(s.dexterity   as i64)
    .bind(s.constitution as i64)
    .bind(s.intelligence as i64)
    .bind(s.wisdom      as i64)
    .bind(s.charisma    as i64)
    .bind(creation.selected_gear().name())
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
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

    Ok(row_to_character(&row))
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

    Ok(rows.iter().map(row_to_character).collect())
}

fn row_to_character(row: &sqlx::sqlite::SqliteRow) -> SavedCharacter {
    SavedCharacter {
        id:       row.get("id"),
        name:     row.get("name"),
        race:     row.get("race"),
        class:    row.get("class"),
        gear:     row.get("gear"),
        level:    row.get::<i64, _>("level")   as i32,
        xp:       row.get::<i64, _>("xp")      as i32,
        hp:       row.get::<i64, _>("hp")       as i32,
        max_hp:   row.get::<i64, _>("max_hp")   as i32,
        gold:     row.get::<i64, _>("gold")     as i32,
        str_stat: row.get::<i64, _>("str_stat") as i32,
        dex_stat: row.get::<i64, _>("dex_stat") as i32,
        con_stat: row.get::<i64, _>("con_stat") as i32,
        int_stat: row.get::<i64, _>("int_stat") as i32,
        wis_stat: row.get::<i64, _>("wis_stat") as i32,
        cha_stat: row.get::<i64, _>("cha_stat") as i32,
    }
}
