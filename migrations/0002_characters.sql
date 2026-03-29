CREATE TABLE IF NOT EXISTS characters (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    name          TEXT    NOT NULL,
    race          TEXT    NOT NULL,
    class         TEXT    NOT NULL,
    -- base stats (after allocation + race bonuses)
    str_stat      INTEGER NOT NULL DEFAULT 5,
    dex_stat      INTEGER NOT NULL DEFAULT 5,
    con_stat      INTEGER NOT NULL DEFAULT 5,
    int_stat      INTEGER NOT NULL DEFAULT 5,
    wis_stat      INTEGER NOT NULL DEFAULT 5,
    cha_stat      INTEGER NOT NULL DEFAULT 5,
    -- starting gear package
    gear          TEXT    NOT NULL,
    -- initial game state
    level         INTEGER NOT NULL DEFAULT 1,
    xp            INTEGER NOT NULL DEFAULT 0,
    hp            INTEGER NOT NULL DEFAULT 100,
    max_hp        INTEGER NOT NULL DEFAULT 100,
    gold          INTEGER NOT NULL DEFAULT 10,
    created_at    INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);
