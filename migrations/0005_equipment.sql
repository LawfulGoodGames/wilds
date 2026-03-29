CREATE TABLE IF NOT EXISTS equipment (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    character_id INTEGER NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    slot         TEXT    NOT NULL,
    item_type    TEXT    NOT NULL,
    UNIQUE(character_id, slot)
);
