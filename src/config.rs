use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
pub(crate) struct Config {
    repos: Vec<PathBuf>,
}

impl Config {
    pub(crate) fn default() -> Result<Self, anyhow::Error> {
        let home = env::var("HOME")?;
        let config_dir = Path::new(&home).join(".config/branches");
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }
        let config_path = config_dir.join("config.json5");
        if !config_path.exists() {
            anyhow::bail!("Missing required ~/.config/branches/config.json5");
        }
        Self::from_path(&config_path)
    }

    pub(crate) fn repos(&self) -> &[PathBuf] {
        &self.repos
    }

    fn from_path<P>(path: P) -> Result<Self, anyhow::Error>
    where
        P: AsRef<Path>,
    {
        let s = fs::read_to_string(path)?;
        let config = json5::from_str::<Config>(&s)?;
        Ok(config)
    }
}
