use super::{App, CharacterTab, DialogueChoice, LoadGameMode, Screen};
use crate::audio;
use crate::character::CharacterCreation;
use crate::db;

impl App {
    pub async fn go_back(&mut self) -> color_eyre::Result<()> {
        match self.screen {
            Screen::Options => {
                self.settings.save(&self.pool).await?;
                self.screen = Screen::MainMenu;
            }
            Screen::CharacterCreation => {
                if self.creation.step == crate::character::CreationStep::Name {
                    self.screen = self.creation_return_screen;
                } else {
                    self.creation.step = self.creation.step.prev();
                }
            }
            Screen::LoadGame if self.load_mode != LoadGameMode::Browse => {
                self.load_mode = LoadGameMode::Browse;
                self.load_name_input.clear();
            }
            Screen::Dialogue => {
                audio::stop(&mut self.dialogue_audio);
                self.screen = self.dialogue_return;
            }
            Screen::Explore
            | Screen::People
            | Screen::CharacterSheet
            | Screen::Inventory
            | Screen::Equipment
            | Screen::Quests
            | Screen::Achievements
            | Screen::Shop
            | Screen::Training
            | Screen::Combat => {
                self.cancel_training_session();
                self.screen = Screen::Town;
                self.combat = None;
            }
            Screen::Town | Screen::LoadGame | Screen::MainMenu => self.screen = Screen::MainMenu,
        }
        Ok(())
    }

    pub fn open_character_creation(&mut self, return_screen: Screen) {
        self.creation = CharacterCreation::default();
        self.creation_return_screen = return_screen;
        self.screen = Screen::CharacterCreation;
    }

    pub fn open_explore(&mut self) {
        self.explore_cursor = 0;
        self.screen = Screen::Explore;
        self.status_message = None;
    }

    pub fn open_people(&mut self) {
        self.npc_cursor = 0;
        self.screen = Screen::People;
        self.status_message = None;
    }

    pub fn open_character_sheet(&mut self) {
        self.character_tab = CharacterTab::Proficiencies;
        self.character_cursor = 0;
        self.detail_scroll = 0;
        self.screen = Screen::CharacterSheet;
    }

    pub fn open_quests(&mut self) {
        self.quest_cursor = 0;
        self.screen = Screen::Quests;
    }

    pub fn open_achievements(&mut self) {
        self.achievement_cursor = 0;
        self.screen = Screen::Achievements;
    }

    pub async fn open_inventory(&mut self) -> color_eyre::Result<()> {
        let Some(ch) = &self.active_character else {
            return Ok(());
        };
        self.inventory.items = db::load_inventory(&self.pool, ch.id).await?;
        self.inventory.cursor = 0;
        self.inventory.last_use_message = None;
        self.detail_scroll = 0;
        self.screen = Screen::Inventory;
        Ok(())
    }

    pub async fn open_equipment(&mut self) -> color_eyre::Result<()> {
        let Some(ch) = &self.active_character else {
            return Ok(());
        };
        self.equipment = db::load_equipment(&self.pool, ch.id).await?;
        self.equipment_cursor = 0;
        self.status_message = None;
        self.detail_scroll = 0;
        self.screen = Screen::Equipment;
        Ok(())
    }

    pub async fn open_shop(&mut self) -> color_eyre::Result<()> {
        let Some(ch) = &self.active_character else {
            return Ok(());
        };
        self.inventory.items = db::load_inventory(&self.pool, ch.id).await?;
        self.vendor_cursor = 0;
        self.shop_cursor = 0;
        self.shop_buy_mode = true;
        self.detail_scroll = 0;
        self.screen = Screen::Shop;
        Ok(())
    }

    pub fn open_training(&mut self) {
        self.training = super::TrainingState::default();
        self.screen = Screen::Training;
        self.status_message = None;
    }

    pub fn open_dialog(&mut self, title: &str, lines: Vec<String>, return_screen: Screen) {
        self.dialogue_title = title.to_string();
        self.dialogue_lines = lines;
        self.dialogue_choices.clear();
        self.dialogue_cursor = 0;
        self.dialogue_npc = None;
        self.dialogue_return = return_screen;
        self.screen = Screen::Dialogue;
    }

    pub fn open_choice_dialog(
        &mut self,
        title: &str,
        lines: Vec<String>,
        choices: Vec<DialogueChoice>,
        npc: crate::world::NpcId,
        return_screen: Screen,
    ) {
        self.dialogue_title = title.to_string();
        self.dialogue_lines = lines;
        self.dialogue_choices = choices;
        self.dialogue_cursor = 0;
        self.dialogue_npc = Some(npc);
        self.dialogue_return = return_screen;
        self.screen = Screen::Dialogue;
    }

