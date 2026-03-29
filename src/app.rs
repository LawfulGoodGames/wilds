use crate::event::{AppEvent, Event, EventHandler};
use crate::settings::{UserSettings, OPTIONS_COUNT};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::DefaultTerminal;
use sqlx::SqlitePool;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MenuItem {
    NewGame,
    LoadGame,
    Options,
    Quit,
}

impl MenuItem {
    pub const ALL: [MenuItem; 4] = [
        MenuItem::NewGame,
        MenuItem::LoadGame,
        MenuItem::Options,
        MenuItem::Quit,
    ];

    pub fn label(&self) -> &str {
        match self {
            MenuItem::NewGame => "New Game",
            MenuItem::LoadGame => "Load Game",
            MenuItem::Options => "Options",
            MenuItem::Quit => "Quit",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Screen {
    MainMenu,
    NewGame,
    LoadGame,
    Options,
}

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub selected: usize,
    pub screen: Screen,
    pub options_cursor: usize,
    pub settings: UserSettings,
    pool: SqlitePool,
    pub events: EventHandler,
}

impl App {
    pub fn new(pool: SqlitePool, settings: UserSettings) -> Self {
        Self {
            running: true,
            selected: 0,
            screen: Screen::MainMenu,
            options_cursor: 0,
            settings,
            pool,
            events: EventHandler::new(),
        }
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            match self.events.next().await? {
                Event::Tick => {}
                Event::Crossterm(event) => {
                    if let crossterm::event::Event::Key(key_event) = event {
                        if key_event.kind == crossterm::event::KeyEventKind::Press {
                            self.handle_key_events(key_event)?;
                        }
                    }
                }
                Event::App(app_event) => match app_event {
                    AppEvent::SelectUp => self.select_up(),
                    AppEvent::SelectDown => self.select_down(),
                    AppEvent::Confirm => self.confirm(),
                    AppEvent::Back => self.go_back().await?,
                    AppEvent::Left => self.change_option(-1),
                    AppEvent::Right => self.change_option(1),
                    AppEvent::Quit => self.quit(),
                },
            }
        }
        Ok(())
    }

    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match self.screen {
            Screen::MainMenu => match key_event.code {
                KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Quit),
                KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                    self.events.send(AppEvent::Quit)
                }
                KeyCode::Up | KeyCode::Char('k') => self.events.send(AppEvent::SelectUp),
                KeyCode::Down | KeyCode::Char('j') => self.events.send(AppEvent::SelectDown),
                KeyCode::Enter => self.events.send(AppEvent::Confirm),
                _ => {}
            },
            Screen::Options => match key_event.code {
                KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Back),
                KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                    self.events.send(AppEvent::Quit)
                }
                KeyCode::Up | KeyCode::Char('k') => self.events.send(AppEvent::SelectUp),
                KeyCode::Down | KeyCode::Char('j') => self.events.send(AppEvent::SelectDown),
                KeyCode::Left | KeyCode::Char('h') => self.events.send(AppEvent::Left),
                KeyCode::Right | KeyCode::Char('l') => self.events.send(AppEvent::Right),
                _ => {}
            },
            _ => match key_event.code {
                KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Back),
                KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                    self.events.send(AppEvent::Quit)
                }
                _ => {}
            },
        }
        Ok(())
    }

    fn select_up(&mut self) {
        let len = match self.screen {
            Screen::MainMenu => MenuItem::ALL.len(),
            Screen::Options => OPTIONS_COUNT,
            _ => return,
        };
        if self.options_cursor_for_screen() > 0 {
            *self.cursor_mut() -= 1;
        } else {
            *self.cursor_mut() = len - 1;
        }
    }

    fn select_down(&mut self) {
        let len = match self.screen {
            Screen::MainMenu => MenuItem::ALL.len(),
            Screen::Options => OPTIONS_COUNT,
            _ => return,
        };
        *self.cursor_mut() = (self.options_cursor_for_screen() + 1) % len;
    }

    fn cursor_mut(&mut self) -> &mut usize {
        match self.screen {
            Screen::Options => &mut self.options_cursor,
            _ => &mut self.selected,
        }
    }

    fn options_cursor_for_screen(&self) -> usize {
        match self.screen {
            Screen::Options => self.options_cursor,
            _ => self.selected,
        }
    }

    fn confirm(&mut self) {
        match MenuItem::ALL[self.selected] {
            MenuItem::NewGame => self.screen = Screen::NewGame,
            MenuItem::LoadGame => self.screen = Screen::LoadGame,
            MenuItem::Options => self.screen = Screen::Options,
            MenuItem::Quit => self.quit(),
        }
    }

    async fn go_back(&mut self) -> color_eyre::Result<()> {
        if self.screen == Screen::Options {
            self.settings.save(&self.pool).await?;
        }
        self.screen = Screen::MainMenu;
        Ok(())
    }

    fn change_option(&mut self, dir: i32) {
        if self.screen != Screen::Options {
            return;
        }
        match self.options_cursor {
            0 => self.settings.sound_effects = !self.settings.sound_effects,
            1 => {
                if dir > 0 {
                    self.settings.music_volume = self.settings.music_volume.saturating_add(10).min(100);
                } else {
                    self.settings.music_volume = self.settings.music_volume.saturating_sub(10);
                }
            }
            2 => {
                self.settings.font_size = if dir > 0 {
                    self.settings.font_size.cycle_next()
                } else {
                    self.settings.font_size.cycle_prev()
                };
            }
            3 => {
                self.settings.color_theme = if dir > 0 {
                    self.settings.color_theme.cycle_next()
                } else {
                    self.settings.color_theme.cycle_prev()
                };
            }
            4 => self.settings.show_hints = !self.settings.show_hints,
            5 => {
                self.settings.difficulty = if dir > 0 {
                    self.settings.difficulty.cycle_next()
                } else {
                    self.settings.difficulty.cycle_prev()
                };
            }
            _ => {}
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}
