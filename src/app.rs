use crate::chess::{GameSnapshot, Move, Outcome};
use crate::event::{AppEvent, Event, EventHandler};
use crate::player::MoveSender;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;

pub mod screen;

pub use screen::{GameScreen, Screen};

/// Main application state - pure router only
///
/// App has two jobs:
/// 1. Render the active screen
/// 2. Route events (Crossterm or AppEvent) to the active screen
pub struct App {
    running: bool,
    events: EventHandler,
    move_tx: MoveSender,
    screen: Screen,
}

impl App {
    pub fn new(events: EventHandler, move_tx: MoveSender) -> Self {
        Self {
            running: true,
            events,
            move_tx,
            screen: Screen::Game(GameScreen::new()),
        }
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            terminal.draw(|frame| self.screen.render(frame))?;

            match self.events.next().await? {
                Event::Tick => {
                    self.screen.tick();
                }
                Event::Crossterm(event) => self.handle_crossterm(event),
                Event::App(event) => self.route(event),
            }
        }
        Ok(())
    }

    fn handle_crossterm(&mut self, event: crossterm::event::Event) {
        if let crossterm::event::Event::Key(key) = event {
            if key.kind != KeyEventKind::Press {
                return;
            }

            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    if !self.screen.wants_esc() {
                        self.events.send(AppEvent::Quit);
                    } else {
                        self.screen.handle_esc();
                    }
                }
                KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                    self.events.send(AppEvent::Quit);
                }
                KeyCode::Char('n') => {
                    self.events.send(AppEvent::NewGame);
                }
                _ => {
                    if let Some(mv) = self.screen.handle_key(key.code) {
                        let _ = self.move_tx.try_send(mv);
                    }
                }
            }
        }
    }

    fn route(&mut self, event: AppEvent) {
        match event {
            AppEvent::Quit => {
                self.running = false;
            }
            AppEvent::NewGame => {
                self.screen = Screen::Game(GameScreen::new());
            }
            AppEvent::StateUpdate(snapshot) => {
                self.screen.apply_snapshot(snapshot);
            }
            AppEvent::GameOver(outcome) => {
                self.screen.set_game_over(outcome);
            }
            AppEvent::IllegalMove => {
                self.screen.flash_illegal();
            }
        }
    }
}