    pub fn quit(&mut self) {
        audio::stop(&mut self.dialogue_audio);
        self.running = false;
    }

    pub fn start_load_game_rename(&mut self) {
        let Some(character) = self.saved_characters.get(self.load_cursor) else {
            self.status_message = Some("Create a character before trying to rename one.".to_string());
            return;
        };
        self.load_mode = LoadGameMode::Renaming;
        self.load_name_input = character.name.clone();
        self.status_message = None;
    }

    pub fn confirm_load_game_delete(&mut self) {
        if self.saved_characters.is_empty() {
            self.status_message = Some("No saved adventurers to delete.".to_string());
            return;
        }
        self.load_mode = LoadGameMode::ConfirmDelete;
        self.load_name_input.clear();
        self.status_message = None;
    }

    pub async fn submit_load_game_rename(&mut self) -> color_eyre::Result<()> {
        let Some(character) = self.saved_characters.get(self.load_cursor) else {
            return Ok(());
        };
        let new_name = self.load_name_input.trim().to_string();
        if new_name.is_empty() {
            self.status_message = Some("A character name cannot be empty.".to_string());
            return Ok(());
        }
        let character_id = character.id;
        let old_name = character.name.clone();
        db::rename_character(&self.pool, character_id, &new_name).await?;
        self.saved_characters = db::load_characters(&self.pool).await?;
        if let Some(updated_idx) = self
            .saved_characters
            .iter()
            .position(|candidate| candidate.id == character_id)
        {
            self.load_cursor = updated_idx;
        }
        self.load_mode = LoadGameMode::Browse;
        self.load_name_input.clear();
        self.status_message = Some(format!("{old_name} is now known as {new_name}."));
        Ok(())
    }

    pub async fn delete_selected_character(&mut self) -> color_eyre::Result<()> {
        let Some(character) = self.saved_characters.get(self.load_cursor) else {
            return Ok(());
        };
        let character_id = character.id;
        let character_name = character.name.clone();
        db::delete_character(&self.pool, character_id).await?;
        self.saved_characters = db::load_characters(&self.pool).await?;
        if self.saved_characters.is_empty() {
            self.load_cursor = 0;
        } else {
            self.load_cursor = self.load_cursor.min(self.saved_characters.len() - 1);
        }
        self.load_mode = LoadGameMode::Browse;
        self.load_name_input.clear();
        self.status_message = Some(format!("{character_name} has been deleted."));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::UserSettings;
    use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

    async fn migrated_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("sqlite memory pool");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrations run");
        pool
    }

    async fn app_with_saved_characters(names: &[&str]) -> App {
        let pool = migrated_pool().await;
        for name in names {
            let mut creation = CharacterCreation::default();
            creation.name = (*name).to_string();
            db::save_character(&pool, &creation)
                .await
                .expect("character saved");
        }
        let mut app = App::new(pool.clone(), UserSettings::default());
        app.saved_characters = db::load_characters(&pool)
            .await
            .expect("characters loaded");
        app.screen = Screen::LoadGame;
        app
    }

    #[tokio::test]
    async fn renaming_character_from_load_game_updates_save_list() {
        let mut app = app_with_saved_characters(&["Before"]).await;
        app.start_load_game_rename();
        app.load_name_input = "After".to_string();

        app.submit_load_game_rename().await.expect("rename succeeds");

        assert_eq!(app.load_mode, LoadGameMode::Browse);
        assert_eq!(app.saved_characters.len(), 1);
        assert_eq!(app.saved_characters[0].name, "After");
        assert_eq!(
            app.status_message.as_deref(),
            Some("Before is now known as After.")
        );
    }

    #[tokio::test]
    async fn deleting_character_from_load_game_removes_only_selected_save() {
        let mut app = app_with_saved_characters(&["First", "Second"]).await;
        let second_idx = app
            .saved_characters
            .iter()
            .position(|character| character.name == "Second")
            .expect("second character exists");
        app.load_cursor = second_idx;
        app.confirm_load_game_delete();

        app.delete_selected_character()
            .await
            .expect("delete succeeds");

        assert_eq!(app.load_mode, LoadGameMode::Browse);
        assert_eq!(app.saved_characters.len(), 1);
        assert_eq!(app.saved_characters[0].name, "First");
        assert_eq!(
            app.status_message.as_deref(),
            Some("Second has been deleted.")
        );
    }
}
