use anyhow::Result;
use iocraft::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    element! {
        View(
            border_style: BorderStyle::Round,
            border_color: Color::Blue,
            width: Size::Length(100),
            height: Size::Length(20),
            justify_content: JustifyContent::Center,
        ) {
            View() {
                View(
                    border_style: BorderStyle::Round,
                    border_color: Color::Red,
                    justify_content: JustifyContent::Center,

                ) {
                        Text(content: "Hello, world", align: TextAlign::Right, wrap: TextWrap::NoWrap)
                }
            }
        }
    }
    .fullscreen()
    .await?;

    Ok(())
}
