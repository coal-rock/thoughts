use anyhow::Result;
use iocraft::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    element! {
        MainPage()
    }
    .fullscreen()
    .await?;

    Ok(())
}

#[component]
fn MainPage(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let (width, height) = hooks.use_terminal_size();

    element! {
        View(
            display: Display::Flex,
            height: height,
            width: width,
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
                NoteContent()
            }

            SearchBar()
        }
    }
}

#[component]
fn StatusBar(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    element! {
        View(
            height: 1,
            padding_left: 1,
            padding_right: 1,
            display: Display::Flex,
            justify_content: JustifyContent::SpaceBetween,
        ) {
            // This is a cheap hack. We are essentially "padding" the width of the 1st and
            // 3rd divs to be 25 regardless of content, that way `justify-content: space-between`
            // will place the 2nd div exactly in the center of the two
            View(width: 25) {
                Text(content: "Thoughts", weight: Weight::Bold, align: TextAlign::Left)
            }
            View() {
                Text(content: "Esc ", weight: Weight::Bold, align: TextAlign::Center, color: Color::DarkGrey, wrap: TextWrap::NoWrap)
                Text(content: "to exit", align: TextAlign::Center, color: Color::DarkGrey)
            }
            View(width: 25, justify_content: JustifyContent::End) {
                Text(content: "entry count: 144", align: TextAlign::Right, color: Color::DarkGrey)
            }
        }
    }
}

#[component]
fn NoteList(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    element! {
        View(
            border_style: BorderStyle::Round,
            border_color: Color::White,
            flex_grow: 1.0,
        )
    }
}

#[component]
fn NoteContent(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
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
