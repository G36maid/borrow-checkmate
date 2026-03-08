use crate::app::screen::GameScreen;
use crate::chess::Outcome;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color as RColor, Style},
    widgets::{Paragraph, Widget},
};

pub fn render(frame: &mut ratatui::Frame, area: Rect, game: &GameScreen) {
    if let Some(outcome) = game.game_over() {
        frame.render_widget(GameOverPopup::new(*outcome), area);
    }
}

struct GameOverPopup {
    outcome: Outcome,
}

impl GameOverPopup {
    fn new(outcome: Outcome) -> Self {
        Self { outcome }
    }

    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
                ratatui::layout::Constraint::Percentage(percent_y),
                ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
                ratatui::layout::Constraint::Percentage(percent_x),
                ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}

impl Widget for GameOverPopup {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let popup_area = Self::centered_rect(60, 50, area);

        let winner_text = match &self.outcome {
            shakmaty::Outcome::Known(shakmaty::KnownOutcome::Decisive { winner }) => {
                format!("{} wins!", winner)
            }
            shakmaty::Outcome::Known(shakmaty::KnownOutcome::Draw) => "Draw!".to_string(),
            _ => return,
        };

        let outcome_detail = match &self.outcome {
            shakmaty::Outcome::Known(shakmaty::KnownOutcome::Decisive { .. }) => "Checkmate",
            shakmaty::Outcome::Known(shakmaty::KnownOutcome::Draw) => "Stalemate",
            _ => return,
        };

        let lines = vec![
            ratatui::text::Line::from(""),
            ratatui::text::Line::from(""),
            ratatui::text::Line::from(vec![ratatui::text::Span::styled(
                "GAME OVER",
                Style::default().fg(RColor::Yellow).bold(),
            )])
            .centered(),
            ratatui::text::Line::from(""),
            ratatui::text::Line::from(""),
            ratatui::text::Line::from(vec![ratatui::text::Span::styled(
                &winner_text,
                Style::default().fg(RColor::Yellow).bold(),
            )])
            .centered(),
            ratatui::text::Line::from(vec![ratatui::text::Span::styled(
                outcome_detail,
                Style::default().fg(RColor::Gray),
            )])
            .centered(),
            ratatui::text::Line::from(""),
            ratatui::text::Line::from(""),
            ratatui::text::Line::from(vec![ratatui::text::Span::styled(
                "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
                Style::default().fg(RColor::Gray),
            )])
            .centered(),
            ratatui::text::Line::from(""),
            ratatui::text::Line::from(vec![ratatui::text::Span::styled(
                "[N] New Game",
                Style::default().fg(RColor::LightGreen),
            )])
            .centered(),
            ratatui::text::Line::from(vec![ratatui::text::Span::styled(
                "[Q] Quit",
                Style::default().fg(RColor::LightCyan),
            )])
            .centered(),
            ratatui::text::Line::from(""),
        ];

        let block = ratatui::widgets::Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Double)
            .border_style(Style::default().fg(RColor::Yellow))
            .style(Style::default().bg(RColor::DarkGray));

        let inner = block.inner(popup_area);

        if inner.height > lines.len() as u16 {
            block.render(popup_area, buf);

            let paragraph = Paragraph::new(lines).alignment(ratatui::layout::Alignment::Center);
            paragraph.render(inner, buf);
        }
    }
}
