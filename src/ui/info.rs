use crate::app::screen::GameScreen;
use crate::chess::{Color, Move, Role};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color as RColor, Style},
    text::{Line, Span},
    widgets::Widget,
};

pub fn render(frame: &mut ratatui::Frame, area: Rect, game: &GameScreen) {
    frame.render_widget(InfoPanel::new(game), area);
}

struct InfoPanel<'a> {
    game: &'a GameScreen,
}

impl<'a> InfoPanel<'a> {
    fn new(game: &'a GameScreen) -> Self {
        Self { game }
    }

    fn role_char(role: Role, color: Color) -> &'static str {
        match (color, role) {
            (Color::White, Role::Queen) => "♕",
            (Color::White, Role::Rook) => "♖",
            (Color::White, Role::Bishop) => "♗",
            (Color::White, Role::Knight) => "♘",
            (Color::White, Role::Pawn) => "♙",
            (Color::Black, Role::Queen) => "♛",
            (Color::Black, Role::Rook) => "♜",
            (Color::Black, Role::Bishop) => "♝",
            (Color::Black, Role::Knight) => "♞",
            (Color::Black, Role::Pawn) => "♟",
            _ => "",
        }
    }
}

impl<'a> Widget for InfoPanel<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let Some(snapshot) = self.game.snapshot() else {
            return;
        };

        let mut lines = Vec::new();

        let turn_piece = match snapshot.turn {
            Color::White => "♙",
            Color::Black => "♟",
        };
        let turn_text = match snapshot.turn {
            Color::White => "White's turn",
            Color::Black => "Black's turn",
        };
        lines.push(Line::from(vec![
            Span::styled(turn_piece, Style::new().fg(RColor::Cyan)),
            Span::raw(" "),
            Span::styled(turn_text, Style::new().fg(RColor::Cyan).bold()),
        ]));

        if let Some(outcome) = self.game.game_over() {
            let outcome_text = format!("Game Over: {}", outcome);
            lines.push(Line::from(vec![Span::styled(
                outcome_text,
                Style::new().fg(RColor::Red).bold(),
            )]));
        } else if snapshot.is_check {
            lines.push(Line::from(vec![Span::styled(
                "Check!",
                Style::new().fg(RColor::Red).bold(),
            )]));
        }

        if self.game.illegal_flash() > 0 {
            lines.push(Line::from(vec![Span::styled(
                "Illegal move!",
                Style::new().fg(RColor::Red).bold(),
            )]));
        }

        lines.push(Line::from(""));

        if !snapshot.captured_white.is_empty() {
            let captured: String = snapshot
                .captured_white
                .iter()
                .map(|&r| Self::role_char(r, Color::White))
                .collect();
            lines.push(Line::from(vec![
                Span::raw("Captured by Black: "),
                Span::styled(captured, Style::new().fg(RColor::White)),
            ]));
        }

        if !snapshot.captured_black.is_empty() {
            let captured: String = snapshot
                .captured_black
                .iter()
                .map(|&r| Self::role_char(r, Color::Black))
                .collect();
            lines.push(Line::from(vec![
                Span::raw("Captured by White: "),
                Span::styled(captured, Style::new().fg(RColor::DarkGray)),
            ]));
        }

        lines.push(Line::default());
        if !snapshot.move_history.is_empty() {
            let max_pairs = area.height.saturating_sub(8) as usize;
            let recent_pairs_start = snapshot.move_history.len().saturating_sub(max_pairs * 2);
            let recent_moves: &[Move] = &snapshot.move_history[recent_pairs_start..];
            let move_text: String = recent_moves
                .chunks(2)
                .enumerate()
                .map(|(i, pair)| {
                    let move_num = (recent_pairs_start / 2) + i + 1;
                    let white_move = pair.get(0).map(|mv| mv.to_string()).unwrap_or_default();
                    let black_move = pair.get(1).map(|mv| mv.to_string()).unwrap_or_default();
                    format!("{}. {} {}", move_num, white_move, black_move)
                })
                .collect::<Vec<_>>()
                .join("\n");
            lines.push(Line::from(vec![
                Span::raw("Move history:\n"),
                Span::styled(move_text, Style::new().fg(RColor::Gray)),
            ]));
        }

        lines.push(Line::default());
        lines.push(Line::from("Controls: [n] New  [u] Undo  [q] Quit"));

        let y_start = area.y;
        for (i, line) in lines.iter().enumerate() {
            if (y_start + i as u16) < (area.y + area.height) {
                line.render(Rect::new(area.x, y_start + i as u16, area.width, 1), buf);
            }
        }

        let block = ratatui::widgets::Block::bordered().title("Info");
        block.render(area, buf);
    }
}
