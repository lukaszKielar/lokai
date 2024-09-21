use clap::Parser;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AppConfig {
    ollama_url: String,
    default_llm_model: String,
    database_url: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ollama_url: "http://localhost:11434".into(),
            default_llm_model: "phi3:3.8b".into(),
            database_url: "sqlite::memory:".into(),
        }
    }
}

impl AppConfig {
    pub fn get_ollama_url(&self) -> &str {
        &self.ollama_url
    }

    pub fn get_default_llm_model(&self) -> &str {
        &self.default_llm_model
    }

    pub fn get_database_url(&self) -> &str {
        &self.database_url
    }

    pub fn set_default_llm_model(&mut self, default_llm_model: String) {
        self.default_llm_model = default_llm_model;
    }

    pub fn update_from_cli_args(&mut self, cli_args: AppConfigCliArgs) {
        if let Some(ref ollama_url) = cli_args.database_url {
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
    /// Sqlite database URL
    #[arg(long)]
    database_url: Option<String>,
}
