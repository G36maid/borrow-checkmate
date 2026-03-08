use crate::app::screen::GameScreen;
use crate::chess::{Color, Piece, Role, Square};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color as RColor, Style},
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

    fn from_index(index: u8) -> Square {
        match index {
            0 => Square::A1,
            1 => Square::B1,
            2 => Square::C1,
            3 => Square::D1,
            4 => Square::E1,
            5 => Square::F1,
            6 => Square::G1,
            7 => Square::H1,
            8 => Square::A2,
            9 => Square::B2,
            10 => Square::C2,
            11 => Square::D2,
            12 => Square::E2,
            13 => Square::F2,
            14 => Square::G2,
            15 => Square::H2,
            16 => Square::A3,
            17 => Square::B3,
            18 => Square::C3,
            19 => Square::D3,
            20 => Square::E3,
            21 => Square::F3,
            22 => Square::G3,
            23 => Square::H3,
            24 => Square::A4,
            25 => Square::B4,
            26 => Square::C4,
            27 => Square::D4,
            28 => Square::E4,
            29 => Square::F4,
            30 => Square::G4,
            31 => Square::H4,
            32 => Square::A5,
            33 => Square::B5,
            34 => Square::C5,
            35 => Square::D5,
            36 => Square::E5,
            37 => Square::F5,
            38 => Square::G5,
            39 => Square::H5,
            40 => Square::A6,
            41 => Square::B6,
            42 => Square::C6,
            43 => Square::D6,
            44 => Square::E6,
            45 => Square::F6,
            46 => Square::G6,
            47 => Square::H6,
            48 => Square::A7,
            49 => Square::B7,
            50 => Square::C7,
            51 => Square::D7,
            52 => Square::E7,
            53 => Square::F7,
            54 => Square::G7,
            55 => Square::H7,
            56 => Square::A8,
            57 => Square::B8,
            58 => Square::C8,
            59 => Square::D8,
            60 => Square::E8,
            61 => Square::F8,
            62 => Square::G8,
            63 => Square::H8,
            _ => Square::A1,
        }
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
                let sq = Self::from_index(rank * 8 + file);
                let x = area.x + 1 + file as u16 * file_width;
                let y = area.y + 1 + (7 - rank) as u16 * rank_height;

                let bg_color = Self::square_color(rank, file);
                let mut style = Style::new().bg(bg_color);

                if Some(sq) == self.game.selected() {
                    style = style.bg(RColor::Rgb(130, 151, 105));
                } else if Some(sq) == Some(self.game.cursor()) {
                    style = style.bg(RColor::Rgb(100, 111, 64));
                }

                if let Some(last_move) = snapshot.last_move {
                    if Some(sq) == last_move.from() || Some(sq) == Some(last_move.to()) {
                        style = style.bg(RColor::Rgb(205, 210, 106));
                    }
                }

                if snapshot.is_check {
                    if let Some(king_sq) = snapshot.board.king_of(snapshot.turn) {
                        if Some(sq) == Some(king_sq) {
                            style = style.bg(RColor::Red);
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

                let content_style = if cell_content == "·" {
                    style.add_modifier(ratatui::style::Modifier::DIM)
                } else {
                    style
                };
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
