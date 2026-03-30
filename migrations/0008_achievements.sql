CREATE TABLE IF NOT EXISTS achievement_metrics (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    character_id INTEGER NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    metric_key   TEXT    NOT NULL,
    value        INTEGER NOT NULL DEFAULT 0,
    UNIQUE(character_id, metric_key)
);
