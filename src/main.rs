use std::{env, io};

use ratatui::{backend::CrosstermBackend, Terminal};
use sqlx::sqlite::SqlitePoolOptions;

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
pub mod prompt;
pub mod tui;
pub mod ui;

#[tokio::main]
async fn main() -> AppResult<()> {
    let sqlite = SqlitePoolOptions::new()
        .max_connections(10)
        .connect(&env::var("DATABASE_URL").unwrap_or("sqlite://db.sqlite3".to_string()))
        .await
        .expect("Cannot make a DB pool");

    let mut app: App = Default::default();
    app.init(sqlite).await?;

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
            Event::Tick => {}
        }
    }

    tui.exit()?;

    Ok(())
}
