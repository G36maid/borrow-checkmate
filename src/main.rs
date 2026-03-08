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

    let (cmd_tx, cmd_rx) = mpsc::channel::<coordinator::CoordinatorCommand>(1);
    let events = event::EventHandler::new();
    let app_tx = events.sender();

    tokio::spawn(coordinator::run(cmd_rx, app_tx));

    let terminal = ratatui::init();
    let result = app::App::new(events, cmd_tx).run(terminal).await;
    ratatui::restore();
    result
}
