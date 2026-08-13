use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::Marker,
    text::{Line, Span, Text},
    widgets::{
        Axis, Block, BorderType, Borders, Cell, Chart, Clear, Dataset, GraphType, LineGauge,
        Paragraph, Row, Sparkline, Table, block::Title,
    },
};
use sysinfo::System;

use crate::app::App;
use crate::theme::Theme;

const UNFILLED: Color = Color::Rgb(58, 66, 86);
const CORE_COLORS: [Color; 8] = [
    Color::Rgb(255, 121, 198),
    Color::Rgb(80, 250, 123),
    Color::Rgb(255, 184, 108),
    Color::Rgb(139, 233, 253),
    Color::Rgb(189, 147, 249),
    Color::Rgb(241, 250, 140),
    Color::Rgb(255, 85, 85),
    Color::Rgb(94, 188, 255),
];

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    if area.width < 8 || area.height < 5 {
        return;
    }
    let th = &app.theme;
    frame.render_widget(Paragraph::new("").style(Style::new().bg(th.bg)), area);

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

fn panel(title: &str, th: &Theme) -> Block<'static> {
    Block::new()
        .title(Title::from(Span::styled(
            format!(" {title} "),
            Style::new().fg(th.title).add_modifier(Modifier::BOLD),
        )))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(th.accent))
        .style(Style::new().bg(th.panel))
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
    let th = &app.theme;
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let left = Line::from(vec![
        Span::styled("▣ ", Style::new().fg(th.accent).add_modifier(Modifier::BOLD)),
        Span::styled(
            "RMon",
            Style::new().fg(th.title).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  v{}", env!("CARGO_PKG_VERSION")),
            Style::new().fg(th.dim),
        ),
    ]);

    let right = Line::from(vec![
        Span::styled("● ", Style::new().fg(if app.paused { th.warn } else { th.ok })),
        Span::styled(now, Style::new().fg(th.dim)),
    ]);
    let os = System::long_os_version().unwrap_or_else(|| "Unknown OS".to_string());
    let host = System::host_name().unwrap_or_else(|| "unknown".to_string());
    let arch = System::cpu_arch().unwrap_or_else(|| "?".to_string());

    let line1 = Line::from(vec![
        Span::styled(host, Style::new().fg(th.text).add_modifier(Modifier::BOLD)),
        Span::styled("  ·  ", Style::new().fg(th.dim)),
        Span::styled(os, Style::new().fg(th.text)),
    ]);

    let line2 = Line::from(vec![
        Span::styled("CPU: ", Style::new().fg(th.dim)),
        Span::styled(format!("{}× {}", app.system.cpus().len(), arch), Style::new().fg(th.text)),
        Span::styled("   Uptime: ", Style::new().fg(th.dim)),
        Span::styled(format_uptime(System::uptime()), Style::new().fg(th.text)),
        Span::styled("   Processes: ", Style::new().fg(th.dim)),
        Span::styled(format!("{}", app.process_count), Style::new().fg(th.text)),
    ]);

    let block = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(th.accent))
        .title(Title::from(left))
        .title(Title {
            content: right,
            alignment: Some(Alignment::Right),
            position: None,
        })
        .style(Style::new().bg(th.panel));

    frame.render_widget(
        Paragraph::new(Text::from(vec![line1, Line::from(" "), line2]))
            .block(block)
            .style(Style::new().fg(th.text)),
        area,
    );
}

fn draw_main(frame: &mut Frame, app: &mut App, area: Rect) {
    let avail = area.height.saturating_sub(16);
    let disks_h = (app.disk_info.len() as u16 * 2 + 4).min(avail).min(10);
    let bottom_h = disks_h.max(6);

    let chunks = Layout::vertical([
        Constraint::Length(12),
        Constraint::Length(bottom_h),
        Constraint::Min(4),
    ])
    .split(area);

    let row = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);
    draw_cpu(frame, app, row[0]);
    draw_memory(frame, app, row[1]);

    let row2 = Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(chunks[1]);
    draw_network(frame, app, row2[0]);
    draw_disks(frame, app, row2[1]);

    app.table_area = Some(chunks[2]);
    draw_processes(frame, app, chunks[2]);
}

