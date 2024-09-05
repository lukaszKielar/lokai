use std::{env, error::Error, io, result::Result};

use ratatui::{backend::CrosstermBackend, Terminal};
use sqlx::{sqlite::SqlitePoolOptions, Executor};
use tokio::sync::mpsc;

use crate::{app::App, event::EventHandler, tui::Tui};

pub mod app;
pub mod chat;
pub mod conversations;
pub mod db;
pub mod event;
pub mod models;
pub mod ollama;
pub mod prompt;
pub mod tui;
pub mod ui;

pub type AppResult<T> = Result<T, Box<dyn Error>>;

#[tokio::main]
async fn main() -> AppResult<()> {
    let sqlite = SqlitePoolOptions::new()
        .min_connections(10)
        .max_connections(50)
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                conn.execute("PRAGMA foreign_keys = ON;").await?;
                Ok(())
            })
        })
        .connect(&env::var("DATABASE_URL").unwrap_or("sqlite::memory:".to_string()))
        .await
        .expect("Cannot make a DB pool");

    let (event_tx, event_rx) = mpsc::unbounded_channel();

    let mut app: App = App::new(sqlite, event_tx.clone());
    app.init().await?;

    let mut event_handler = EventHandler::new(250, event_tx, event_rx);

    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let mut tui = Tui::new(terminal);
    tui.init()?;

    while app.is_running() {
        tui.draw(&mut app)?;

        let event = event_handler.next().await?;
        app.handle_events(event).await?;
    }

    tui.exit()?;

    Ok(())
}
