use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, LineGauge, Paragraph, Row, Sparkline, Table,
        block::Title,
    },
};
use sysinfo::System;

use crate::app::App;

const BG: Color = Color::Rgb(18, 22, 32);
const PANEL: Color = Color::Rgb(24, 29, 42);
const ACCENT: Color = Color::Rgb(88, 176, 255);
const TITLE: Color = Color::Rgb(121, 216, 255);
const TEXT: Color = Color::Rgb(214, 221, 232);
const DIM: Color = Color::Rgb(112, 122, 140);
const OK: Color = Color::Rgb(92, 220, 152);
const WARN: Color = Color::Rgb(255, 199, 92);
const DANGER: Color = Color::Rgb(255, 94, 104);

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    if area.width < 8 || area.height < 5 {
        return;
    }
    frame.render_widget(Paragraph::new("").style(Style::new().bg(BG)), area);

    let chunks = Layout::vertical([
        Constraint::Length(5),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(area);

    draw_header(frame, app, chunks[0]);
    draw_main(frame, app, chunks[1]);
    draw_footer(frame, app, chunks[2]);

    if app.confirm.is_some() {
        draw_confirm(frame, app, area);
    }
}

fn panel(title: &str) -> Block<'static> {
    Block::new()
        .title(Title::from(Span::styled(
            format!(" {title} "),
            Style::new().fg(TITLE).add_modifier(Modifier::BOLD),
        )))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(ACCENT))
        .style(Style::new().bg(PANEL))
}

fn inner_area(area: Rect) -> Rect {
    Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    }
}

fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let left = Line::from(vec![
        Span::styled("▣ ", Style::new().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Span::styled(
            "RMon",
            Style::new().fg(TITLE).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  v{}", env!("CARGO_PKG_VERSION")),
            Style::new().fg(DIM),
        ),
    ]);

    let right = Line::from(vec![
        Span::styled("● ", Style::new().fg(if app.paused { WARN } else { OK })),
        Span::styled(now, Style::new().fg(DIM)),
    ]);
    let os = System::long_os_version().unwrap_or_else(|| "Unknown OS".to_string());
    let host = System::host_name().unwrap_or_else(|| "unknown".to_string());
    let arch = System::cpu_arch().unwrap_or_else(|| "?".to_string());

    let line1 = Line::from(vec![
        Span::styled(host, Style::new().fg(TEXT).add_modifier(Modifier::BOLD)),
        Span::styled("  ·  ", Style::new().fg(DIM)),
        Span::styled(os, Style::new().fg(TEXT)),
    ]);

    let line2 = Line::from(vec![
        Span::styled("CPU: ", Style::new().fg(DIM)),
        Span::styled(format!("{}× {}", app.system.cpus().len(), arch), Style::new().fg(TEXT)),
        Span::styled("   Uptime: ", Style::new().fg(DIM)),
        Span::styled(format_uptime(System::uptime()), Style::new().fg(TEXT)),
        Span::styled("   Processes: ", Style::new().fg(DIM)),
        Span::styled(format!("{}", app.process_count), Style::new().fg(TEXT)),
    ]);

    let block = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(ACCENT))
        .title(Title::from(left))
        .title(Title {
            content: right,
            alignment: Some(Alignment::Right),
            position: None,
        })
        .style(Style::new().bg(PANEL));

    frame.render_widget(
        Paragraph::new(Text::from(vec![line1, Line::from(" "), line2]))
            .block(block)
            .style(Style::new().fg(TEXT)),
        area,
    );
}

fn draw_main(frame: &mut Frame, app: &mut App, area: Rect) {
    let avail = area.height.saturating_sub(14);
    let disk_h = (app.disk_info.len() as u16 * 2 + 4).min(avail);

    let chunks = Layout::vertical([
        Constraint::Length(10),
        Constraint::Length(disk_h),
        Constraint::Min(4),
    ])
    .split(area);

    let row = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    draw_cpu(frame, app, row[0]);
    draw_memory(frame, app, row[1]);
    draw_disks(frame, app, chunks[1]);
    draw_processes(frame, app, chunks[2]);
}

