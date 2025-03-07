pub mod config;
pub mod database;

use anyhow::Result;
use config::Config;
use database::{Database, Entry};
use iocraft::prelude::*;
use std::cmp::{max, min};

#[tokio::main]
async fn main() -> Result<()> {
    let config_path = Config::get_path();
    let config = Config::read(config_path)?;

    println!("{:#?}", config);

    element! {
        App(config: config)
    }
    .fullscreen()
    .await?;

    Ok(())
}

#[derive(Props, Default)]
struct AppProps {
    config: Config,
}

#[component]
fn App(mut hooks: Hooks, props: &AppProps) -> impl Into<AnyElement<'static>> {
    let (width, height) = hooks.use_terminal_size();
    let mut system = hooks.use_context_mut::<SystemContext>();
    let mut should_exit = hooks.use_state(|| false);

    let mut database = hooks.use_state(|| Database::new(props.config.thoughts_path.clone()));

    hooks.use_terminal_events({
        move |event| match event {
            TerminalEvent::Key(KeyEvent { code, kind, .. }) if kind != KeyEventKind::Release => {
                match code {
                    KeyCode::Esc => should_exit.set(true),
                    _ => {}
                }
            }
            _ => {}
        }
    });

    if should_exit.get() == true {
        system.exit();
    }

    let should_render = width >= props.config.min_width && height >= props.config.min_height;
    let show_note_content = width >= props.config.react_width;

    element! {
        View(){
            #(match should_render {
                true => element!{MainPage(term_width: width, term_height: height, show_note_content, database: database.read().clone())}.into_any(),
                false => element!{ResizeTermPage(term_width: width, term_height: height, min_width: props.config.min_width, min_height: props.config.min_height)}.into_any(),
            })
        }
    }
}

#[derive(Default, Props)]
struct MainPageProps {
    database: Database,
    show_note_content: bool,
    term_width: u16,
    term_height: u16,
}

#[component]
fn MainPage(mut hooks: Hooks, props: &MainPageProps) -> impl Into<AnyElement<'static>> {
    // Manually adjusting width of the containers isn't a great idea, but I can't
    // seem to get flexboxes to play nicely at the moment
    let note_list_width = match props.show_note_content {
        true => props.term_width / 2,
        false => props.term_width - 2,
    };

    let mut selected_entry: State<Option<usize>> = hooks.use_state(|| None);

    let entry_count = props.database.entries.len().clone();

    if entry_count > 0 && selected_entry.get() == None {
        selected_entry.set(Some(0));
    }

    if entry_count == 0 {
        selected_entry.set(None);
    }

    hooks.use_terminal_events({
        move |event| match event {
            TerminalEvent::Key(KeyEvent { code, kind, .. }) if kind != KeyEventKind::Release => {
                match code {
                    KeyCode::Up => match selected_entry.get() {
                        Some(index) => {
                            selected_entry.set(Some(max(index as i32 - 1, 0 as i32) as usize))
                        }
                        None => {}
                    },
                    KeyCode::Down => match selected_entry.get() {
                        Some(index) => selected_entry
                            .set(Some(min(index as i32 + 1, entry_count as i32 - 1) as usize)),
                        None => {}
                    },
                    _ => {}
                }
            }
            _ => {}
        }
    });

    element! {
        View(
            display: Display::Flex,
            height: props.term_height,
            width: props.term_width,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            padding_left: 1,
            padding_right: 1,
        ) {
            StatusBar()

            View(
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                height: 100pct,
                height: props.term_height,
                width: props.term_width - 2,
                ) {
                NoteList(width: note_list_width, entries: props.database.entries.clone(), selected_entry: selected_entry.read().clone())

                // Hide the content of a note if the terminal is smaller than
                // or equal to the react width set through the config
                #(match props.show_note_content {
                        true => element!{NoteContent()}.into_any(),
                        false => element!{View}.into_any(),
                })
            }

            SearchBar()
        }
    }
}

#[derive(Default, Props)]
struct ResizeTermPageProps {
    term_width: u16,
    term_height: u16,
    min_width: u16,
    min_height: u16,
}

