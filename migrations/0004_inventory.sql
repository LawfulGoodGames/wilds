CREATE TABLE IF NOT EXISTS inventory (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    character_id INTEGER NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    item_type    TEXT    NOT NULL,
    quantity     INTEGER NOT NULL DEFAULT 1,
    UNIQUE(character_id, item_type)
);
