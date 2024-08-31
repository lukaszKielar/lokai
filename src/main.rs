use std::{env, io};

use handler::handle_tick_events;
use ratatui::{backend::CrosstermBackend, Terminal};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::Executor;

use crate::{
    app::{App, AppResult},
    event::{Event, EventHandler},
    handler::handle_key_events,
    tui::Tui,
};

pub mod app;
pub mod config;
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

    let mut app: App = App::new(sqlite);
    app.init().await?;

    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    while app.running {
        tui.draw(&mut app)?;

        match tui.events.next().await? {
            Event::Key(key_event) => handle_key_events(key_event, &mut app).await?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            Event::Tick => handle_tick_events(&mut app).await?,
        }
    }

    tui.exit()?;

    Ok(())
}
