//! Ymir is a tool for finding projects
#![warn(missing_docs)]

#[macro_use]
extern crate log;

mod app;
mod cache;
mod config;
mod huffman;
mod projects;
mod sorting;
mod utils;

use std::{fs::File, path::PathBuf};

use anyhow::bail;
use app::App;
use clap::Parser;
use config::{Cache, Settings};
use log::LevelFilter;
use simplelog::ConfigBuilder;

/// Arguments for cli
#[derive(Parser, Debug)]
struct Args {
    /// Path from where to search
    path: Option<PathBuf>,

    /// Saves config in config directory
    #[arg(long)]
    gen_config: bool,

    /// Don't create cache file
    #[arg(long)]
    no_cache: bool,

    /// Update of cache
    #[arg(long)]
    fresh: bool,
}

fn main() -> anyhow::Result<()> {
    let Some(config_dir) = dirs::config_dir() else {
        bail!("Failed to find config_directory")
    };

    let log_path = config_dir
        .join(env!("CARGO_PKG_NAME"))
        .join(format!("{}.log", env!("CARGO_PKG_NAME")));

    simplelog::WriteLogger::init(
        LevelFilter::Info,
        ConfigBuilder::new().add_filter_ignore_str("tokei").build(),
        File::create(log_path).unwrap(),
    )?;

    let args = Args::parse();

    if args.gen_config {
        Settings::write_config()?;
        return Ok(());
    }

    let settings = Settings::new();
    let Some(find_dir) = args.path.or(settings.default_dir) else {
        bail!("You must specify the directory");
    };

    let projects = if args.no_cache {
        eprintln!("Loading fresh data");
        debug!("Loading fresh data");
        projects::find(&find_dir, &settings.ignore_dirs)
    } else if args.fresh {
        eprintln!("Refreshing cache");
        debug!("Refreshing cache");
        Cache::create_cache(&projects::find(&find_dir, &settings.ignore_dirs))
            .unwrap_or_default()
            .projects
    } else {
        eprintln!("Loading data from cache");
        debug!("Loading data from cache");
        let cache = Cache::read_cache();
        if cache.is_empty() {
            Cache::create_cache(&projects::find(&find_dir, &settings.ignore_dirs))
                .unwrap_or_default()
                .projects
        } else {
            cache
        }
    };

    let terminal = ratatui::init();
    let app_result = App::new(projects).run(terminal);
    ratatui::restore();
    app_result
}
