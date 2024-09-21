use std::{env, path::PathBuf};

use config::{Config, File};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AppConfig {
    pub ollama_url: String,
    pub llm_model: String,
    pub database_url: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ollama_url: "http://localhost:11434".into(),
            llm_model: "phi3:3.8b".into(),
            database_url: "sqlite::memory:".into(),
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        let config_path = env::var("LOKAI_CONFIG")
            .map(PathBuf::from)
            .ok()
            .unwrap_or_else(|| {
                dirs::config_local_dir()
                    .expect("Cannot get local config dir")
                    .join("LokAI")
                    .join("config.toml")
            });

        if !config_path.exists() {
            eprintln!("Config file {:?} doesn't exist", config_path);
            std::process::exit(1);
        }

        let cfg = Config::builder()
            .add_source(File::with_name(
                config_path.with_extension("").to_str().unwrap(),
            ))
            .build()
            .expect("Cannot build config from file");

        cfg.try_deserialize().expect("Cannot deserialise config")
    }
}
