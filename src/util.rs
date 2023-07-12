use std::io::{self, Stdout};

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};

use ratatui::{backend::CrosstermBackend, Terminal};

pub fn setup_term() {
    let mut stdout = io::stdout();
    enable_raw_mode().unwrap();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
}

pub fn restore_term(terminal: &mut Terminal<CrosstermBackend<Stdout>>) {
    disable_raw_mode().unwrap();
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .unwrap();
    terminal.show_cursor().unwrap();
}
