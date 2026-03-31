use super::{App, CharacterTab, DialogueChoice, Screen};
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
                    self.screen = Screen::MainMenu;
                } else {
                    self.creation.step = self.creation.step.prev();
                }
            }
            Screen::Dialogue => self.screen = self.dialogue_return,
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
        self.running = false;
    }
}
