use shakmaty::{Chess, Color, Move, Outcome, Position, Square};
use std::fmt;

// Re-export shakmaty types directly
pub use shakmaty::{Board, Color, Move, Outcome, Piece, Role, Square};
pub use shakmaty::MoveList;
pub use shakmaty::PlayError;

/// Snapshot of the game state sent to TUI after each move
#[derive(Clone, Debug)]
pub struct GameSnapshot {
    pub board: Board,
    pub turn: Color,
    pub is_check: bool,
    pub legal_moves: MoveList,
    pub last_move: Option<Move>,
    pub outcome: Option<Outcome>,
    pub move_history: Vec<Move>,
    pub captured_white: Vec<Role>,
    pub captured_black: Vec<Role>,
}

/// Game wrapper around shakmaty's Chess with undo history
pub struct Game {
    pos: Chess,
    history: Vec<Chess>,
    move_history: Vec<Move>,
    captured_white: Vec<Role>,
    captured_black: Vec<Role>,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    /// Create a new game from the starting position
    pub fn new() -> Self {
        Self {
            pos: Chess::default(),
            history: Vec::new(),
            move_history: Vec::new(),
            captured_white: Vec::new(),
            captured_black: Vec::new(),
        }
    }

    /// Get a snapshot of the current game state
    pub fn snapshot(&self) -> GameSnapshot {
        GameSnapshot {
            board: self.pos.board().clone(),
            turn: self.pos.turn(),
            is_check: self.pos.is_check(),
            legal_moves: self.pos.legal_moves(),
            last_move: self.move_history.last().copied(),
            outcome: self.outcome(),
            move_history: self.move_history.clone(),
            captured_white: self.captured_white.clone(),
            captured_black: self.captured_black.clone(),
        }
    }

    /// Get all legal moves from a specific square
    pub fn legal_moves_from(&self, sq: Square) -> MoveList {
        self.pos
            .legal_moves()
            .into_iter()
            .filter(|mv| mv.from() == Some(sq))
            .collect()
    }

    /// Apply a move to the game
    pub fn make_move(&mut self, mv: Move) -> Result<(), PlayError<Chess>> {
        // Save current state for undo
        self.history.push(self.pos.clone());

        // Track captures
        if let Some(capture) = mv.capture() {
            let capturer = mv.role();
            let capture_color = self.pos.turn();
            if capture_color == Color::White {
                // Black captured a white piece
                self.captured_black.push(capture);
            } else {
                // White captured a black piece
                self.captured_white.push(capture);
            }
        }

        // Apply the move
        self.pos = self.pos.play(mv)?;
        self.move_history.push(mv);

        Ok(())
    }

    /// Undo the last move
    pub fn undo(&mut self) -> bool {
        if let Some(prev_pos) = self.history.pop() {
            self.pos = prev_pos;
            self.move_history.pop();
            
            // Recompute captured pieces by simulating from start
            let mut temp_pos = Chess::default();
            self.captured_white.clear();
            self.captured_black.clear();
            
            // Simulate all moves except the last one (which we just undid)
            for mv in &self.move_history {
                if let Some(capture) = mv.capture() {
                    let capture_color = temp_pos.turn();
                    if capture_color == Color::White {
                        self.captured_black.push(capture);
                    } else {
                        self.captured_white.push(capture);
                    }
                }
                temp_pos = temp_pos.play(*mv).ok();
            }
            
            true
        } else {
            false
        }
    }

    /// Get the game outcome (None if game still in progress)
    pub fn outcome(&self) -> Option<Outcome> {
        self.pos.outcome()
    }
}

impl fmt::Debug for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Game")
            .field("turn", &self.pos.turn())
            .field("is_check", &self.pos.is_check())
            .field("outcome", &self.outcome())
            .finish()
    }
}
