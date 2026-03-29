CREATE TABLE IF NOT EXISTS user_settings (
    id             INTEGER PRIMARY KEY NOT NULL DEFAULT 1,
    sound_effects  INTEGER NOT NULL DEFAULT 1,
    music_volume   INTEGER NOT NULL DEFAULT 80,
    font_size      TEXT    NOT NULL DEFAULT 'Medium',
    color_theme    TEXT    NOT NULL DEFAULT 'Dark',
    show_hints     INTEGER NOT NULL DEFAULT 1,
    difficulty     TEXT    NOT NULL DEFAULT 'Normal',
    CHECK (id = 1)
);

-- Seed the single settings row if it doesn't exist yet
INSERT OR IGNORE INTO user_settings (id) VALUES (1);
