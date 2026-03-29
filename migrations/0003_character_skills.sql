CREATE TABLE IF NOT EXISTS character_skills (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    character_id INTEGER NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    skill_name   TEXT    NOT NULL,
    xp           INTEGER NOT NULL DEFAULT 0,
    UNIQUE(character_id, skill_name)
);

CREATE INDEX IF NOT EXISTS idx_character_skills_char
    ON character_skills(character_id);