fn draw_cpu(frame: &mut Frame, app: &App, area: Rect) {
    if area.height < 2 {
        return;
    }
    let th = &app.theme;
    let inner = inner_area(area);
    frame.render_widget(panel(" CPU ", th), area);

    let chunks = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(0),
        Constraint::Length(2),
        Constraint::Length(3),
    ])
    .split(inner);

    let pct = app.cpu_usage;
    let gauge = LineGauge::default()
        .ratio((pct as f64 / 100.0).clamp(0.0, 1.0))
        .filled_style(Style::new().fg(usage_color(pct, th)))
        .unfilled_style(Style::new().fg(UNFILLED))
        .label(Span::styled(
            format!(" Overall  {pct:5.1}%"),
            Style::new().fg(th.text).add_modifier(Modifier::BOLD),
        ));
    frame.render_widget(gauge, chunks[0]);

    draw_core_bars(frame, app, chunks[1]);

    let data: Vec<u64> = app.cpu_history.iter().copied().collect();
    frame.render_widget(
        Sparkline::default().data(&data).max(255).style(Style::new().fg(th.accent)),
        chunks[2],
    );

    draw_core_chart(frame, app, chunks[3]);
}

fn draw_core_bars(frame: &mut Frame, app: &App, area: Rect) {
    if area.height == 0 {
        return;
    }
    let th = &app.theme;
    let width = area.width.saturating_sub(2);
    let per_line = (width / 24).max(1) as usize;

    let mut lines: Vec<Line> = Vec::new();
    for (i, usage) in app.cores.iter().enumerate() {
        if i > 0 && i % per_line == 0 {
            lines.push(Line::from(""));
        }
        let filled = (usage / 5.0).round() as usize;
        let bar = "█".repeat(filled) + &"░".repeat(20 - filled);
        lines.push(Line::from(vec![
            Span::styled(format!("C{:02} ", i + 1), Style::new().fg(th.dim)),
            Span::styled(bar, Style::new().fg(usage_color(*usage, th))),
            Span::styled(format!(" {usage:3.0}%"), Style::new().fg(th.text)),
        ]));
    }
    frame.render_widget(Paragraph::new(lines).style(Style::new().bg(th.panel)), area);
}

fn draw_core_chart(frame: &mut Frame, app: &App, area: Rect) {
    if area.height < 2 || app.core_history.is_empty() {
        return;
    }
    let th = &app.theme;
    let n = app.core_history.len();
    let max_cores = 24;

    let mut points: Vec<Vec<(f64, f64)>> = Vec::with_capacity(n.min(max_cores));
    for hist in app.core_history.iter().take(max_cores) {
        points.push(
            hist.iter()
                .enumerate()
                .map(|(x, &v)| (x as f64, v as f64))
                .collect(),
        );
    }

    let datasets: Vec<Dataset> = points
        .iter()
        .enumerate()
        .map(|(i, pts)| {
            Dataset::default()
                .marker(Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::new().fg(CORE_COLORS[i % CORE_COLORS.len()]))
                .data(pts)
        })
        .collect();

    let x_max = app.core_history[0].len().max(1) as f64;
    let chart = Chart::new(datasets)
        .x_axis(Axis::default().bounds([0.0, x_max]).labels(["past", "now"]))
        .y_axis(Axis::default().bounds([0.0, 100.0]).labels(["0", "50", "100"]))
        .style(Style::new().bg(th.panel))
        .legend_position(None);
    frame.render_widget(chart, area);
}

