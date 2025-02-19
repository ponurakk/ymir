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

use std::{env, fs::File, path::PathBuf};

use anyhow::bail;
use app::App;
use config::{Cache, Settings};
use getopts::Options;
use log::LevelFilter;
use simplelog::ConfigBuilder;

fn print_usage(opts: &Options) {
    let brief = format!("Usage: {} [PATH] [OPTIONS]", env!("CARGO_PKG_NAME"));
    print!("{}", opts.usage(&brief));
}

fn main() -> anyhow::Result<()> {
    let Some(config_dir) = dirs::config_dir() else {
        bail!("Failed to find config_directory")
    };

    let log_path = config_dir
        .join(env!("CARGO_PKG_NAME"))
        .join(format!("{}.log", env!("CARGO_PKG_NAME")));

    let Ok(log_file) = File::create(log_path) else {
        bail!("Failed to create log file");
    };

    simplelog::WriteLogger::init(
        LevelFilter::Info,
        ConfigBuilder::new().add_filter_ignore_str("tokei").build(),
        log_file,
    )?;

    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optflag("", "gen-config", "Saves config in config directory");
    opts.optflag("", "no-cache", "Don't create cache file");
    opts.optflag("f", "fresh", "Recreate cache file from scratch");
    opts.optflag("h", "help", "Print help");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => bail!("{}", f),
    };

    if matches.opt_present("h") {
        print_usage(&opts);
        return Ok(());
    }

    if matches.opt_present("gen-config") {
        Settings::write_config()?;
        return Ok(());
    }

    let path = matches.free.first().map(PathBuf::from);
    let settings = Settings::new();

    let Some(find_dir) = path.or(settings.default_dir) else {
        bail!("You must specify the directory");
    };

    let projects = if matches.opt_present("no-cache") {
        eprintln!("Loading fresh data");
        debug!("Loading fresh data");
        projects::find(&find_dir, &settings.ignore_dirs)
    } else if matches.opt_present("fresh") {
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
