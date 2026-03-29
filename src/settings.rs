use sqlx::{Row, SqlitePool};

pub const OPTIONS_COUNT: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontSize {
    Small,
    Medium,
    Large,
}

impl FontSize {
    pub fn label(self) -> &'static str {
        match self {
            Self::Small => "Small",
            Self::Medium => "Medium",
            Self::Large => "Large",
        }
    }
    pub fn from_str(s: &str) -> Self {
        match s {
            "Small" => Self::Small,
            "Large" => Self::Large,
            _ => Self::Medium,
        }
    }
    pub fn cycle_next(self) -> Self {
        match self {
            Self::Small => Self::Medium,
            Self::Medium => Self::Large,
            Self::Large => Self::Small,
        }
    }
    pub fn cycle_prev(self) -> Self {
        match self {
            Self::Small => Self::Large,
            Self::Medium => Self::Small,
            Self::Large => Self::Medium,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorTheme {
    Dark,
    Light,
    HighContrast,
}

impl ColorTheme {
    pub fn label(self) -> &'static str {
        match self {
            Self::Dark => "Dark",
            Self::Light => "Light",
            Self::HighContrast => "High Contrast",
        }
    }
    pub fn from_str(s: &str) -> Self {
        match s {
            "Light" => Self::Light,
            "High Contrast" => Self::HighContrast,
            _ => Self::Dark,
        }
    }
    pub fn cycle_next(self) -> Self {
        match self {
            Self::Dark => Self::Light,
            Self::Light => Self::HighContrast,
            Self::HighContrast => Self::Dark,
        }
    }
    pub fn cycle_prev(self) -> Self {
        match self {
            Self::Dark => Self::HighContrast,
            Self::Light => Self::Dark,
            Self::HighContrast => Self::Light,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
}

impl Difficulty {
    pub fn label(self) -> &'static str {
        match self {
            Self::Easy => "Easy",
            Self::Normal => "Normal",
            Self::Hard => "Hard",
        }
    }
    pub fn from_str(s: &str) -> Self {
        match s {
            "Easy" => Self::Easy,
            "Hard" => Self::Hard,
            _ => Self::Normal,
        }
    }
    pub fn cycle_next(self) -> Self {
        match self {
            Self::Easy => Self::Normal,
            Self::Normal => Self::Hard,
            Self::Hard => Self::Easy,
        }
    }
    pub fn cycle_prev(self) -> Self {
        match self {
            Self::Easy => Self::Hard,
            Self::Normal => Self::Easy,
            Self::Hard => Self::Normal,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserSettings {
    pub sound_effects: bool,
    pub music_volume: u8,
    pub font_size: FontSize,
    pub color_theme: ColorTheme,
    pub show_hints: bool,
    pub difficulty: Difficulty,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            sound_effects: true,
            music_volume: 80,
            font_size: FontSize::Medium,
            color_theme: ColorTheme::Dark,
            show_hints: true,
            difficulty: Difficulty::Normal,
        }
    }
}

impl UserSettings {
    pub async fn load(pool: &SqlitePool) -> color_eyre::Result<Self> {
        let row = sqlx::query(
            "SELECT sound_effects, music_volume, font_size, color_theme, show_hints, difficulty \
             FROM user_settings WHERE id = 1",
        )
        .fetch_optional(pool)
        .await?;

        Ok(match row {
            Some(r) => Self {
                sound_effects: r.get::<i64, _>("sound_effects") != 0,
                music_volume: r.get::<i64, _>("music_volume") as u8,
                font_size: FontSize::from_str(r.get("font_size")),
                color_theme: ColorTheme::from_str(r.get("color_theme")),
                show_hints: r.get::<i64, _>("show_hints") != 0,
                difficulty: Difficulty::from_str(r.get("difficulty")),
            },
            None => Self::default(),
        })
    }

    pub async fn save(&self, pool: &SqlitePool) -> color_eyre::Result<()> {
        let sound = self.sound_effects as i64;
        let volume = self.music_volume as i64;
        let hints = self.show_hints as i64;
        sqlx::query(
            "UPDATE user_settings \
             SET sound_effects = ?1, music_volume = ?2, font_size = ?3, \
                 color_theme = ?4, show_hints = ?5, difficulty = ?6 \
             WHERE id = 1",
        )
        .bind(sound)
        .bind(volume)
        .bind(self.font_size.label())
        .bind(self.color_theme.label())
        .bind(hints)
        .bind(self.difficulty.label())
        .execute(pool)
        .await?;
        Ok(())
    }
}
