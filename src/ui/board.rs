use crate::app::screen::GameScreen;
use crate::chess::{Color, Piece, Role, Square};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color as RColor, Modifier, Style},
    widgets::Widget,
};

pub fn render(frame: &mut ratatui::Frame, area: Rect, game: &GameScreen) {
    frame.render_widget(ChessBoard::new(game), area);
}

struct ChessBoard<'a> {
    game: &'a GameScreen,
}

impl<'a> ChessBoard<'a> {
    fn new(game: &'a GameScreen) -> Self {
        Self { game }
    }

    fn square_color(rank: u8, file: u8) -> RColor {
        if (rank + file) % 2 != 0 {
            RColor::Rgb(240, 217, 181)
        } else {
            RColor::Rgb(181, 136, 99)
        }
    }

    fn piece_char(piece: Piece) -> &'static str {
        match (piece.color, piece.role) {
            (Color::White, Role::King) => "♔",
            (Color::White, Role::Queen) => "♕",
            (Color::White, Role::Rook) => "♖",
            (Color::White, Role::Bishop) => "♗",
            (Color::White, Role::Knight) => "♘",
            (Color::White, Role::Pawn) => "♙",
            (Color::Black, Role::King) => "♚",
            (Color::Black, Role::Queen) => "♛",
            (Color::Black, Role::Rook) => "♜",
            (Color::Black, Role::Bishop) => "♝",
            (Color::Black, Role::Knight) => "♞",
            (Color::Black, Role::Pawn) => "♟",
        }
    }

    fn rank_label(rank: u8) -> char {
        (b'8' - rank) as char
    }

    fn file_label(file: u8) -> char {
        (b'a' + file) as char
    }
}

impl<'a> Widget for ChessBoard<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let Some(snapshot) = self.game.snapshot() else {
            return;
        };

        let rank_height = area.height.saturating_sub(2) / 8;
        let file_width = area.width.saturating_sub(3) / 8;
        if rank_height < 1 || file_width < 2 {
            return;
        }

        for rank in 0..8u8 {
            for file in 0..8u8 {
                let sq = Square::new((rank * 8 + file) as u32);
                let x = area.x + 1 + file as u16 * file_width;
                let y = area.y + 1 + (7 - rank) as u16 * rank_height;

                let bg_color = Self::square_color(rank, file);
                let mut style = Style::new().bg(bg_color);
                let mut extra_modifier: Option<Modifier> = None;

                // Priority order (chess-tui pattern):
                // 1. cursor (lowest)
                // 2. last move highlight
                // 3. selected piece
                // 4. king in check (highest — overrides everything, + SLOW_BLINK)
                if Some(sq) == Some(self.game.cursor()) {
                    style = style.bg(RColor::Rgb(100, 111, 64));
                }

                if let Some(last_move) = snapshot.last_move {
                    if last_move.from() == Some(sq) || last_move.to() == sq {
                        style = style.bg(RColor::Rgb(205, 210, 106));
                    }
                }

                if Some(sq) == self.game.selected() {
                    style = style.bg(RColor::Rgb(130, 151, 105));
                }

                if snapshot.is_check {
                    if let Some(king_sq) = snapshot.board.king_of(snapshot.turn) {
                        if sq == king_sq {
                            style = style.bg(RColor::Red);
                            extra_modifier = Some(Modifier::SLOW_BLINK);
                        }
                    }
                }

                let cell_content = if let Some(piece) = snapshot.board.piece_at(sq) {
                    Self::piece_char(piece)
                } else {
                    if self.game.selected().is_some()
                        && self
                            .game
                            .legal_moves_from_selected()
                            .iter()
                            .any(|mv| mv.to() == sq)
                    {
                        "·"
                    } else {
                        "  "
                    }
                };

                let mut content_style = if cell_content == "·" {
                    style.add_modifier(Modifier::DIM)
                } else {
                    style
                };

                if let Some(modifier) = extra_modifier {
                    content_style = content_style.add_modifier(modifier);
                }
                buf.set_string(
                    x + (file_width / 2).saturating_sub(1),
                    y,
                    cell_content,
                    content_style,
                );
            }

            let rank_y = area.y + 1 + (7 - rank) as u16 * rank_height;
            buf.set_string(
                area.x,
                rank_y,
                Self::rank_label(rank).to_string(),
                Style::new(),
            );
        }

        for file in 0..8u8 {
            let file_x = area.x + 1 + file as u16 * file_width;
            buf.set_string(
                file_x,
                area.y + area.height - 1,
                Self::file_label(file).to_string(),
                Style::new(),
            );
        }

        let block = ratatui::widgets::Block::bordered().title("Chess Board");
        block.render(area, buf);
    }
}
