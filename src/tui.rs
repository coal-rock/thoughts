use crate::config::Config;
use crate::database::{Database, EntryQuery};
use crate::editor;
use crate::input::handle_input;
use crate::sql::SqlDatabase;

use std::cmp;

use chrono::NaiveDateTime;

use crossterm::event::{self, Event};

use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};

pub enum Focused {
    MainWindow,
    DeletionPrompt,
}

pub enum InputState {
    Search,
    Rename,
}

pub struct App {
    pub config: Config,
    pub input: Vec<char>,
    pub input_backup: Vec<char>,
    pub input_index: i32,
    pub input_index_backup: i32,
    pub input_state: InputState,
    pub entries: StatefulList<(Entry, usize)>,
    pub database: Box<dyn Database>,
    pub focused: Focused,
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub title: String,
    pub dt: NaiveDateTime,
    pub content: String,
    pub id: i32,
    pub favorite: bool,
}

impl Entry {
    pub fn get_date(&self) -> String {
        format!("{}", self.dt.format("%m-%d-%Y"))
    }

    pub fn get_time(&self) -> String {
        format!("{}", self.dt.format("%I:%M%P"))
    }
}

// SQL HACK
impl From<EntryQuery> for Entry {
    fn from(query: EntryQuery) -> Entry {
        Entry {
            title: query.title,
            dt: NaiveDateTime::from_timestamp_opt(query.timestamp.parse().unwrap(), 0).unwrap(),
            content: query.content,
            id: query.id,
            favorite: query.favorite,
        }
    }
}

impl From<Entry> for EntryQuery {
    fn from(entry: Entry) -> EntryQuery {
        EntryQuery {
            id: entry.id,
            title: entry.title,
            timestamp: entry.dt.timestamp().to_string(),
            content: entry.content,
            favorite: entry.favorite,
        }
    }
}

impl App {
    pub fn new(config: Config) -> App {
        let database = match config.db_type {
            crate::config::DbType::Sql => Box::new(SqlDatabase::new(&config)),
            crate::config::DbType::Fs => todo!(),
        };

        return App {
            config,
            input: Vec::new(),
            input_backup: Vec::new(),
            input_index: 0,
            input_index_backup: 0,
            input_state: InputState::Search,
            entries: StatefulList::with_items(vec![]),
            database,
            focused: Focused::MainWindow,
        };
    }

    fn get_query_string(&self) -> String {
        self.input
            .clone()
            .into_iter()
            .map(|x| x.to_string())
            .collect()
    }

    // FIXME: this introduces a bug with toggling favorite
    // when the length of a list exceeds that of a screen height
    pub fn update_entries(&mut self) {
        let pos = self.entries.state.selected();

        let entries: Vec<(Entry, usize)> = self
            .database
            .search(self.get_query_string())
            .into_iter()
            .enumerate()
            .map(|(i, e)| (e, i))
            .collect();

        self.entries = StatefulList::with_items(entries);
        self.entries.last();

        self.entries.state.select(pos);
    }

    pub fn insert_query_char(&mut self, char: char) {
        self.input.insert(self.input_index as usize, char);
        self.input_index += 1;
    }

    pub fn delete_query_char(&mut self) {
        if self.input_index > 0 {
            self.input.remove((self.input_index - 1) as usize);
        } else {
            self.input = Vec::new();
        }
        self.input_index = cmp::max(self.input_index - 1, 0);
    }

    // Unsure how to feel about this implementation
    // feels a bit odd, but whatever
    pub fn delete_selected_entry(&mut self) {
        match self.focused {
            Focused::MainWindow => {
                if self.entries.get_selected().is_some() {
                    self.focused = Focused::DeletionPrompt;
                }
            }
            Focused::DeletionPrompt => {
                if let Some(entry) = self.entries.get_selected() {
                    self.focused = Focused::MainWindow;
                    self.database.delete_entry(entry.0.id);
                    self.update_entries();
                }
            }
        }
    }

