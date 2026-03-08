use tokio::sync::mpsc;

pub mod app;
pub mod chess;
pub mod coordinator;
pub mod event;
pub mod player;
pub mod ui;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let (move_tx, move_rx) = mpsc::channel::<chess::Move>(1);
    let events = event::EventHandler::new();
    let app_tx = events.sender();

    tokio::spawn(coordinator::run(move_rx, app_tx));

    let terminal = ratatui::init();
    let result = app::App::new(events, move_tx).run(terminal).await;
    ratatui::restore();
    result
}