fn draw_cpu(frame: &mut Frame, app: &App, area: Rect) {
    if area.height < 2 {
        return;
    }
    let inner = inner_area(area);
    frame.render_widget(panel(" CPU "), area);

    let chunks = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .split(inner);

    let pct = app.cpu_usage;
    let gauge = LineGauge::default()
        .ratio((pct as f64 / 100.0).clamp(0.0, 1.0))
        .filled_style(Style::new().fg(usage_color(pct)))
        .unfilled_style(Style::new().fg(Color::Rgb(58, 66, 86)))
        .label(Span::styled(
            format!(" Overall  {pct:5.1}%"),
            Style::new().fg(TEXT).add_modifier(Modifier::BOLD),
        ));
    frame.render_widget(gauge, chunks[0]);

    let width = chunks[1].width.saturating_sub(2);
    let per_line = (width / 24).max(1) as usize;

    let mut lines: Vec<Line> = Vec::new();
    for (i, usage) in app.cores.iter().enumerate() {
        if i > 0 && i % per_line == 0 {
            lines.push(Line::from(""));
        }
        let filled = (usage / 5.0).round() as usize;
        let bar = "█".repeat(filled) + &"░".repeat(20 - filled);
        lines.push(Line::from(vec![
            Span::styled(format!("C{:02} ", i + 1), Style::new().fg(DIM)),
            Span::styled(bar, Style::new().fg(usage_color(*usage))),
            Span::styled(format!(" {usage:3.0}%"), Style::new().fg(TEXT)),
        ]));
    }
    frame.render_widget(Paragraph::new(lines).style(Style::new().bg(PANEL)), chunks[1]);

    let data: Vec<u64> = app.cpu_history.iter().copied().collect();
    frame.render_widget(
        Sparkline::default().data(&data).max(255).style(Style::new().fg(ACCENT)),
        chunks[2],
    );
}

fn draw_memory(frame: &mut Frame, app: &App, area: Rect) {
    if area.height < 2 {
        return;
    }
    let inner = inner_area(area);
    frame.render_widget(panel(" MEMORY "), area);

    let chunks = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .split(inner);

    let total = app.system.total_memory();
    let used = app.system.used_memory();
    let pct = if total > 0 {
        used as f64 / total as f64 * 100.0
    } else {
        0.0
    };

    let used_gauge = LineGauge::default()
        .ratio((pct / 100.0).clamp(0.0, 1.0))
        .filled_style(Style::new().fg(usage_color(pct as f32)))
        .unfilled_style(Style::new().fg(Color::Rgb(58, 66, 86)))
        .label(Span::styled(
            format!(" Used  {} / {}", human_bytes(used), human_bytes(total)),
            Style::new().fg(TEXT).add_modifier(Modifier::BOLD),
        ));
    frame.render_widget(used_gauge, chunks[0]);

    let swap_total = app.system.total_swap();
    let swap_used = app.system.used_swap();
    let swap_pct = if swap_total > 0 {
        swap_used as f64 / swap_total as f64 * 100.0
    } else {
        0.0
    };
    let swap_gauge = LineGauge::default()
        .ratio((swap_pct / 100.0).clamp(0.0, 1.0))
        .filled_style(Style::new().fg(usage_color(swap_pct as f32)))
        .unfilled_style(Style::new().fg(Color::Rgb(58, 66, 86)))
        .label(Span::styled(
            format!(" Swap  {} / {}", human_bytes(swap_used), human_bytes(swap_total)),
            Style::new().fg(DIM),
        ));
    frame.render_widget(swap_gauge, chunks[1]);

    let info = Line::from(vec![
        Span::styled("Free: ", Style::new().fg(DIM)),
        Span::styled(human_bytes(app.system.free_memory()), Style::new().fg(OK)),
        Span::styled("    Used: ", Style::new().fg(DIM)),
        Span::styled(
            format!("{pct:.1}%"),
            Style::new().fg(usage_color(pct as f32)).add_modifier(Modifier::BOLD),
        ),
    ]);
    frame.render_widget(
        Paragraph::new(info).style(Style::new().bg(PANEL)),
        chunks[2],
    );

    let data: Vec<u64> = app.memory_history.iter().copied().collect();
    frame.render_widget(
        Sparkline::default().data(&data).max(255).style(Style::new().fg(OK)),
        chunks[4],
    );
}

