use crate::app::screen::GameScreen;
use ratatui::{layout::Rect, Frame};

pub fn render(frame: &mut Frame, area: Rect, game: &GameScreen) {
    let block = ratatui::widgets::Block::bordered().title("Info Panel");
    frame.render_widget(block, area);
}