#[component]
fn ResizeTermPage(props: &ResizeTermPageProps) -> impl Into<AnyElement<'static>> {
    let width_color = match props.term_width >= props.min_width {
        true => Color::Green,
        false => Color::Red,
    };

    let height_color = match props.term_height >= props.min_height {
        true => Color::Green,
        false => Color::Red,
    };

    element! {
        View(
            display: Display::Flex,
            height: props.term_height,
            width: props.term_width,
            padding_left: 1,
            padding_right: 1,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
        ) {
            View(padding_top: 1, padding_bottom: 1, padding_left: 4, padding_right: 4, border_style: BorderStyle::Single) {
                Text(content: "Terminal is too small!", weight: Weight::Bold)
            }

            View(padding_top: 1, padding_bottom: 1, padding_left: 4, padding_right: 4, border_style: BorderStyle::Single) {
                View(flex_direction: FlexDirection::Column) {
                    View() {
                        Text(content: "Current Dimensions:")
                        View(padding_left: 4) {
                            Text(content: format!("{}", props.term_width), color: width_color)
                            Text(content: "x", color: Color::DarkGrey)
                            Text(content: format!("{}", props.term_height),color: height_color)
                        }
                    }
                    View() {
                        Text(content: "Desired Dimensions:")
                        View(padding_left: 4) {
                            Text(content: props.min_width.to_string())
                            Text(content: "x", color: Color::DarkGrey)
                            Text(content: props.min_height.to_string())
                        }
                    }

                }
            }
        }
    }
}

#[component]
fn StatusBar() -> impl Into<AnyElement<'static>> {
    element! {
        View(
            height: 1,
            padding_left: 1,
            padding_right: 1,
            display: Display::Flex,
            justify_content: JustifyContent::SpaceBetween,
        ) {
            // This is a cheap hack. We are essentially "padding" the width of the 1st and
            // 3rd divs to be 19 units regardless of content, that way `justify-content: space-between`
            // will place the 2nd div exactly in the center of the two.
            //
            // This is less of an issue due to the 1st and 3rd divs being primarily static,
            // but it may create issues when resizing.
            //
            // TODO: In the future, set the width equal to the amount of characters in the
            // content field of the Text element in the 3rd, div. Eg:
            // "entry count: 144"   - width = 16
            // "entry count: 1092"  - width = 17
            View(width: 19) {
                Text(content: "Thoughts", weight: Weight::Bold, align: TextAlign::Left)
            }
            View() {
                Text(content: "Esc ", weight: Weight::Bold, align: TextAlign::Center, color: Color::DarkGrey, wrap: TextWrap::NoWrap)
                Text(content: "to exit", align: TextAlign::Center, color: Color::DarkGrey)
            }
            View(width: 19, justify_content: JustifyContent::End) {
                Text(content: "entry count: 144", align: TextAlign::Right, color: Color::DarkGrey)
            }
        }
    }
}

#[derive(Props, Default)]
struct NoteListProps {
    width: u16,
    entries: Vec<Entry>,
    selected_entry: Option<usize>,
}

#[component]
fn NoteList(props: &NoteListProps) -> impl Into<AnyElement<'static>> {
    element! {
        View(
            border_style: BorderStyle::Round,
            border_color: Color::White,
            flex_grow: 1.0,
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            max_width: props.width,
            // min_width: props.width,
        ) {
            #(props.entries.clone().into_iter().enumerate().map(|(index, entry)| element!{
                NoteListEntry(title: entry.title, width: props.width, is_favorite: entry.favorite, is_selected: Some(index) == props.selected_entry)
            }))
        }
    }
}

#[derive(Props, Default)]
struct NoteListEntryProps {
    width: u16,
    is_favorite: bool,
    modified_at: u64,
    title: String,
    is_selected: bool,
}

#[component]
fn NoteListEntry(props: &NoteListEntryProps) -> impl Into<AnyElement<'static>> {
    // Use to calculate where to truncate title
    let width_remaining = props.width - 25;
    let did_truncate = width_remaining < props.title.len() as u16;

    let mut title = props.title.clone();
    title.truncate(width_remaining.into());

    element! {
        View(width: props.width) {
            Text(content: if props.is_favorite {" ★ "} else {" ☆ "})
            Text(content: "06-06-2023 ", color: Color::Blue)
            Text(content: "11:40am ", color: Color::Green)
            Text(content: title, color: if props.is_selected { Color::Red } else { Color::White })
            Text(content: if did_truncate {"…"} else {""})
        }
    }
}

