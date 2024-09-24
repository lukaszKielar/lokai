use std::{error::Error, io, result::Result, time::Duration};

use clap::Parser;
use config::{AppConfig, AppConfigCliArgs};
use once_cell::sync::Lazy;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::fs::OpenOptions;
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, Executor, SqlitePool};
use tokio::sync::{mpsc, RwLock};
use tracing::{info, Level};
use tracing_subscriber;

use crate::{app::App, event::EventHandler, tui::Tui};

pub mod app;
pub mod chat;
pub mod config;
pub mod conversations;
pub mod db;
pub mod event;
pub mod models;
pub mod ollama;
pub mod prompt;
pub mod tui;
pub mod ui;

pub type AppResult<T> = Result<T, Box<dyn Error>>;

static APP_CONFIG: Lazy<RwLock<AppConfig>> = Lazy::new(|| RwLock::new(AppConfig::default()));

#[tokio::main]
async fn main() -> AppResult<()> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("app.log")?;

    tracing_subscriber::fmt()
        .json()
        .with_max_level(Level::INFO)
        .with_writer(file)
        .init();

    info!("starting");

    let cli_args = AppConfigCliArgs::parse();
    {
        let mut app_config = APP_CONFIG.write().await;
        app_config.update_from_cli_args(cli_args);
    }

    if let Err(err) = verify_ollama_server().await {
        eprintln!("Cannot connect to Ollama server: {:?}", err.to_string());
        std::process::exit(1);
    };

    let sqlite = setup_sqlite_pool().await?;

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

    info!("shutting down!");

    Ok(())
}

async fn setup_sqlite_pool() -> AppResult<SqlitePool> {
    {
        let app_config = APP_CONFIG.read().await;
        let database_url = app_config.get_database_url();
        if !sqlx::Sqlite::database_exists(database_url).await? {
            sqlx::Sqlite::create_database(database_url).await?;
        }
    }

    let sqlite = SqlitePoolOptions::new()
        .min_connections(10)
        .max_connections(50)
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                conn.execute("PRAGMA foreign_keys = ON;").await?;
                Ok(())
            })
        })
        .connect(APP_CONFIG.read().await.get_database_url())
        .await
        .expect("Cannot make a DB pool");

    sqlx::migrate!().run(&sqlite).await?;

    Ok(sqlite)
}

async fn verify_ollama_server() -> AppResult<()> {
    // TODO: create APP_STATE that store all different clients I need
    // introduce similar concept to axum's State
    let http_client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .build()?;

    let app_config = APP_CONFIG.read().await;
    let ollama_url = app_config.get_ollama_url();

    let response = http_client.get(ollama_url).send().await?;

    if !response.status().is_success() {
        return Err("Cannot connect to Ollama server, make sure it's up and running.".into());
    }

    Ok(())
}
