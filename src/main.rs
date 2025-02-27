use anyhow::Result;
use iocraft::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    element! {
        App()
    }
    .fullscreen()
    .await?;

    Ok(())
}

#[component]
fn App(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let (width, height) = hooks.use_terminal_size();
    let mut system = hooks.use_context_mut::<SystemContext>();
    let mut should_exit = hooks.use_state(|| false);

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

    let should_render = width >= 58 && height >= 18;
    let show_note_content = width >= 80;

    element! {
        View(){
            #(match should_render {
                true => element!{MainPage(term_width: width, term_height: height, show_note_content)}.into_any(),
                false => element!{ResizeTermPage(term_width: width, term_height: height)}.into_any(),
            })
        }
    }
}

#[derive(Default, Props)]
struct MainPageProps {
    show_note_content: bool,
    term_width: u16,
    term_height: u16,
}

#[component]
fn MainPage(props: &MainPageProps) -> impl Into<AnyElement<'static>> {
    element! {
        View(
            display: Display::Flex,
            height: props.term_height,
            width: props.term_width,
            flex_direction: FlexDirection::Column,
            padding_left: 1,
            padding_right: 1,
        ) {
            StatusBar()

            View(
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                height: 100pct,
                ) {
                NoteList()

                // Hide the content of a note if the terminal is smaller than
                // or equal to 80 characters wide
                #(match props.show_note_content {
                        true => element!{NoteContent}.into_any(),
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
}

#[component]
fn ResizeTermPage(props: &ResizeTermPageProps) -> impl Into<AnyElement<'static>> {
    let width_color = match props.term_width >= 58 {
        true => Color::Green,
        false => Color::Red,
    };

    let height_color = match props.term_height >= 18 {
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
                            Text(content: "58")
                            Text(content: "x", color: Color::DarkGrey)
                            Text(content: "18")
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
            // 3rd divs to be 19 regardless of content, that way `justify-content: space-between`
            // will place the 2nd div exactly in the center of the two
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

#[component]
fn NoteList() -> impl Into<AnyElement<'static>> {
    element! {
        View(
            border_style: BorderStyle::Round,
            border_color: Color::White,
            flex_grow: 1.0,
        )
    }
}

#[component]
fn NoteContent() -> impl Into<AnyElement<'static>> {
    element! {
        View(
            border_style: BorderStyle::Round,
            border_color: Color::White,
            flex_grow: 1.0,
        )
    }
}

#[component]
fn SearchBar(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut query = hooks.use_state(|| String::new());

    element! {
        View(
            height: 3,
            border_style: BorderStyle::Round,
            border_color: Color::White,
        ) {
            Text(content: "Search: ", wrap: TextWrap::NoWrap)
            // Unbelievable hack, but it must be done
            // I don't see any way to set the cursor character,
            // so the solution that I've come up with is to create a "back buffer"
            // that handles polling input, followed by a "front buffer" that simply
            // displays the content of the "back buffer" with a cursor char at the end.
            //
            // I can't simply use a Text() element for the front buffer,
            // as it lacks the scroll-on-overflow effect, and I can't add
            // the cursor char to the end of back buffer, as that would break
            // the use_state hook.

            TextInput(value: format!("{}▌", query.to_string()))
            View(width: 0) {
                TextInput(
                        has_focus: true,
                        value: query.to_string(),
                        on_change: move |new_value| query.set(new_value),
                )
            }
        }
    }
}
