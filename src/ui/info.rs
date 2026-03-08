use crate::app::screen::GameScreen;
use crate::chess::{Color, Role};
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

        let turn_text = match snapshot.turn {
            Color::White => "White's turn",
            Color::Black => "Black's turn",
        };
        lines.push(Line::from(vec![Span::styled(
            turn_text,
            Style::new().fg(RColor::Cyan).bold(),
        )]));

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
                Span::styled(captured, Style::new().fg(RColor::Gray)),
            ]));
        }

        lines.push(Line::default());
        lines.push(Line::from("Controls: [n] New  [q] Quit"));

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
