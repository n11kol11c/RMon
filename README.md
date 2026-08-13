# 🖥️ RMon — Real-time System Monitor for the Terminal

> Your computer's **vitals**, rendered in beautiful ANSI color, right inside your terminal.
> Think *Task Manager*, *htop*, and a NASA mission control display had a baby — written in pure 🦀 **Rust**.

![Rust](https://img.shields.io/badge/built%20with-Rust-%23dea584?style=flat-square&logo=rust&logoColor=white)
![Platforms](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-%2358b0ff?style=flat-square)
![TUI](https://img.shields.io/badge/UI-Ratatui-%23f7f8f8?style=flat-square)
![License](https://img.shields.io/badge/license-MIT-%23a8ff60?style=flat-square)
![Status](https://img.shields.io/badge/status-alive%20and%20kicking-%2392dc92?style=flat-square)

---

## 🚀 What Is RMon?

**RMon** is a cross-platform **terminal system monitor**. It watches your machine's
CPU, memory, disk, and running processes — and shows it all on one live dashboard
that refreshes itself while you watch.

It runs on **Windows**, **macOS**, and **Linux** from a single codebase, and needs
**no GUI**, **no browser**, and **no cloud**. Just you, a terminal, and a lot of
colored block characters.

### ✨ Features

- 🔥 **Live CPU usage** — overall gauge, per-core bars, a rolling history sparkline, **and a real-time per-core line chart**
- 🧠 **Memory & Swap** — used/free with color-coded gauges and sparkline
- 🌐 **Network panel** — live upload/download speeds (per second) plus lifetime traffic totals
- 💾 **Disk usage** — every mounted disk with usage bar, filesystem, and disk type
- 📋 **Process list** — PID, CPU%, memory, status, and name, continuously refreshed
- 🖱️ **Mouse support** — click a process row to select it, scroll the wheel to move the selection
- 🎨 **Themes** — pick from 6 built-in themes (or define your own colors in a config file)
- ⚙️ **Config file** — control theme, default sort order, refresh rate, and process-list size
- ⌨️ **Fully keyboard-driven** — no mouse required (but we love you anyway)
- 🎨 **Fancy terminal UI** — rounded panels, color-coded urgency
- ⚡ **Adjustable refresh speed** — from 200 ms (adrenaline) to 5 s (zen mode)
- 🧊 **Pause mode** — freeze the world when you need to stare at a number
- 🔪 **Kill processes** — right from the keyboard, with a safety confirmation dialog

---

## 🛠️ How It Works

RMon is built on a handful of battle-tested Rust crates:

| Crate | Job |
|-------|-----|
| [**sysinfo**](https://crates.io/crates/sysinfo) | Talks to the OS to fetch CPU, memory, disk, network, and process data on all three platforms |
| [**ratatui**](https://crates.io/crates/ratatui) | Renders the fancy terminal UI — panels, gauges, charts, sparklines, tables |
| [**crossterm**](https://crates.io/crates/crossterm) | Handles raw-mode keyboard *and mouse* input and terminal size events |
| [**serde**](https://crates.io/crates/serde) + [**toml**](https://crates.io/crates/toml) | Parses your `config.toml` and theme colors |

The architecture is cleanly split into modules:

```
src/
├── main.rs     → Boots the terminal (with mouse capture), runs the event/refresh loop
├── app.rs      → The "brain": gathers system data, keeps UI state, sorts processes
├── ui.rs       → The "eyes": draws every panel, gauge, chart, bar, and table
├── handlers.rs → The "ears": maps every keypress and mouse event to an action
├── config.rs   → Reads your TOML config file (theme, sort, refresh rate)
└── theme.rs    → The 6 built-in themes + custom hex-color overrides
```

**The loop** (it's simpler than it sounds):

1. **Refresh** — `sysinfo` grabs fresh CPU, memory, disk, network, and process data (~every 1 s).
2. **Render** — `ratatui` redraws the dashboard with the new numbers.
3. **Listen** — `crossterm` waits for your keypresses and mouse moves, so RMon is instant to interact with.

Repeat forever. That's it. That's the whole job. 💤

---

## 📦 Installation

### ✅ Requirements

**Supported platforms:**

| Platform | Notes |
|----------|-------|
| 🪟 Windows | 10 / 11 — works natively; `.sh` installers run in Git Bash / MSYS2 |
| 🍎 macOS | 12+ (Monterey and newer) |
| 🐧 Linux | Most modern distributions (kernel 4.x+); uses `/proc`, `/sys`, and `statvfs` |

**Software:**

- **Rust & Cargo** (edition 2021, Rust **1.70+**) — install from [rustup.rs](https://rustup.rs) if you don't have them
- **A terminal that likes ANSI colors** — pretty much any of them (Terminal.app, iTerm2, Windows Terminal, GNOME Terminal, Alacritty, Kitty, VS Code integrated terminal…)

**Recommended terminal setup:**

- 🖥️ Minimum size **80 × 30** (the dashboard adapts to smaller sizes, but it gets cozy)
- 🔤 A font with **box-drawing, block, and braille glyphs** (`█ ░ ▒ ▓ ─ │ ⣿`) — most modern terminal fonts include these; Nerd Fonts look best
- 🌈 **Truecolor / 256-color** enabled for the full fancy experience (RMon falls back gracefully if not)

**Build resources (only needed if building from source):**

- ~1.5 GB free disk space (Rust toolchain + dependencies)
- An internet connection on first build (crates are downloaded once, then cached)

**Runtime footprint:**

- 🐜 Tiny — a single ~800 KB binary, no runtime dependencies, no daemon, no cloud.

### 🧰 Option A — One-line installer (macOS / Linux)

```bash
./install.sh
```

This will:

1. 🏗️ Build the release binary (`cargo build --release`)
2. 📁 Install it to `~/.local/bin/rmon`
3. 🧭 Print a hint if that folder isn't on your `PATH`

**Install to a custom location:**

```bash
PREFIX="$HOME/bin" ./install.sh
```

### 🧰 Option B — Install with Cargo (all platforms, including Windows)

```bash
cargo install --path .
```

### 🧰 Option C — Just run it (no install at all)

```bash
cargo run            # development build
./target/release/rmon  # if you've already built once
```

> 💡 **Windows users:** the `.sh` installers work great in **Git Bash** / **MSYS2**.
> Or skip them entirely and use `cargo install` — RMon is fully Windows-native.

---

## 🎮 How to Use

```bash
rmon
```

That's it. The dashboard appears and starts live-refreshing. Now take control:

| Key | Action |
|-----|--------|
| `q` / `Esc` / `Ctrl+C` | 👋 Quit |
| `↑` / `↓` or `j` / `k` | Navigate the process list |
| `PgUp` / `PgDn` | Jump 10 processes at a time |
| `Home` / `End` | Jump to first / last process |
| `d` / `Delete` | 🔪 Kill the selected process (asks for confirmation) |
| `y` / `n` | Confirm / cancel the kill |
| `s` | Cycle sort: CPU% → Memory → Name → PID |
| `p` | ⏸️ Pause / resume the live refresh |
| `-` / `[` | 🐢 Slow down refresh (up to 5 s) |
| `+` / `]` | 🐇 Speed up refresh (down to 200 ms) |
| `r` | 🔄 Force an immediate refresh |

**Mouse:**

| Action | Effect |
|--------|--------|
| 🖱️ Click a row in the process table | Select that process |
| 🖱️ Scroll wheel up / down | Move the selection through the process list |

**Pro tips:**

- The gauge colors tell you how stressed things are:
  🟢 green = chill · 🟡 yellow = getting warm · 🔴 red = panic mode
- Sort by **Memory** and pause (`p`) to catch that sneaky RAM hog in the act.
- The per-core line chart under CPU shows every core's last ~2 minutes of history.
- The sparklines under CPU/Memory show your last ~2 minutes of activity.

---

## ⚙️ Configuration

RMon reads a TOML config file on startup, if one exists:

| Platform | Location |
|----------|----------|
| 🍎 macOS / 🐧 Linux | `~/.config/rmon/config.toml` (or `$XDG_CONFIG_HOME/rmon/config.toml`) |
| 🪟 Windows | `%APPDATA%\rmon\config.toml` |

You can point it anywhere with the `RMON_CONFIG` env var:

```bash
RMON_CONFIG="$HOME/my-rmon.toml" rmon
```

### Example config

```toml
[monitor]
refresh_ms = 1000   # refresh every second (200–5000)
sort = "cpu"        # default sort: cpu | memory | name | pid
max_processes = 300 # how many processes the table shows

[theme]
name = "dracula"    # see theme list below
```

### 🎨 Themes

RMon ships with **6 built-in themes** — set one with `name`:

`dark` · `light` · `gruvbox` · `dracula` · `solarized` · `nord`

Want your own colors? Leave `name` out (or keep it as a base) and override any
slot with hex values:

```toml
[theme]
name = "dark"

[theme.colors]
bg     = "#0d1117"
panel  = "#161b22"
accent = "#58a6ff"
title  = "#79c0ff"
text   = "#c9d1d9"
dim    = "#8b949e"
ok     = "#3fb950"
warn   = "#d29922"
danger = "#f85149"
```

Any field you omit keeps the theme's default. Bad hex values or unknown theme
names are ignored (with a warning) and RMon falls back to `dark`.

---

## 🗑️ Uninstall

> *"Goodbyes are hard. This one is easy."*

### The polite way (macOS / Linux)

```bash
./uninstall.sh
```

This removes the `rmon` binary from `~/.local/bin` and asks if you'd also like to
delete the build artifacts in `target/` (yes, that's where we keep the spare bits).

**To also clean a custom install location:**

```bash
PREFIX="$HOME/bin" ./uninstall.sh
```

### The blunt way (everything, everywhere, all at once)

```bash
rm ~/.local/bin/rmon     # remove the binary
rm -rf ~/Desktop/projects/RMon/target   # remove build artifacts
rm -rf ~/Desktop/projects/RMon          # remove the project entirely
```

> ⚠️ **Heads up:** like a breakup, this one's permanent. You can always
> `git clone` / re-download and start the romance again, though. 💔→❤️

### The Cargo way

```bash
cargo uninstall rmon
```

---

## 🆘 Troubleshooting

| Problem | Fix |
|---------|-----|
| `rmon: command not found` | `~/.local/bin` isn't on your `PATH`. Add it (the installer prints how) or use the full path. |
| Everything looks like a wall of boxes | Your terminal font is missing block glyphs. Try a Nerd Font or any modern terminal font. |
| The per-core chart is just dots | Braille glyphs (`⣿`) missing from your font — same fix as above. |
| CPU shows 0% on the first frame | Normal — RMon needs two samples to compute usage. It corrects itself on the next tick. |
| Terminal looks scrambled after an accident | RMon always restores your terminal (including mouse capture) on exit and on panics. If something weird happens, just run `reset`. |
| You can't kill a process | Some processes need elevated privileges. That's the OS protecting itself — and occasionally you. |
| My config file is ignored | Double-check the path (see the Configuration section), or set `RMON_CONFIG` to force a location. Invalid TOML prints a warning at startup and falls back to defaults. |

---

## 🧭 Roadmap

- [x] CPU, memory, disk, and process monitoring
- [x] Keyboard navigation + process kill
- [x] Live sparkline graphs
- [x] Per-core graph history (proper line charts)
- [x] Network upload/download speed panel
- [x] Config file for colors, sorting, and refresh rate
- [x] Mouse support
- [x] Themes
- [ ] Process search / filtering
- [ ] Per-network-interface breakdown
- [ ] GPU and temperature sensors
- [ ] Custom keybindings

---

## 🤝 Contributing

Found a bug? Want a feature? Open an issue or submit a PR. Fork it, break it, fix it, love it.

## 📜 License

MIT. Do whatever you want, just don't blame us when you kill your own browser in the process table. 😄

---

<p align="center">
  <sub>Made with 💙, 🦀, and an unhealthy amount of `█░▒▓` characters.</sub>
</p>