fn draw_memory(frame: &mut Frame, app: &App, area: Rect) {
    if area.height < 2 {
        return;
    }
    let th = &app.theme;
    let inner = inner_area(area);
    frame.render_widget(panel(" MEMORY ", th), area);

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
        .filled_style(Style::new().fg(usage_color(pct as f32, th)))
        .unfilled_style(Style::new().fg(UNFILLED))
        .label(Span::styled(
            format!(" Used  {} / {}", human_bytes(used), human_bytes(total)),
            Style::new().fg(th.text).add_modifier(Modifier::BOLD),
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
        .filled_style(Style::new().fg(usage_color(swap_pct as f32, th)))
        .unfilled_style(Style::new().fg(UNFILLED))
        .label(Span::styled(
            format!(" Swap  {} / {}", human_bytes(swap_used), human_bytes(swap_total)),
            Style::new().fg(th.dim),
        ));
    frame.render_widget(swap_gauge, chunks[1]);

    let info = Line::from(vec![
        Span::styled("Free: ", Style::new().fg(th.dim)),
        Span::styled(human_bytes(app.system.free_memory()), Style::new().fg(th.ok)),
        Span::styled("    Used: ", Style::new().fg(th.dim)),
        Span::styled(
            format!("{pct:.1}%"),
            Style::new().fg(usage_color(pct as f32, th)).add_modifier(Modifier::BOLD),
        ),
    ]);
    frame.render_widget(
        Paragraph::new(info).style(Style::new().bg(th.panel)),
        chunks[2],
    );

    let data: Vec<u64> = app.memory_history.iter().copied().collect();
    frame.render_widget(
        Sparkline::default().data(&data).max(255).style(Style::new().fg(th.ok)),
        chunks[4],
    );
}

fn draw_network(frame: &mut Frame, app: &App, area: Rect) {
    if area.height < 2 {
        return;
    }
    let th = &app.theme;
    let inner = inner_area(area);
    frame.render_widget(panel(" NETWORK ", th), area);

    let chunks = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(2),
        Constraint::Min(0),
    ])
    .split(inner);

    let dl = LineGauge::default()
        .ratio(speed_ratio(app.net_down_speed))
        .filled_style(Style::new().fg(th.accent))
        .unfilled_style(Style::new().fg(UNFILLED))
        .label(Span::styled(
            format!(" ↓ {}", format_speed(app.net_down_speed)),
            Style::new().fg(th.text).add_modifier(Modifier::BOLD),
        ));
    frame.render_widget(dl, chunks[0]);

    let ul = LineGauge::default()
        .ratio(speed_ratio(app.net_up_speed))
        .filled_style(Style::new().fg(th.ok))
        .unfilled_style(Style::new().fg(UNFILLED))
        .label(Span::styled(
            format!(" ↑ {}", format_speed(app.net_up_speed)),
            Style::new().fg(th.text).add_modifier(Modifier::BOLD),
        ));
    frame.render_widget(ul, chunks[1]);

    let totals = Line::from(vec![
        Span::styled("Total: ", Style::new().fg(th.dim)),
        Span::styled(
            format!("↓ {}  ↑ {}", human_bytes(app.net_down_total), human_bytes(app.net_up_total)),
            Style::new().fg(th.dim),
        ),
    ]);
    frame.render_widget(
        Paragraph::new(totals).style(Style::new().bg(th.panel)),
        chunks[2],
    );
}

fn draw_disks(frame: &mut Frame, app: &App, area: Rect) {
    if area.height == 0 || app.disk_info.is_empty() {
        return;
    }
    let th = &app.theme;
    let inner = inner_area(area);
    frame.render_widget(panel(" DISKS ", th), area);

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
                Style::new().fg(th.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!(" [{mount}] "), Style::new().fg(th.dim)),
            Span::styled(format!("[{}] ", d.kind), Style::new().fg(th.dim)),
            Span::styled(
                format!("({}) ", d.fs),
                Style::new().fg(th.dim),
            ),
            Span::styled(bar, Style::new().fg(usage_color(pct as f32, th))),
            Span::styled(
                format!(" {pct:4.1}%"),
                Style::new().fg(usage_color(pct as f32, th)),
            ),
        ]));
    }
    frame.render_widget(Paragraph::new(lines).style(Style::new().bg(th.panel)), inner);
}

