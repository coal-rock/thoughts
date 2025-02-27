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
