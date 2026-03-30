use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

mod character_sheet;
mod combat_view;
mod creation;
mod menu;
mod panels;
mod shared;

use crate::app::{App, Screen};
use character_sheet::render_character_sheet;
use combat_view::render_combat;
use creation::render_character_creation;
use menu::{render_load_game, render_main_menu, render_options};
use panels::{
    render_achievements, render_dialogue, render_equipment, render_explore, render_inventory,
    render_quests, render_shop, render_town,
};
use shared::{
    GOLD, dim_style, hint_bar, normal_style, progress_bar, render_centered, render_item_detail,
    render_placeholder, render_status_bar, selected_style,
};

const TITLE: &str = r"
 __        _____ _     ____  ____
 \ \      / /_ _| |   |  _ \/ ___|
  \ \ /\ / / | || |   | | | \___ \
   \ V  V /  | || |___| |_| |___) |
    \_/\_/  |___|_____|____/|____/
";

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.screen {
            Screen::MainMenu => render_main_menu(self, area, buf),
            Screen::CharacterCreation => render_character_creation(self, area, buf),
            Screen::LoadGame => render_load_game(self, area, buf),
            Screen::Options => render_options(self, area, buf),
            Screen::Town => render_town(self, area, buf),
            Screen::Explore => render_explore(self, area, buf),
            Screen::CharacterSheet => render_character_sheet(self, area, buf),
            Screen::Inventory => render_inventory(self, area, buf),
            Screen::Equipment => render_equipment(self, area, buf),
            Screen::Quests => render_quests(self, area, buf),
            Screen::Achievements => render_achievements(self, area, buf),
            Screen::Shop => render_shop(self, area, buf),
            Screen::Dialogue => render_dialogue(self, area, buf),
            Screen::Combat => render_combat(self, area, buf),
        }
    }
}