fn draw_processes(frame: &mut Frame, app: &mut App, area: Rect) {
    if area.height < 3 {
        return;
    }
    let th = &app.theme;
    let indicator = if app.sort_desc { "↓" } else { "↑" };
    let title = format!(" Processes  ·  sorted by {} {}", app.sort.label(), indicator);

    let block = Block::new()
        .title(Title::from(Span::styled(
            format!(" {title} "),
            Style::new().fg(th.title).add_modifier(Modifier::BOLD),
        )))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(th.accent))
        .style(Style::new().bg(th.panel));

    let header_style = Style::new().fg(th.accent).add_modifier(Modifier::BOLD);
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
            .style(Style::new().fg(th.text))
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
        .row_highlight_style(Style::new().bg(th.accent).fg(th.bg).add_modifier(Modifier::BOLD));
    frame.render_stateful_widget(table, area, &mut app.process_state);
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let th = &app.theme;
    let key = |c: &str| {
        Span::styled(
            format!(" {c} "),
            Style::new().fg(th.bg).bg(th.accent).add_modifier(Modifier::BOLD),
        )
    };
    let dim = |s: &str| Span::styled(format!(" {s} "), Style::new().fg(th.dim));

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
            Style::new().fg(th.bg).bg(th.warn).add_modifier(Modifier::BOLD),
        ));
    } else {
        spans.push(Span::raw("   "));
        spans.push(Span::styled(
            format!(" {}ms ", app.interval_ms),
            Style::new().fg(th.dim),
        ));
    }

    frame.render_widget(
        Paragraph::new(Line::from(spans))
            .style(Style::new().bg(th.bg))
            .alignment(Alignment::Center),
        area,
    );
}

fn draw_confirm(frame: &mut Frame, app: &App, area: Rect) {
    let th = &app.theme;
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
            Style::new().fg(th.title).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("PID {pid}  —  {name}"),
            Style::new().fg(th.text),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  y  Kill   ·   n / Esc  Cancel  ",
            Style::new().fg(th.dim).add_modifier(Modifier::BOLD),
        )),
    ];

    let block = Block::new()
        .title(Title::from(Span::styled(
            " ⚠ Confirm ",
            Style::new().fg(th.danger).add_modifier(Modifier::BOLD),
        )))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(th.danger))
        .style(Style::new().bg(th.panel));

    frame.render_widget(
        Paragraph::new(lines).block(block).alignment(Alignment::Center),
        rect,
    );
}

fn usage_color(pct: f32, th: &Theme) -> Color {
    if pct >= 80.0 {
        th.danger
    } else if pct >= 50.0 {
        th.warn
    } else {
        th.ok
    }
}

fn speed_ratio(bps: u64) -> f64 {
    const FULL_SCALE: f64 = 10.0 * 1024.0 * 1024.0;
    (bps as f64 / FULL_SCALE).clamp(0.0, 1.0)
}

fn format_speed(bps: u64) -> String {
    format!("{}/s", human_bytes(bps))
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
        let th = crate::theme::dark();
        assert_eq!(usage_color(0.0, &th), th.ok);
        assert_eq!(usage_color(49.0, &th), th.ok);
        assert_eq!(usage_color(50.0, &th), th.warn);
        assert_eq!(usage_color(79.0, &th), th.warn);
        assert_eq!(usage_color(80.0, &th), th.danger);
    }

    #[test]
    fn speed_ratio_full_scale() {
        assert_eq!(speed_ratio(0), 0.0);
        assert_eq!(speed_ratio(10 * 1024 * 1024), 1.0);
        assert!(speed_ratio(5 * 1024 * 1024) > 0.4);
        assert_eq!(speed_ratio(u64::MAX), 1.0);
    }
}
