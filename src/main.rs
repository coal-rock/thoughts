pub mod cli;
pub mod config;
pub mod database;
pub mod editor;
pub mod filesystem;
pub mod input;
pub mod search;
pub mod sql;
pub mod tui;
pub mod util;

use clap::Parser;
use cli::{Cli, Commands};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use tui::{run_app, App};
use util::{restore_term, setup_term};

fn main() {
    // The day I gave up home on crossplat
    let config = config::load_config("~/.config/thoughts/config.toml".to_string()).unwrap();

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    let cli = Cli::parse();
    let mut app = App::new(config.clone());

    match &cli.command {
        Some(Commands::New { name }) => {
            app.database
                .insert_entry(editor::new(name.clone(), &config.editor_command).into());
            return;
        }
        Some(Commands::Backup) => {
            app.database.backup(&config.backup_path);
            println!("Backup taken at: {}", config.backup_path.to_str().unwrap());
            return;
        }
        Some(Commands::Search { query }) => {
            let query = query.clone().unwrap_or("".to_string());
            let entries = app.database.search(query);

            for entry in entries {
                println!("{} {} {}", entry.get_date(), entry.get_time(), entry.title);
            }

            return;
        }
        None => {}
    }

    setup_term();

    run_app(&mut terminal, app);

    editor::purge_entry_files(&config);

    restore_term(&mut terminal);
}
