use std::{
    error::Error,
    io,
    result::Result,
    sync::{Arc, LazyLock},
};

use clap::Parser;
use config::Config;
use kalosm::language::{Llama, LlamaSource};
use kalosm_language::kalosm_llama::Cache;
use kalosm_sound::{Whisper, WhisperLanguage, WhisperSource};
use ratatui::{backend::CrosstermBackend, Terminal};
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, Executor, SqlitePool};
use tokio::sync::{mpsc, RwLock};
use tracing::{info, Level};
use tracing_subscriber::EnvFilter;
use transcribe::Transcriber;

use crate::{app::App, event::EventHandler, tui::Tui};

pub mod app;
pub mod assistant;
pub mod chat;
pub mod config;
pub mod conversations;
pub mod db;
pub mod event;
pub mod models;
pub mod prompt;
pub mod transcribe;
pub mod tui;
pub mod ui;

pub type AppResult<T> = Result<T, Box<dyn Error>>;

#[derive(Parser)]
pub struct CliArgs {
    /// Sqlite database URL ["sqlite::memory:" (in-memory), "sqlite://db.slite3" (persistent), "db.sqlite3" (persitent)]
    #[arg(long)]
    database_url: Option<String>,
    /// Enables prompt transcription
    #[arg(long, action = clap::ArgAction::SetTrue)]
    enable_transcription: bool,
}

static CONFIG: LazyLock<Arc<RwLock<Config>>> =
    LazyLock::new(|| Arc::new(RwLock::new(Config::new())));

#[tokio::main]
async fn main() -> AppResult<()> {
    let log_file = {
        let logs_dir = CONFIG.read().await.logs_dir();
        tracing_appender::rolling::daily(logs_dir, "lokai.log")
    };
    let (non_blocking, _guard) = tracing_appender::non_blocking(log_file);
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    info!("starting");

    let cli_args = CliArgs::parse();
    {
        if let Some(database_url) = &cli_args.database_url {
            let mut config = CONFIG.write().await;
            config.update_database_url(database_url.clone());
        }
    }

    let sqlite = {
        let config = CONFIG.read().await;
        setup_sqlite_pool(config.database_url()).await?
    };

    let (event_tx, event_rx) = mpsc::unbounded_channel();

    let kalosm_cache = {
        let kalosm_cache_dir = CONFIG.read().await.kalosm_cache_dir();
        Cache::new(kalosm_cache_dir)
    };

    let _transcriber = {
        if cli_args.enable_transcription {
            let whisper = Whisper::builder()
                .with_cache(kalosm_cache.clone())
                .with_source(WhisperSource::BaseEn)
                .with_language(Some(WhisperLanguage::English))
                .build()
                .await?;

            Some(Transcriber::new(event_tx.clone(), whisper))
        } else {
            None
        }
    };

    let llama = Llama::builder()
        .with_source(LlamaSource::llama_3_1_8b_chat().with_cache(kalosm_cache))
        .build()
        .await?;

    let mut app: App = App::new(sqlite, event_tx.clone(), llama);
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

async fn setup_sqlite_pool(database_url: &str) -> AppResult<SqlitePool> {
    if !sqlx::Sqlite::database_exists(database_url).await? {
        sqlx::Sqlite::create_database(database_url).await?;
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
        .connect(database_url)
        .await
        .expect("Cannot make a DB pool");

    sqlx::migrate!().run(&sqlite).await?;

    Ok(sqlite)
}
