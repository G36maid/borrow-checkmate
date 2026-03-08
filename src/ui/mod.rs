pub mod board;
pub mod info;
pub mod promotion;

use crate::app::screen::GameScreen;
use ratatui::Frame;

/// Render the game screen
pub fn render_game(frame: &mut Frame, game: &GameScreen) {
    let size = frame.area();

    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .margin(1)
        .constraints([
            ratatui::layout::Constraint::Min(24),
            ratatui::layout::Constraint::Length(1),
        ])
        .split(size);

    board::render(frame, chunks[0], game);
    info::render(frame, chunks[1], game);
}
