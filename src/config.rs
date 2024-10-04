use clap::Parser;
use serde::Deserialize;

use crate::LOKAI_DIR;

#[derive(Deserialize, Debug)]
pub struct AppConfig {
    // TODO: rename to sqlite_url
    database_url: String,
}

impl AppConfig {
    pub fn init() -> Self {
        Self {
            database_url: "".to_string(),
        }
    }

    pub fn get_database_url(&self) -> &str {
        &self.database_url
    }

    pub fn update_from_cli_args(&mut self, cli_args: AppConfigCliArgs) {
        if let Some(ref database_url) = cli_args.database_url {
            self.database_url = database_url.to_string();
        }
    }
}

#[derive(Parser)]
pub struct AppConfigCliArgs {
    /// Sqlite database URL ["sqlite::memory:" (in-memory), "sqlite://db.slite3" (persistent), "db.sqlite3" (persitent)]
    #[arg(long)]
    database_url: Option<String>,
}

impl From<AppConfigCliArgs> for AppConfig {
    fn from(value: AppConfigCliArgs) -> Self {
        Self {
            database_url: value.database_url.unwrap_or_else(|| {
                format!(
                    "sqlite://{}/db.sqlite",
                    LOKAI_DIR.to_str().expect("Cannot get lokai config dir")
                )
            }),
        }
    }
}
