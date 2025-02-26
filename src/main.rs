use anyhow::{Result, anyhow};
use crossterm::event::{self, Event};
use ratatui::{DefaultTerminal, Frame, layout::Rect, widgets::Paragraph};

fn main() -> Result<()> {
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    loop {
        terminal.draw(render)?;

        if matches!(event::read()?, Event::Key(_)) {
            break Ok(());
        }
    }
}

fn render(frame: &mut Frame) {
    frame.render_widget("hello world", frame.area());

    frame.render_widget(
        Paragraph::new("hello world, but lower this time"),
        Rect::new(10, 10, 50, 50),
    );
}