    pub fn next_query_char(&mut self) {
        self.input_index = cmp::min(self.input.len() as i32, self.input_index + 1);
    }

    pub fn prev_query_char(&mut self) {
        self.input_index = cmp::max(self.input_index - 1, 0);
    }

    pub fn toggle_favorite_selected_entry(&mut self) {
        let query = if let Some(entry) = self.entries.get_selected() {
            EntryQuery::from(entry.0.clone())
        } else {
            return;
        };

        self.database.update_entry(EntryQuery {
            favorite: !query.favorite,
            ..query
        });

        // Patch UI if state: `favorite` isn't relevant to search
        // if search::tokenize_search(self.get_query_string())
        //     .tokens
        //     .contains(&Token {
        //         content: search::SearchToken::Favorite,
        //     })
        // {
        //     self.update_entries();
        // } else {
        //     let selected = self.entries.state.selected().unwrap();
        //     self.entries.items[selected].0.favorite = !self.entries.items[selected].0.favorite;
        // }

        self.update_entries();
    }

    pub fn create_entry<B: Backend>(&mut self, terminal: &mut Terminal<B>) {
        let name = self.get_query_string();

        if name.replace(" ", "").len() == 0 {
            return;
        }

        let entry = editor::new(name, &self.config.editor_command);

        self.database.insert_entry(entry.into());

        terminal.clear().unwrap();
        self.update_entries();
    }

    pub fn edit_entry<B: Backend>(&mut self, terminal: &mut Terminal<B>) {
        if let Some(entry) = self.entries.get_selected() {
            let entry = editor::edit(entry.0.clone(), &self.config.editor_command);

            self.database.update_entry(entry.into());

            terminal.clear().unwrap();
            self.update_entries();
        }
    }

    pub fn enter_rename_mode(&mut self) {
        if let Some(entry) = self.entries.get_selected() {
            self.input_backup = self.input.clone();
            self.input_index_backup = self.input_index;

            self.input = entry.0.title.chars().into_iter().collect();
            self.input_index = self.input.len() as i32;
            self.input_state = InputState::Rename;
        }
    }

    pub fn exit_rename_mode(&mut self) {
        self.input = self.input_backup.clone();
        self.input_index = self.input_index_backup;
        self.input_state = InputState::Search;
    }

    pub fn rename_entry(&mut self) {
        if let Some(entry) = self.entries.get_selected() {
            let mut entry = entry.0.clone();
            entry.title = self.get_query_string();
            self.database.update_entry(entry.into());
            self.exit_rename_mode();
        }
    }
}

pub struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    cmp::max(self.items.len() as i32 - 1, 0) as usize
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn last(&mut self) {
        let i: Option<usize> = if self.items.len() != 0 {
            Some(self.items.len() - 1)
        } else {
            Some(0)
        };
        self.state.select(i);
    }

    fn get_selected(&self) -> Option<&T> {
        if self.items.is_empty() {
            return None;
        }

        if let Some(item) = self.state.selected() {
            self.items.get(item)
        } else {
            None
        }
    }
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) {
    app.update_entries();
    app.entries.last();

    // TODO:
    // input handling needs to be massively reworked
    loop {
        terminal.draw(|f| ui(f, &mut app)).unwrap();

        if let Event::Key(event::KeyEvent {
            code, modifiers, ..
        }) = event::read().unwrap()
        {
            if !handle_input(&code, &modifiers, &mut app, terminal) {
                break;
            };
        }
    }

    terminal.clear().unwrap();
}

