//! Ymir is a tool for finding projects
#![warn(missing_docs)]

mod app;
mod cache;
mod config;
mod projects;
mod sorting;
mod utils;

use std::path::PathBuf;

use anyhow::bail;
use app::App;
use clap::Parser;
use config::{Cache, Settings};

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
        eprintln!("No cache");
        projects::find(&find_dir, &settings.ignore_dirs)
    } else if args.fresh {
        eprintln!("Fresh");
        Cache::create_cache(&projects::find(&find_dir, &settings.ignore_dirs))?.projects
    } else {
        eprintln!("From Cache");
        let cache = Cache::read_cache();
        if cache.is_empty() {
            Cache::create_cache(&projects::find(&find_dir, &settings.ignore_dirs))?.projects
        } else {
            cache
        }
    };

    let terminal = ratatui::init();
    let app_result = App::new(projects).run(terminal);
    ratatui::restore();
    app_result
}
