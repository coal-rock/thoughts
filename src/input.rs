use crate::tui::App;
use crate::tui::Focused;
use crate::tui::InputState;

use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::backend::Backend;
use ratatui::Terminal;

// I HATE NESTING
pub fn handle_input<B: Backend>(
    code: &KeyCode,
    modifiers: &KeyModifiers,
    app: &mut App,
    terminal: &mut Terminal<B>,
) -> bool {
    match app.focused {
        Focused::MainWindow => return handle_main_window(code, modifiers, app, terminal),
        Focused::DeletionPrompt => return handle_delete_window(code, app),
    }
}

fn handle_main_window<B: Backend>(
    code: &KeyCode,
    modifiers: &KeyModifiers,
    app: &mut App,
    terminal: &mut Terminal<B>,
) -> bool {
    if modifiers == &event::KeyModifiers::CONTROL {
        match app.input_state {
            InputState::Search => match code {
                event::KeyCode::Char(c) => match c {
                    'f' => app.toggle_favorite_selected_entry(),
                    'e' => app.edit_entry(terminal),
                    'r' => app.enter_rename_mode(),
                    _ => {}
                },
                _ => {}
            },
            InputState::Rename => {}
        }
    } else {
        match app.input_state {
            InputState::Search => match code {
                event::KeyCode::Enter => app.create_entry(terminal),
                event::KeyCode::Esc => return false,
                event::KeyCode::Down => app.entries.next(),
                event::KeyCode::Up => app.entries.previous(),
                event::KeyCode::Delete => app.delete_selected_entry(),
                _ => {}
            },
            InputState::Rename => match code {
                event::KeyCode::Esc => app.exit_rename_mode(),
                event::KeyCode::Enter => app.rename_entry(),
                _ => {}
            },
        }

        match code {
            event::KeyCode::Char(c) => app.insert_query_char(*c),
            event::KeyCode::Backspace => app.delete_query_char(),
            event::KeyCode::Left => app.prev_query_char(),
            event::KeyCode::Right => app.next_query_char(),
            _ => {}
        }
    }

    match app.input_state {
        InputState::Search => app.update_entries(),
        _ => {}
    }

    true
}

fn handle_delete_window(code: &KeyCode, app: &mut App) -> bool {
    match code {
        event::KeyCode::Char(c) => match c.to_string().as_str() {
            "y" => app.delete_selected_entry(),
            "n" => app.focused = Focused::MainWindow,
            _ => {}
        },
        _ => {}
    }

    true
}
