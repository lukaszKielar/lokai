use clap::Parser;
use serde::Deserialize;

use crate::LOKAI_DIR;

static OLLAMA_URL: &str = "http://localhost:11434";
static DEFAULT_LLM_MODEL: &str = "phi3.5:3.8b";

#[derive(Deserialize, Debug)]
pub struct AppConfig {
    // TODO: replace with Kalosm
    ollama_url: String,
    // TODO: I need to be opinionated and allow to define only one model
    default_llm_model: String,
    // TODO: rename to sqlite_url
    database_url: String,
}

impl AppConfig {
    pub fn init() -> Self {
        Self {
            ollama_url: "".to_string(),
            default_llm_model: "".to_string(),
            database_url: "".to_string(),
        }
    }

    pub fn get_ollama_url(&self) -> &str {
        &self.ollama_url
    }

    pub fn get_default_llm_model(&self) -> &str {
        &self.default_llm_model
    }

    pub fn get_database_url(&self) -> &str {
        &self.database_url
    }

    pub fn update_from_cli_args(&mut self, cli_args: AppConfigCliArgs) {
        if let Some(ref ollama_url) = cli_args.ollama_url {
            self.ollama_url = ollama_url.to_string();
        }

        if let Some(ref default_llm_model) = cli_args.default_llm_model {
            self.default_llm_model = default_llm_model.to_string();
        }

        if let Some(ref database_url) = cli_args.database_url {
            self.database_url = database_url.to_string();
        }
    }
}

#[derive(Parser)]
pub struct AppConfigCliArgs {
    /// Ollama server URL
    #[arg(long)]
    ollama_url: Option<String>,
    /// Default LLM Model user want to use
    #[arg(long)]
    default_llm_model: Option<String>,
    /// Sqlite database URL ["sqlite::memory:" (in-memory), "sqlite://db.slite3" (persistent), "db.sqlite3" (persitent)]
    #[arg(long)]
    database_url: Option<String>,
}

impl From<AppConfigCliArgs> for AppConfig {
    fn from(value: AppConfigCliArgs) -> Self {
        Self {
            ollama_url: value.ollama_url.unwrap_or_else(|| OLLAMA_URL.to_string()),
            default_llm_model: value
                .default_llm_model
                .unwrap_or_else(|| DEFAULT_LLM_MODEL.to_string()),
            database_url: value.database_url.unwrap_or_else(|| {
                format!(
                    "sqlite://{}/db.sqlite",
                    LOKAI_DIR.to_str().expect("Cannot get lokai config dir")
                )
            }),
        }
    }
}
