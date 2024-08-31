use std::{env, io};

use ratatui::{backend::CrosstermBackend, Terminal};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::Executor;
use tokio::sync::mpsc;

use crate::app::{App, AppResult};
use crate::event::{Event, EventHandler};
use crate::handler::handle_key_events;
use crate::tui::Tui;

pub mod app;
pub mod crud;
pub mod event;
pub mod handler;
pub mod models;
pub mod ollama;
pub mod prompt;
pub mod tui;
pub mod ui;

#[tokio::main]
async fn main() -> AppResult<()> {
    let sqlite = SqlitePoolOptions::new()
        .min_connections(2)
        .max_connections(10)
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

    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250, event_tx, event_rx);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    while app.running {
        tui.draw(&mut app)?;

        match tui.events.next().await? {
            Event::Tick => {}
            Event::Key(key_event) => handle_key_events(key_event, &mut app).await?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            Event::Inference(message) => println!("{:?}", message),
        }
    }

    tui.exit()?;

    Ok(())
}