fn draw_disks(frame: &mut Frame, app: &App, area: Rect) {
    if area.height == 0 || app.disk_info.is_empty() {
        return;
    }
    let inner = inner_area(area);
    frame.render_widget(panel(" DISKS "), area);

    let mut lines: Vec<Line> = Vec::new();
    for d in &app.disk_info {
        let pct = if d.total > 0 {
            d.used as f64 / d.total as f64 * 100.0
        } else {
            0.0
        };

        let name = truncate(&d.name, 14);
        let mount = truncate(&d.mount, 20);
        let fixed = name.chars().count()
            + 1
            + mount.chars().count()
            + 1
            + d.kind.chars().count()
            + 3
            + d.fs.chars().count()
            + 3
            + 8;
        let bar_len = (inner.width as isize - fixed as isize).clamp(4, 40) as usize;
        let filled = ((pct / 100.0) * bar_len as f64).round() as usize;
        let bar = "█".repeat(filled) + &"░".repeat(bar_len - filled);

        lines.push(Line::from(vec![
            Span::styled(
                format!(" {name}"),
                Style::new().fg(TEXT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!(" [{mount}] "), Style::new().fg(DIM)),
            Span::styled(format!("[{}] ", d.kind), Style::new().fg(DIM)),
            Span::styled(
                format!("({}) ", d.fs),
                Style::new().fg(DIM),
            ),
            Span::styled(bar, Style::new().fg(usage_color(pct as f32))),
            Span::styled(
                format!(" {pct:4.1}%"),
                Style::new().fg(usage_color(pct as f32)),
            ),
        ]));
    }
    frame.render_widget(Paragraph::new(lines).style(Style::new().bg(PANEL)), inner);
}

fn draw_processes(frame: &mut Frame, app: &mut App, area: Rect) {
    if area.height < 3 {
        return;
    }
    let indicator = if app.sort_desc { "↓" } else { "↑" };
    let title = format!(" Processes  ·  sorted by {} {}", app.sort.label(), indicator);

    let block = Block::new()
        .title(Title::from(Span::styled(
            format!(" {title} "),
            Style::new().fg(TITLE).add_modifier(Modifier::BOLD),
        )))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(ACCENT))
        .style(Style::new().bg(PANEL));

    let header_style = Style::new().fg(ACCENT).add_modifier(Modifier::BOLD);
    let header = Row::new(vec![
        Cell::from("PID"),
        Cell::from("CPU%"),
        Cell::from("MEM"),
        Cell::from("STATUS"),
        Cell::from("NAME"),
    ])
    .style(header_style);

    let rows: Vec<Row> = app
        .processes
        .iter()
        .map(|p| {
            Row::new(vec![
                Cell::from(p.pid.to_string()),
                Cell::from(format!("{:5.1}", p.cpu)),
                Cell::from(human_bytes(p.mem)),
                Cell::from(p.status.clone()),
                Cell::from(p.name.clone()),
            ])
            .style(Style::new().fg(TEXT))
        })
        .collect();

    let widths = [
        Constraint::Length(8),
        Constraint::Length(8),
        Constraint::Length(10),
        Constraint::Length(12),
        Constraint::Min(10),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .column_spacing(2)
        .highlight_symbol("▸ ")
        .row_highlight_style(Style::new().bg(ACCENT).fg(BG).add_modifier(Modifier::BOLD));
    frame.render_stateful_widget(table, area, &mut app.process_state);
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let key = |c: &str| {
        Span::styled(
            format!(" {c} "),
            Style::new().fg(BG).bg(ACCENT).add_modifier(Modifier::BOLD),
        )
    };
    let dim = |s: &str| Span::styled(format!(" {s} "), Style::new().fg(DIM));

    let mut spans = vec![
        key("q"),
        dim("Quit"),
        Span::raw("   "),
        key("↑↓"),
        dim("Select"),
        Span::raw("   "),
        key("d"),
        dim("Kill"),
        Span::raw("   "),
        key("s"),
        dim("Sort"),
        Span::raw("   "),
        key("p"),
        dim("Pause"),
        Span::raw("   "),
        key("-/+"),
        dim("Speed"),
        Span::raw("   "),
        key("r"),
        dim("Refresh"),
    ];

    if app.paused {
        spans.push(Span::raw("   "));
        spans.push(Span::styled(
            " PAUSED ",
            Style::new().fg(BG).bg(WARN).add_modifier(Modifier::BOLD),
        ));
    } else {
        spans.push(Span::raw("   "));
        spans.push(Span::styled(
            format!(" {}ms ", app.interval_ms),
            Style::new().fg(DIM),
        ));
    }

    frame.render_widget(
        Paragraph::new(Line::from(spans))
            .style(Style::new().bg(BG))
            .alignment(Alignment::Center),
        area,
    );
}

fn draw_confirm(frame: &mut Frame, app: &App, area: Rect) {
    let w = area.width.min(56);
    let h = 8;
    let x = area.x + area.width.saturating_sub(w) / 2;
    let y = area.y + area.height.saturating_sub(h) / 2;
    let rect = Rect::new(x, y, w, h);
    frame.render_widget(Clear, rect);

    let (pid, name) = app.confirm.as_ref().unwrap();
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Terminate process?",
            Style::new().fg(TITLE).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("PID {pid}  —  {name}"),
            Style::new().fg(TEXT),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  y  Kill   ·   n / Esc  Cancel  ",
            Style::new().fg(DIM).add_modifier(Modifier::BOLD),
        )),
    ];

    let block = Block::new()
        .title(Title::from(Span::styled(
            " ⚠ Confirm ",
            Style::new().fg(DANGER).add_modifier(Modifier::BOLD),
        )))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(DANGER))
        .style(Style::new().bg(Color::Rgb(40, 26, 32)));

    frame.render_widget(
        Paragraph::new(lines).block(block).alignment(Alignment::Center),
        rect,
    );
}

