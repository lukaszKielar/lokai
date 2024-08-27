#[derive(Debug)]
pub struct Config {
    pub ollama_url: String,
    pub default_llm_model: String,
}

// TODO: read config from toml config file
impl Default for Config {
    fn default() -> Self {
        Self {
            ollama_url: "http://host.docker.internal:11434".to_string(),
            default_llm_model: "phi3:3.8".to_string(),
        }
    }
}
