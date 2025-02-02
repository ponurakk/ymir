//! Config for ymir

use std::path::PathBuf;

use config::{Config, ConfigError, File};
use serde::Deserialize;

/// Settings for ymir
#[derive(Debug, Deserialize)]
pub struct Settings {
    pub ignore_dirs: Vec<String>,
    pub default_dir: Option<PathBuf>,
}

impl Settings {
    /// Default ignore directories
    fn ignore_dirs() -> Vec<String> {
        vec![
            // Build
            "node_modules",
            "target",
            "build",
            "CMakeFiles",
            "_build",
            "venv",
            "vendor",
            ".zig-cache",
            ".zig-out",
            "dist",
            "site-packages",
            // Cache
            ".cache",
            ".gradle",
            ".nuxt",
            ".svelte-kit",
            ".mypy_cache",
        ]
        .iter()
        .map(|&v| (*v).to_string())
        .collect()
    }

    /// Load config
    pub fn new() -> anyhow::Result<Self, ConfigError> {
        let Some(config_dir) = dirs::config_dir() else {
            // TODO: Add notification
            eprintln!("Failed to find config_directory");
            return Ok(Self::default());
        };

        let config_path = format!("{}/{}/config", config_dir.display(), env!("CARGO_PKG_NAME"),);

        let Ok(config) = Config::builder()
            .add_source(File::with_name(&config_path).format(config::FileFormat::Toml))
            .set_default("ignore_dirs", Self::ignore_dirs())?
            .build()
        else {
            // TODO: Add logs
            eprintln!("Config doesn't exist. Using defaults.");
            return Ok(Self::default());
        };

        config.try_deserialize()
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ignore_dirs: Self::ignore_dirs(),
            default_dir: None,
        }
    }
}
