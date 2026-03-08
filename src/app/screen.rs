use crate::chess::{GameSnapshot, Move, MoveList, Outcome, Role, Square};
use crossterm::event::KeyCode;
use ratatui::Frame;

fn square_up(sq: Square) -> Square {
    let current = sq as u32;
    if current >= 56 {
        sq
    } else {
        Square::new(current + 8)
    }
}

fn square_down(sq: Square) -> Square {
    let current = sq as u32;
    if current < 8 {
        sq
    } else {
        Square::new(current - 8)
    }
}

fn square_left(sq: Square) -> Square {
    let current = sq as u32;
    let file = current % 8;
    if file == 0 {
        sq
    } else {
        Square::new(current - 1)
    }
}

fn square_right(sq: Square) -> Square {
    let current = sq as u32;
    let file = current % 8;
    if file == 7 {
        sq
    } else {
        Square::new(current + 1)
    }
}

/// Screen state for the chess game board
pub struct GameScreen {
    snapshot: Option<GameSnapshot>,
    cursor: Square,
    selected: Option<Square>,
    legal_from_selected: MoveList,
    promotion_pending: Option<(Square, Square)>,
    promotion_cursor: usize,
    game_over: Option<Outcome>,
    illegal_flash: u8,
}

impl Default for GameScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl GameScreen {
    pub fn new() -> Self {
        Self {
            snapshot: None,
            cursor: Square::E4,
            selected: None,
            legal_from_selected: MoveList::new(),
            promotion_pending: None,
            promotion_cursor: 0,
            game_over: None,
            illegal_flash: 0,
        }
    }

    pub fn apply_snapshot(&mut self, snapshot: GameSnapshot) {
        self.snapshot = Some(snapshot);
        self.illegal_flash = 0;
    }

    pub fn set_game_over(&mut self, outcome: Outcome) {
        self.game_over = Some(outcome);
    }

    pub fn flash_illegal(&mut self) {
        self.illegal_flash = 10;
    }

    pub fn wants_esc(&self) -> bool {
        self.promotion_pending.is_some() || self.selected.is_some()
    }

    pub fn handle_esc(&mut self) {
        if self.promotion_pending.is_some() {
            self.promotion_pending = None;
            self.promotion_cursor = 0;
        } else if self.selected.is_some() {
            self.selected = None;
            self.legal_from_selected.clear();
        }
    }

    pub fn handle_key(&mut self, key: KeyCode) -> Option<Move> {
        if self.game_over.is_some() {
            return None;
        }

        let _snapshot = self.snapshot.as_ref()?;

        match key {
            KeyCode::Up => {
                self.cursor = square_up(self.cursor);
                None
            }
            KeyCode::Down => {
                self.cursor = square_down(self.cursor);
                None
            }
            KeyCode::Left => {
                if self.promotion_pending.is_some() {
                    self.promotion_cursor = if self.promotion_cursor > 0 {
                        self.promotion_cursor - 1
                    } else {
                        3
                    };
                } else {
                    self.cursor = square_left(self.cursor);
                }
                None
            }
            KeyCode::Right => {
                if self.promotion_pending.is_some() {
                    self.promotion_cursor = (self.promotion_cursor + 1) % 4;
                } else {
                    self.cursor = square_right(self.cursor);
                }
                None
            }
            KeyCode::Enter => self.handle_enter(),
            _ => None,
        }
    }

    fn handle_enter(&mut self) -> Option<Move> {
        let snapshot = self.snapshot.as_ref()?;

        if let Some((from, to)) = self.promotion_pending {
            let roles = [Role::Queen, Role::Rook, Role::Bishop, Role::Knight];
            let promotion_role = roles[self.promotion_cursor];
            let mv = Move::Normal {
                role: Role::Pawn,
                from,
                to,
                capture: snapshot.board.piece_at(to).map(|p| p.role),
                promotion: Some(promotion_role),
            };
            self.promotion_pending = None;
            self.promotion_cursor = 0;
            self.selected = None;
            Some(mv)
        } else if let Some(selected) = self.selected {
            let mv_idx = self
                .legal_from_selected
                .iter()
                .position(|mv| mv.to() == self.cursor);
            if let Some(idx) = mv_idx {
                let mv = self.legal_from_selected[idx];
                if mv.is_promotion() {
                    self.promotion_pending = Some((selected, self.cursor));
                    None
                } else {
                    self.selected = None;
                    self.legal_from_selected.clear();
                    Some(mv)
                }
            } else {
                self.selected = None;
                self.legal_from_selected.clear();
                None
            }
        } else if snapshot.board.piece_at(self.cursor).is_some() {
            let piece = snapshot.board.piece_at(self.cursor)?;
            if piece.color == snapshot.turn {
                self.selected = Some(self.cursor);
                self.legal_from_selected = snapshot
                    .legal_moves
                    .iter()
                    .filter(|mv| mv.from() == Some(self.cursor))
                    .copied()
                    .collect();
                None
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn cursor(&self) -> Square {
        self.cursor
    }

    pub fn selected(&self) -> Option<Square> {
        self.selected
    }

    pub fn legal_moves_from_selected(&self) -> &[Move] {
        &self.legal_from_selected
    }

    pub fn snapshot(&self) -> Option<&GameSnapshot> {
        self.snapshot.as_ref()
    }

    pub fn promotion_pending(&self) -> Option<(Square, Square)> {
        self.promotion_pending
    }

    pub fn promotion_cursor(&self) -> usize {
        self.promotion_cursor
    }

    pub fn game_over(&self) -> Option<&Outcome> {
        self.game_over.as_ref()
    }

    pub fn illegal_flash(&self) -> u8 {
        self.illegal_flash
    }

    pub fn tick(&mut self) {
        if self.illegal_flash > 0 {
            self.illegal_flash -= 1;
        }
    }
}

pub enum Screen {
    Game(GameScreen),
}

impl Screen {
    pub fn render(&self, frame: &mut Frame) {
        match self {
            Screen::Game(game) => {
                crate::ui::render_game(frame, game);
            }
        }
    }

    pub fn tick(&mut self) {
        match self {
            Screen::Game(game) => game.tick(),
        }
    }

    pub fn wants_esc(&self) -> bool {
        match self {
            Screen::Game(game) => game.wants_esc(),
        }
    }

    pub fn handle_esc(&mut self) {
        match self {
            Screen::Game(game) => game.handle_esc(),
        }
    }

    pub fn handle_key(&mut self, key: KeyCode) -> Option<Move> {
        match self {
            Screen::Game(game) => game.handle_key(key),
        }
    }

    pub fn apply_snapshot(&mut self, snapshot: GameSnapshot) {
        match self {
            Screen::Game(game) => game.apply_snapshot(snapshot),
        }
    }

    pub fn set_game_over(&mut self, outcome: Outcome) {
        match self {
            Screen::Game(game) => game.set_game_over(outcome),
        }
    }

    pub fn flash_illegal(&mut self) {
        match self {
            Screen::Game(game) => game.flash_illegal(),
        }
    }
}
