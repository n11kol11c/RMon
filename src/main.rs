pub mod app;
pub mod config;
pub mod handlers;
pub mod theme;
pub mod ui;

use std::io::{self, Write};
use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use ratatui::{DefaultTerminal, backend::CrosstermBackend, Terminal};

fn main() -> io::Result<()> {
    install_panic_hook();
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let result = run(&mut terminal);

    terminal.show_cursor()?;
    execute!(terminal.backend_mut(), DisableMouseCapture, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    result
}

fn install_panic_hook() {
    let original = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = execute!(
            std::io::stdout(),
            DisableMouseCapture,
            LeaveAlternateScreen
        );
        let _ = disable_raw_mode();
        let _ = std::io::stdout().flush();
        original(info);
    }));
}

fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let config = config::Config::load();
    let mut app = app::App::with_config(config);
    let mut last_tick = std::time::Instant::now();

    while !app.should_quit {
        terminal.draw(|frame| ui::draw(frame, &mut app))?;

        let interval = Duration::from_millis(app.interval_ms);
        let timeout = interval.saturating_sub(last_tick.elapsed());

        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    handlers::handle_key(&mut app, key)
                }
                Event::Mouse(mouse) => handlers::handle_mouse(&mut app, mouse),
                _ => {}
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
