use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Position;

use crate::app::App;

pub fn handle_key(app: &mut App, key: KeyEvent) {
    if app.confirm.is_some() {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => app.confirm_kill(),
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => app.cancel_kill(),
            KeyCode::Char('q') => app.should_quit = true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.should_quit = true
            }
            _ => {}
        }
        return;
    }

    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true
        }
        KeyCode::Up | KeyCode::Char('k') => app.move_selection(-1),
        KeyCode::Down | KeyCode::Char('j') => app.move_selection(1),
        KeyCode::PageUp => app.move_selection(-10),
        KeyCode::PageDown => app.move_selection(10),
        KeyCode::Home => app.jump_to_first(),
        KeyCode::End => app.jump_to_last(),
        KeyCode::Char('d') | KeyCode::Delete => app.request_kill(),
        KeyCode::Char('p') => app.paused = !app.paused,
        KeyCode::Char('s') => app.cycle_sort(),
        KeyCode::Char('r') => app.refresh(),
        KeyCode::Char('+') | KeyCode::Char(']') => app.speed_up(),
        KeyCode::Char('-') | KeyCode::Char('[') => app.speed_down(),
        _ => {}
    }
}

pub fn handle_mouse(app: &mut App, mouse: MouseEvent) {
    if app.confirm.is_some() {
        return;
    }
    match mouse.kind {
        MouseEventKind::ScrollUp => app.move_selection(-1),
        MouseEventKind::ScrollDown => app.move_selection(1),
        MouseEventKind::Down(MouseButton::Left) => {
            if let Some(area) = app.table_area {
                let pos = Position { x: mouse.column, y: mouse.row };
                let header_end = area.y.saturating_add(2);
                if area.contains(pos) && mouse.row >= header_end {
                    let max = app.processes.len().saturating_sub(1);
                    let idx = mouse.row - header_end;
                    app.process_state.select(Some(idx.min(max as u16) as usize));
                }
            }
        }
        _ => {}
    }
}
