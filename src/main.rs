use std::{error::Error, io, path::PathBuf, result::Result, sync::LazyLock, time::Duration};

use clap::Parser;
use config::{AppConfig, AppConfigCliArgs};
use kalosm_language::kalosm_llama::Cache;
use kalosm_sound::{Whisper, WhisperLanguage, WhisperSource};
use ratatui::{backend::CrosstermBackend, Terminal};
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, Executor, SqlitePool};
use tokio::sync::{mpsc, RwLock};
use tracing::{info, Level};
use transcribe::Transcriber;

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
pub mod transcribe;
pub mod tui;
pub mod ui;

pub type AppResult<T> = Result<T, Box<dyn Error>>;

static LOKAI_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let lokai_dir = dirs::home_dir()
        .unwrap_or_else(|| std::env::current_dir().expect("Cannot get current working directory"))
        .join(".lokai");

    if !lokai_dir.exists() {
        std::fs::create_dir(&lokai_dir)
            .unwrap_or_else(|_| panic!("cannot create {:?} directory", lokai_dir))
    }

    lokai_dir
});
static APP_CONFIG: LazyLock<RwLock<AppConfig>> = LazyLock::new(|| RwLock::new(AppConfig::init()));

#[tokio::main]
async fn main() -> AppResult<()> {
    let logs_dir = LOKAI_DIR.join("logs");
    if !logs_dir.exists() {
        std::fs::create_dir(&logs_dir)?
    }
    let log_file = tracing_appender::rolling::daily(logs_dir, "lokai.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(log_file);
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_writer(non_blocking)
        .with_ansi(false)
        .init();

    info!("starting");

    let cli_args = AppConfigCliArgs::parse();
    {
        let mut app_config = APP_CONFIG.write().await;
        *app_config = cli_args.into();
    }

    if let Err(err) = verify_ollama_server().await {
        eprintln!("Cannot connect to Ollama server: {:?}", err.to_string());
        std::process::exit(1);
    };

    let sqlite = setup_sqlite_pool().await?;

    let (event_tx, event_rx) = mpsc::unbounded_channel();

    let _transcriber = {
        let cache_dir = LOKAI_DIR.join("kalosm_cache");
        if !cache_dir.exists() {
            std::fs::create_dir(&cache_dir)?
        }
        let cache = Cache::new(cache_dir);

        let whisper = Whisper::builder()
            .with_cache(cache)
            .with_source(WhisperSource::BaseEn)
            .with_language(Some(WhisperLanguage::English))
            .build()
            .await?;

        Transcriber::new(event_tx.clone(), whisper)
    };

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
