pub mod app;
pub mod handlers;
pub mod ui;

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind};
use ratatui::DefaultTerminal;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let mut app = app::App::new();
    let mut last_tick = std::time::Instant::now();

    while !app.should_quit {
        terminal.draw(|frame| ui::draw(frame, &mut app))?;

        let interval = Duration::from_millis(app.interval_ms);
        let timeout = interval.saturating_sub(last_tick.elapsed());

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    handlers::handle_key(&mut app, key);
                }
            }
        }

        if last_tick.elapsed() >= interval {
            if !app.paused {
                app.refresh();
            }
            last_tick = std::time::Instant::now();
        }
    }

    Ok(())
}
