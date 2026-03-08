pub mod board;
pub mod info;
pub mod promotion;

use crate::app::screen::GameScreen;
use ratatui::Frame;

/// Render the game screen
pub fn render_game(frame: &mut Frame, game: &GameScreen) {
    let size = frame.area();

    let main_chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .margin(1)
        .constraints([
            ratatui::layout::Constraint::Min(60),
            ratatui::layout::Constraint::Min(20),
        ])
        .split(size);

    board::render(frame, main_chunks[0], game);
    info::render(frame, main_chunks[1], game);
    promotion::render(frame, size, game);
}
