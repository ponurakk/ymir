//! Ymir is a tool for finding projects
#![warn(missing_docs)]

mod app;
mod config;
mod projects;
mod utils;

use std::path::PathBuf;

use anyhow::bail;
use app::App;
use clap::Parser;
use config::Settings;

/// Arguments for cli
#[derive(Parser, Debug)]
struct Args {
    name: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let settings = Settings::new()?;
    let args = Args::parse();

    let Some(find_dir) = args.name.or(settings.default_dir) else {
        bail!("You must specify the directory");
    };

    let projects = projects::find(find_dir, &settings.ignore_dirs);

    let terminal = ratatui::init();
    let app_result = App::new(projects).run(terminal);
    ratatui::restore();
    app_result
}
