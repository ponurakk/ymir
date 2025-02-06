//! Config for ymir

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::bail;
use serde::Deserialize;

use crate::{cache::CacheSerializer, projects::Project};

/// Settings for ymir
#[derive(Debug, Deserialize)]
pub struct Settings {
    pub ignore_dirs: Vec<String>,
    pub default_dir: Option<PathBuf>,
}

fn pre_config() -> anyhow::Result<String> {
    let Some(config_dir) = dirs::config_dir() else {
        eprintln!("Failed to find config_directory");
        bail!("Failed to find config_directory")
    };

    let app_dir = format!("{}/{}", config_dir.display(), env!("CARGO_PKG_NAME"));

    if !Path::new(&app_dir).exists() {
        if let Err(err) = fs::create_dir_all(&app_dir) {
            eprintln!("Failed to create config directory: {err}");
            bail!("Failed to create config directory")
        }
    }

    Ok(app_dir)
}

impl Settings {
    /// Default ignore directories
    pub fn ignore_dirs<'a>() -> Vec<&'a str> {
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
    }

    /// Load config
    pub fn new() -> Self {
        let Some(config_dir) = dirs::config_dir() else {
            // TODO: Add notification
            eprintln!("Failed to find config_directory");
            return Self::default();
        };

        let config_path = format!(
            "{}/{}/config.toml",
            config_dir.display(),
            env!("CARGO_PKG_NAME")
        );

        if let Ok(file) = fs::read_to_string(&config_path) {
            return toml::from_str(&file).unwrap_or(Self::default());
        }

        Self::default()
    }

    pub fn write_config() -> anyhow::Result<()> {
        let default_config = Self::default();
        let serialized = format!(
            "ignore_dirs = {:?}\ndefault_dir = None",
            default_config.ignore_dirs
        );

        let Ok(app_dir) = pre_config() else {
            bail!("Failed to find config_dir");
        };

        let config_path = format!("{app_dir}/config.toml");

        if !Path::new(&config_path).exists() {
            if let Err(err) = fs::write(&config_path, serialized) {
                eprintln!("Failed to write config: {err}");
            } else {
                eprintln!("Default config saved to {config_path}");
            }
        }

        Ok(())
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ignore_dirs: Self::ignore_dirs()
                .iter()
                .map(|&v| (*v).to_string())
                .collect(),
            default_dir: None,
        }
    }
}

#[derive(Default, Debug)]
pub struct Cache {
    pub projects: Vec<Project>,
}

impl Cache {
    pub fn read_cache() -> Vec<Project> {
        let Some(config_dir) = dirs::config_dir() else {
            // TODO: Add notification
            eprintln!("Failed to find config_directory");
            return Vec::new();
        };

        let cache_path = format!("{}/{}/cache", config_dir.display(), env!("CARGO_PKG_NAME"));

        if let Ok(file) = fs::read(&cache_path) {
            let mut cursor = std::io::Cursor::new(file.as_slice());
            let cache: Cache = CacheSerializer::deserialize(&mut cursor).unwrap_or_default();
            return cache.projects;
        }

        eprintln!("Failed to find file");
        Vec::new()
    }

    pub fn create_cache(projects: &[Project]) -> anyhow::Result<Self> {
        let Ok(app_dir) = pre_config() else {
            bail!("Failed to find config_dir");
        };

        let config_path = format!("{app_dir}/cache");

        let cache = Cache {
            projects: projects.to_vec(),
        };

        let Ok(serialized) = CacheSerializer::serialize(&cache) else {
            bail!("Failed to serialize cache");
        };

        if !Path::new(&config_path).exists() {
            if let Err(err) = fs::write(&config_path, serialized) {
                eprintln!("Failed to write config: {err}");
            } else {
                eprintln!("Default config saved to {config_path}");
            }
        }

        Ok(cache)
    }
}
