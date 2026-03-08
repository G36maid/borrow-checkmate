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
    if let Some((_, _)) = game.promotion_pending() {
        frame.render_widget(PromotionDialog::new(game), area);
    }
}

struct PromotionDialog<'a> {
    game: &'a GameScreen,
}

impl<'a> PromotionDialog<'a> {
    fn new(game: &'a GameScreen) -> Self {
        Self { game }
    }

    fn role_char(role: Role, color: Color) -> &'static str {
        match (color, role) {
            (Color::White, Role::Queen) => "♕",
            (Color::White, Role::Rook) => "♖",
            (Color::White, Role::Bishop) => "♗",
            (Color::White, Role::Knight) => "♘",
            (Color::Black, Role::Queen) => "♛",
            (Color::Black, Role::Rook) => "♜",
            (Color::Black, Role::Bishop) => "♝",
            (Color::Black, Role::Knight) => "♞",
            _ => "",
        }
    }
}

impl<'a> Widget for PromotionDialog<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let Some(snapshot) = self.game.snapshot() else {
            return;
        };

        let roles = [Role::Queen, Role::Rook, Role::Bishop, Role::Knight];
        let role_labels = ["Q", "R", "B", "N"];

        let block = ratatui::widgets::Block::bordered()
            .title("Promote pawn to")
            .title_style(Style::new().fg(RColor::Yellow));

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.height < 3 {
            return;
        }

        let piece_row = inner.y + 1;
        let label_row = inner.y + 2;

        let width_per_role = inner.width / 4;

        for (i, &role) in roles.iter().enumerate() {
            let x = inner.x + i as u16 * width_per_role + (width_per_role / 2).saturating_sub(1);

            let piece = Self::role_char(role, snapshot.turn);
            buf.set_string(x, piece_row, piece, Style::new().bold());

            buf.set_string(x, label_row, role_labels[i], Style::new().fg(RColor::Gray));
        }

        let center_x = inner.x + inner.width / 2;
        let help_text = "Arrow keys + Enter";
        buf.set_string(
            center_x.saturating_sub(help_text.len() as u16 / 2),
            inner.y,
            help_text,
            Style::new().fg(RColor::DarkGray),
        );
    }
}
