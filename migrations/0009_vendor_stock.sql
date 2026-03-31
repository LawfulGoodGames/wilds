CREATE TABLE IF NOT EXISTS vendor_stock (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    character_id INTEGER NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    vendor_id    TEXT    NOT NULL,
    item_type    TEXT    NOT NULL,
    quantity     INTEGER NOT NULL DEFAULT 0,
    UNIQUE(character_id, vendor_id, item_type)
);
