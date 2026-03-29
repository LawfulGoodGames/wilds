use crate::event::{AppEvent, Event, EventHandler};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::DefaultTerminal;

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
    pub events: EventHandler,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            selected: 0,
            screen: Screen::MainMenu,
            events: EventHandler::new(),
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
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
                    AppEvent::Back => self.screen = Screen::MainMenu,
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
        if self.selected > 0 {
            self.selected -= 1;
        } else {
            self.selected = MenuItem::ALL.len() - 1;
        }
    }

    fn select_down(&mut self) {
        self.selected = (self.selected + 1) % MenuItem::ALL.len();
    }

    fn confirm(&mut self) {
        match MenuItem::ALL[self.selected] {
            MenuItem::NewGame => self.screen = Screen::NewGame,
            MenuItem::LoadGame => self.screen = Screen::LoadGame,
            MenuItem::Options => self.screen = Screen::Options,
            MenuItem::Quit => self.quit(),
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}
