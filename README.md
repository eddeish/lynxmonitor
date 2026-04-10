# 🐱 LynxMonitor

> Modern Linux System Monitor — fast, minimal, beautiful.

A high-performance CLI system monitor written in **Rust**, inspired by `htop`, `glances`, and `btop` — but with a cleaner architecture and lower resource footprint.

![Rust](https://img.shields.io/badge/language-Rust-orange?style=flat-square&logo=rust)
![Linux](https://img.shields.io/badge/platform-Linux-blue?style=flat-square&logo=linux)
![License](https://img.shields.io/badge/license-MIT-green?style=flat-square)

---

## Features

- **Real-time system monitoring** with configurable refresh rate
- **CPU**: total usage gauge + per-core bars + frequency + temperature
- **Memory**: RAM and Swap gauges with smart byte formatting
- **Disk**: read/write speeds and filesystem usage
- **Network**: upload/download speeds per interface
- **Processes**: sortable, filterable list with kill support
- **Keyboard navigation**: arrows, PageUp/Down, Home/End
- **Color-coded** metrics: Green → Yellow → Red by severity
- Very low resource usage (< 1% CPU when idle)

---

## Screenshots

_Terminal UI with CPU, RAM, Disk, Network panels and process list_

---

## Installation

### Requirements

- Linux (reads `/proc` and `/sys` directly)
- [Rust](https://rustup.rs/) 1.70+

### Build from source

```bash
git clone https://github.com/YOUR_USERNAME/lynxmonitor.git
cd lynxmonitor
cargo build --release
./target/release/lynxmonitor
```

### Install globally

```bash
cargo install --path .
lynxmonitor
```

---

## Usage

```bash
lynxmonitor                        # default (1s refresh, sort by CPU)
lynxmonitor --refresh 500          # refresh every 500ms
lynxmonitor --sort mem             # sort processes by memory
lynxmonitor --sort pid             # sort processes by PID
lynxmonitor --filter root          # show only root processes
lynxmonitor --no-gpu               # disable GPU panel
```

---

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `q` / `Esc` | Quit |
| `↑` / `l` | Move up in process list |
| `↓` / `j` | Move down in process list |
| `PgUp` / `PgDn` | Scroll a full page |
| `Home` / `End` | Jump to top / bottom |
| `s` | Cycle sort: CPU → MEM → PID |
| `K` | Kill selected process (SIGTERM) |

---

## Architecture

```
src/
├── main.rs                  # Entry point, main loop, CLI args
├── models.rs                # Data models (CpuStats, MemoryStats, ...)
├── collectors/
│   ├── cpu.rs               # /proc/stat parser
│   ├── memory.rs            # /proc/meminfo parser
│   ├── disk.rs              # /proc/diskstats + statvfs
│   ├── network.rs           # /proc/net/dev parser
│   └── system.rs            # uptime, running services
├── process/
│   └── manager.rs           # Process list, kill, renice
├── ui/
│   └── engine.rs            # Ratatui TUI rendering + input
└── utils/
    └── logger.rs            # Structured logger
```

---

## License

MIT © 2024