fn entries_pane<'a>(items: &Vec<(Entry, usize)>, selected: usize) -> List<'a> {
    let entries: Vec<ListItem> = items
        .iter()
        .map(|x| {
            let title_style = if selected == x.1 {
                Style::default().add_modifier(Modifier::BOLD).fg(Color::Red)
            } else {
                Style::default()
            };

            let favorited = if x.0.favorite { " ★ " } else { " ☆ " };

            let content = Spans::from(vec![
                Span::from(favorited),
                Span::styled(x.0.get_date(), Style::default().fg(Color::LightBlue)),
                Span::from(" "),
                Span::styled(x.0.get_time(), Style::default().fg(Color::LightGreen)),
                Span::from(" "),
                Span::styled(x.0.title.clone(), title_style),
            ]);

            ListItem::new(content).style(Style::default())
        })
        .collect();

    List::new(entries).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    )
}

fn preview_pane(text: &String) -> Paragraph {
    Paragraph::new(Text::from(text.to_string()))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .wrap(Wrap { trim: { true } })
}

fn title<'a>() -> Paragraph<'a> {
    Paragraph::new(Text::from(Span::styled(
        " Thoughts",
        Style::default().add_modifier(Modifier::BOLD),
    )))
    .alignment(Alignment::Left)
}

fn exit_prompt<'a>() -> Paragraph<'a> {
    Paragraph::new(Text::from(Spans::from(vec![
        Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to exit"),
    ])))
    .style(Style::default().fg(Color::DarkGray))
    .alignment(Alignment::Center)
}

fn entry_count<'a>(entry_count: i32) -> Paragraph<'a> {
    Paragraph::new(Text::from(Span::styled(
        format!("entry count: {} ", entry_count),
        Style::default().fg(Color::DarkGray),
    )))
    .alignment(Alignment::Right)
}

fn input_box<'a>(input_state: &InputState, input_string: &str) -> Paragraph<'a> {
    let mode = match input_state {
        InputState::Search => "Search:",
        InputState::Rename => "Rename:",
    };

    Paragraph::new(format!("{} {}", mode, input_string)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    )
}

fn confirm_delete<'a>(window_size: &Rect) -> (Paragraph<'a>, Rect) {
    let paragraph = Paragraph::new(Text::from("y / n"))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title("Confirm Delete")
                .borders(Borders::ALL)
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded),
        )
        .alignment(Alignment::Center)
        .style(Style::default().add_modifier(Modifier::ITALIC));

    (paragraph, centered_rect(15, 15, *window_size))
}

// consumes a rect, and returns another rect centered within in
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

// TODO: break each chunk into it's own function
fn ui<B: Backend>(frame: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .horizontal_margin(1)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(frame.size());

    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
            ]
            .as_ref(),
        )
        .split(chunks[0]);

    frame.render_widget(title(), header_chunks[0]);
    frame.render_widget(exit_prompt(), header_chunks[1]);
    frame.render_widget(
        entry_count(app.database.get_entry_count()),
        header_chunks[2],
    );

    let query = input_box(&app.input_state, &app.get_query_string());

    // FIXME: for fucks sake please fix this
    let selected = if app.entries.get_selected().is_some() {
        app.entries.get_selected().unwrap().1
    } else {
        usize::MAX
    };

    let entries_pane = entries_pane(&app.entries.items, selected);

    if frame.size().width > 80 {
        let body_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)].as_ref())
            .split(chunks[1]);

        frame.render_stateful_widget(entries_pane, body_chunks[0], &mut app.entries.state);

        let preview_text = match app.entries.get_selected() {
            Some(entry) => entry.0.content.clone(),
            None => "".to_string(),
        };

        let preview_pane = preview_pane(&preview_text);
        frame.render_widget(preview_pane, body_chunks[1]);
    } else {
        frame.render_stateful_widget(entries_pane, chunks[1], &mut app.entries.state);
    }

    frame.render_widget(query, chunks[2]);

    match app.focused {
        Focused::MainWindow => {}
        Focused::DeletionPrompt => {
            let (block, area) = confirm_delete(&frame.size());
            frame.render_widget(Clear, area);
            frame.render_widget(block, area);
        }
    }

    // This is hardcoded and bad.
    frame.set_cursor(
        chunks[2].x + app.input_index as u16 + 9 as u16,
        chunks[2].y + 1,
    );
}