#[derive(Props, Default)]
struct NoteContentProps {
    width: u16,
}

#[component]
fn NoteContent(props: &NoteContentProps) -> impl Into<AnyElement<'static>> {
    element! {
        View(
            border_style: BorderStyle::Round,
            border_color: Color::White,
            flex_grow: 1.0,
            // max_width: props.width,
            // min_width: props.width,
        )
    }
}

#[component]
fn SearchBar(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    // TODO: Allow for text to scroll if it extends pass the bounds of what
    // can be displayed to the user
    let mut query = hooks.use_state(|| String::new());
    let mut cursor_position = hooks.use_state(|| 0);
    let query_cloned = query.to_string();
    // I unironically had to whiteboard the following logic, I feel so dumb.
    //
    // Brazinski's algorithm:
    // Assuming a stateful, append-only stream of alpha-numerical userinput, and a stateful
    // "cursor", how could one infer the correct position of the appended text.
    //
    // The solution is to track both the previous buffer, and the current buffer.
    //
    // Eg:
    //
    // Old State: ["h", "e", "l", "o"]
    // New State: ["h", "e", "l", "o", "l"]
    // Cursor Position: 3
    //
    // Length of Old State = 4
    // Length of New State = 5
    // Length difference = 5 - 4 = 1
    //
    // The length of the difference is 1, meaning we pop the most recently appended character,
    // and insert it at the position of the cursor.

    // // there is some text after the cursor if the position of the cursor is less
    // than the total length of the query
    let after_cursor = if cursor_position < query_cloned.len() {
        &query_cloned[cursor_position.get() + 1..query_cloned.len()]
    } else {
        ""
    };

    // there is some text before the cursor if the position of the cursor is greater than 0
    let before_cursor = if cursor_position.get() > 0 {
        &query_cloned[..cursor_position.get()]
    } else {
        ""
    };

    // there is some text under the cursor at the position of the cursor
    let during_cursor = match &query_cloned.get(cursor_position.get()..=cursor_position.get()) {
        Some(char) => *char,
        None => " ",
    };

    hooks.use_terminal_events({
        move |event| match event {
            TerminalEvent::Key(KeyEvent { code, kind, .. }) if kind != KeyEventKind::Release => {
                match code {
                    KeyCode::Left => {
                        if cursor_position.get() > 0 {
                            cursor_position.set(cursor_position - 1);
                        }
                    }
                    KeyCode::Right => {
                        if cursor_position.get() < query.to_string().len() {
                            cursor_position.set(cursor_position + 1);
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    });

    element! {
        View(
            height: 3,
            border_style: BorderStyle::Round,
            border_color: Color::White,
        ) {
            Text(content: "Search: ", wrap: TextWrap::NoWrap)
            View() {
                Text(content: *before_cursor)
                View(background_color: Color::White) {
                    Text(content: *during_cursor, color: Color::Black)
                }
                Text(content: *after_cursor)
            }

            View(width: 0) {
                TextInput(
                        has_focus: true,
                        value: query.to_string(),
                        on_change: move |new_value: String| {
                            let mut new_value = new_value.clone();
                            let mut old_value = query.to_string();

                            // We have to cast these to i32s, as we rely on significant negation
                            let length_difference = new_value.len() as i32 - old_value.len() as i32;

                            // if length_difference is positive, we've adding characters
                            if length_difference > 0 {
                                // This cast is safe, as the difference in length can
                                // never exceed the length of the array
                                let start = new_value.len() - length_difference as usize;

                                for (index, char) in new_value.clone()[start..].chars().enumerate() {
                                    new_value.pop();
                                    new_value.insert(cursor_position.get() + index, char);
                                }
                            }

                            // if length_difference is negative, we've removed characters
                            if length_difference < 0 {
                                // if we're pressing backspace at the start of the string, we
                                // shouldn't do anything
                                if cursor_position > 0 {
                                    for _ in 0..length_difference.abs() as usize {
                                        old_value.remove(cursor_position.get() - 1);
                                    }
                                }

                                new_value = old_value;
                            }

                            // offset the cursor by the amount of chars that were removed or added
                            let new_cursor_pos = max(cursor_position.get() as i32 + length_difference, 0) as usize;
                            cursor_position.set(new_cursor_pos);

                            query.set(new_value);
                        },

                )
            }
        }
    }
}
