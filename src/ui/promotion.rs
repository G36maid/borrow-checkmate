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

        let popup_width = area.width.min(40);
        let popup_height = area.height.min(7);

        let popup_x = area.x + (area.width - popup_width) / 2;
        let popup_y = area.y + (area.height - popup_height) / 2;
        let popup = Rect::new(popup_x, popup_y, popup_width, popup_height);

        let block = ratatui::widgets::Block::bordered()
            .title("Promote pawn to:")
            .title_style(Style::new().fg(RColor::Yellow));

        let inner = block.inner(popup);
        block.render(popup, buf);

        if inner.height < 4 {
            return;
        }

        let piece_row = inner.y + 1;
        let label_row = inner.y + 2;
        let caret_row = inner.y + 3;

        let width_per_role = inner.width / 4;

        let cursor_idx = self.game.promotion_cursor();

        for (i, &role) in roles.iter().enumerate() {
            let x = inner.x + i as u16 * width_per_role + (width_per_role / 2).saturating_sub(1);

            let piece = Self::role_char(role, snapshot.turn);
            let label = role_labels[i];
            let is_selected = i == cursor_idx;

            let style = if is_selected {
                Style::new().bg(RColor::Cyan).fg(RColor::Black).bold()
            } else {
                Style::new().bold()
            };

            buf.set_string(x, piece_row, piece, style);

            let label_style = if is_selected {
                Style::new().bg(RColor::Cyan).fg(RColor::Black).bold()
            } else {
                Style::new().fg(RColor::Gray)
            };
            buf.set_string(x, label_row, label, label_style);

            if is_selected {
                buf.set_string(x, caret_row, "^", Style::new().fg(RColor::Cyan));
            }
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
