use std::error;

use crate::config::Config;

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub config: Config,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            config: Default::default(),
        }
    }
}

impl App {
    pub fn quit(&mut self) {
        self.running = false;
    }
}
