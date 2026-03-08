use crate::chess::{Game, Move};
use crate::event::{AppEvent, Event};
use crate::player::{HotseatPlayer, MoveReceiver, Player};
use tokio::sync::mpsc;

/// Game Coordinator - owns game state and runs as a background task
///
/// The Coordinator is the sole authority on game state. It receives moves
/// from the TUI via a channel, applies them, and broadcasts state updates
/// back by injecting events into the TUI's event loop.
struct Coordinator {
    game: Game,
    white: Player,
    black: Player,
    move_rx: MoveReceiver,
    app_tx: mpsc::UnboundedSender<Event>,
}

impl Coordinator {
    fn new(
        move_rx: MoveReceiver,
        app_tx: mpsc::UnboundedSender<Event>,
    ) -> Self {
        Self {
            game: Game::new(),
            white: Player::Hotseat(HotseatPlayer::new(shakmaty::Color::White)),
            black: Player::Hotseat(HotseatPlayer::new(shakmaty::Color::Black)),
            move_rx,
            app_tx,
        }
    }

    async fn run(mut self) {
        self.broadcast_state();

        loop {
            match self.move_rx.recv().await {
                None => break,
                Some(mv) => {
                    match self.game.make_move(mv) {
                        Ok(()) => {
                            self.broadcast_state();
                            if self.game.outcome().is_some() {
                                self.inject(AppEvent::GameOver(self.game.outcome().unwrap()));
                                break;
                            }
                        }
                        Err(_) => {
                            self.inject(AppEvent::IllegalMove);
                        }
                    }
                }
            }
        }
    }

    fn broadcast_state(&self) {
        self.inject(AppEvent::StateUpdate(self.game.snapshot()));
    }

    fn inject(&self, event: AppEvent) {
        let _ = self.app_tx.send(Event::App(event));
    }
}

/// Spawn the game coordinator as a background task
///
/// This should be called once from main.rs during initialization.
pub async fn run(move_rx: MoveReceiver, app_tx: mpsc::UnboundedSender<Event>) {
    Coordinator::new(move_rx, app_tx).run().await;
}
