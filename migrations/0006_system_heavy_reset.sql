PRAGMA foreign_keys = OFF;

DROP TABLE IF EXISTS character_skills;
DROP TABLE IF EXISTS inventory;
DROP TABLE IF EXISTS equipment;
DROP TABLE IF EXISTS characters;

CREATE TABLE IF NOT EXISTS characters (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    name                TEXT    NOT NULL,
    race                TEXT    NOT NULL,
    class               TEXT    NOT NULL,
    gear                TEXT    NOT NULL,
    level               INTEGER NOT NULL DEFAULT 1,
    xp                  INTEGER NOT NULL DEFAULT 0,
    gold                INTEGER NOT NULL DEFAULT 30,
    unspent_stat_points INTEGER NOT NULL DEFAULT 0,
    str_stat            INTEGER NOT NULL,
    dex_stat            INTEGER NOT NULL,
    con_stat            INTEGER NOT NULL,
    int_stat            INTEGER NOT NULL,
    wis_stat            INTEGER NOT NULL,
    cha_stat            INTEGER NOT NULL,
    hp                  INTEGER NOT NULL,
    max_hp              INTEGER NOT NULL,
    mana                INTEGER NOT NULL,
    max_mana            INTEGER NOT NULL,
    stamina             INTEGER NOT NULL,
    max_stamina         INTEGER NOT NULL,
    created_at          INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE TABLE IF NOT EXISTS character_proficiencies (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    character_id INTEGER NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    skill_name   TEXT    NOT NULL,
    xp           INTEGER NOT NULL DEFAULT 0,
    UNIQUE(character_id, skill_name)
);

CREATE TABLE IF NOT EXISTS character_abilities (
    id                 INTEGER PRIMARY KEY AUTOINCREMENT,
    character_id       INTEGER NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    ability_id         TEXT    NOT NULL,
    rank               INTEGER NOT NULL DEFAULT 1,
    unlocked           INTEGER NOT NULL DEFAULT 1,
    cooldown_remaining INTEGER NOT NULL DEFAULT 0,
    UNIQUE(character_id, ability_id)
);

CREATE TABLE IF NOT EXISTS inventory (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    character_id INTEGER NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    item_type    TEXT    NOT NULL,
    quantity     INTEGER NOT NULL DEFAULT 1,
    UNIQUE(character_id, item_type)
);

CREATE TABLE IF NOT EXISTS equipment (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    character_id INTEGER NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    slot         TEXT    NOT NULL,
    item_type    TEXT    NOT NULL,
    UNIQUE(character_id, slot)
);

CREATE TABLE IF NOT EXISTS quests (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    character_id    INTEGER NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    quest_id        TEXT    NOT NULL,
    accepted        INTEGER NOT NULL DEFAULT 1,
    completed       INTEGER NOT NULL DEFAULT 0,
    objective_index INTEGER NOT NULL DEFAULT 0,
    progress        INTEGER NOT NULL DEFAULT 0,
    UNIQUE(character_id, quest_id)
);

CREATE TABLE IF NOT EXISTS world_state (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    character_id  INTEGER NOT NULL UNIQUE REFERENCES characters(id) ON DELETE CASCADE,
    current_area  TEXT,
    unlocked_areas TEXT NOT NULL DEFAULT '',
    completed_quests TEXT NOT NULL DEFAULT '',
    world_flags     TEXT NOT NULL DEFAULT ''
);

PRAGMA foreign_keys = ON;
