use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    lokai_dir: PathBuf,
    database_url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        let lokai_dir = dirs::home_dir()
            .unwrap_or_else(|| {
                std::env::current_dir().expect("cannot get current working directory")
            })
            .join(".lokai");

        create_dir_if_not_exists(&lokai_dir);

        let database_url = format!(
            "sqlite://{}/db.sqlite",
            lokai_dir.to_str().expect("cannot create database url")
        );

        let config = Config {
            lokai_dir,
            database_url,
        };

        create_dir_if_not_exists(&config.logs_dir());
        create_dir_if_not_exists(&config.kalosm_cache_dir());

        config
    }

    pub fn database_url(&self) -> &str {
        &self.database_url
    }

    pub fn logs_dir(&self) -> PathBuf {
        self.lokai_dir.join("logs")
    }

    pub fn kalosm_cache_dir(&self) -> PathBuf {
        self.lokai_dir.join("kalosm_cache")
    }

    pub fn update_database_url(&mut self, database_url: String) {
        self.database_url = database_url;
    }
}

fn create_dir_if_not_exists(path: &PathBuf) {
    if !path.exists() {
        std::fs::create_dir(path).unwrap_or_else(|_| panic!("cannot create dir: {path:?}"));
    }
}
