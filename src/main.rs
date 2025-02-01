mod config;
mod projects;

use std::path::PathBuf;

use config::Settings;

fn main() -> anyhow::Result<()> {
    let settings = Settings::new()?;

    let home_dir = format!(
        "{}/Scripts",
        dirs::home_dir().expect("Couldn't find home dir").display()
    );

    let projects = projects::find(PathBuf::from(home_dir), &settings.ignore_dirs);
    println!("{projects:#?}");
    println!("Found: {}", projects.len());
    Ok(())
}

// fn main() -> anyhow::Result<()> {
//     let terminal = ratatui::init();
//     let result = run(terminal);
//     ratatui::restore();
//     result
// }
//
// fn run(mut terminal: DefaultTerminal) -> anyhow::Result<()> {
//     loop {
//         terminal.draw(render)?;
//         if matches!(event::read()?, Event::Key(_)) {
//             break Ok(());
//         }
//     }
// }
//
// fn render(frame: &mut Frame) {
//     frame.render_widget("hello world", frame.area());
// }
