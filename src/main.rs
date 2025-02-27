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
            Text(content: "Thoughts")
            Text(content: "Esc to exit")
            Text(content: "entry count: 144")
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
    element! {
        View(
            height: 3,
            border_style: BorderStyle::Round,
            border_color: Color::White,
        ) {
            Text(content: "Search: ")
        }
    }
}