fn usage_color(pct: f32) -> Color {
    if pct >= 80.0 {
        DANGER
    } else if pct >= 50.0 {
        WARN
    } else {
        OK
    }
}

fn human_bytes(b: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut v = b as f64;
    let mut i = 0;
    while v >= 1024.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    format!("{v:.1} {}", UNITS[i])
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let cut: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{cut}…")
    }
}

fn format_uptime(secs: u64) -> String {
    let d = secs / 86400;
    let h = (secs % 86400) / 3600;
    let m = (secs % 3600) / 60;
    if d > 0 {
        format!("{d}d {h}h {m}m")
    } else if h > 0 {
        format!("{h}h {m}m")
    } else {
        format!("{m}m")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_bytes_scales_units() {
        assert_eq!(human_bytes(0), "0.0 B");
        assert_eq!(human_bytes(1023), "1023.0 B");
        assert_eq!(human_bytes(1024), "1.0 KB");
        assert_eq!(human_bytes(3 * 1024 * 1024), "3.0 MB");
        assert_eq!(human_bytes(2 * 1024 * 1024 * 1024), "2.0 GB");
    }

    #[test]
    fn uptime_formats_units() {
        assert_eq!(format_uptime(45), "0m");
        assert_eq!(format_uptime(3600), "1h 0m");
        assert_eq!(format_uptime(86400), "1d 0h 0m");
        assert_eq!(format_uptime(90061), "1d 1h 1m");
    }

    #[test]
    fn usage_color_thresholds() {
        assert_eq!(usage_color(0.0), OK);
        assert_eq!(usage_color(49.0), OK);
        assert_eq!(usage_color(50.0), WARN);
        assert_eq!(usage_color(79.0), WARN);
        assert_eq!(usage_color(80.0), DANGER);
    }
}
