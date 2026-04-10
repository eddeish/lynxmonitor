use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Row, Table},
    Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Stdout};
use std::time::Duration;

use crate::models::{CpuStats, DiskStats, MemoryStats, NetworkStats, ProcessInfo};

pub enum InputResult {
    Continue,
    Quit,
    Kill(u32),
}

pub struct UiEngine {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    #[allow(dead_code)]
    show_gpu: bool,
    sort_mode: SortMode,
    filter: String,
    selected: usize,
    scroll: usize,
    visible_rows: usize,
}

#[derive(Clone, PartialEq)]
enum SortMode {
    Cpu,
    Mem,
    Pid,
}

impl SortMode {
    fn label(&self) -> &'static str {
        match self {
            SortMode::Cpu => "CPU",
            SortMode::Mem => "MEM",
            SortMode::Pid => "PID",
        }
    }

    fn next(&self) -> SortMode {
        match self {
            SortMode::Cpu => SortMode::Mem,
            SortMode::Mem => SortMode::Pid,
            SortMode::Pid => SortMode::Cpu,
        }
    }
}

impl UiEngine {
    pub fn new(show_gpu: bool, sort_mode: String, filter: String) -> anyhow::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        let parsed_sort = match sort_mode.as_str() {
            "mem" => SortMode::Mem,
            "pid" => SortMode::Pid,
            _ => SortMode::Cpu,
        };

        Ok(Self {
            terminal,
            show_gpu,
            sort_mode: parsed_sort,
            filter,
            selected: 0,
            scroll: 0,
            visible_rows: 20,
        })
    }

    pub fn cleanup(&mut self) -> anyhow::Result<()> {
        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    fn sorted_procs<'a>(&self, procs: &'a [ProcessInfo]) -> Vec<&'a ProcessInfo> {
        let mut list: Vec<&ProcessInfo> = if self.filter.is_empty() {
            procs.iter().collect()
        } else {
            procs
                .iter()
                .filter(|p| p.user.contains(&self.filter) || p.command.contains(&self.filter))
                .collect()
        };

        match self.sort_mode {
            SortMode::Cpu => list.sort_by(|a, b| {
                b.cpu_usage
                    .partial_cmp(&a.cpu_usage)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }),
            SortMode::Mem => list.sort_by(|a, b| b.memory_usage.cmp(&a.memory_usage)),
            SortMode::Pid => list.sort_by(|a, b| a.pid.cmp(&b.pid)),
        }
        list
    }

    fn cpu_color(usage: f32) -> Color {
        if usage >= 80.0 {
            Color::Red
        } else if usage >= 50.0 {
            Color::Yellow
        } else {
            Color::Green
        }
    }

    fn mem_color(pct: f64) -> Color {
        if pct >= 85.0 {
            Color::Red
        } else if pct >= 60.0 {
            Color::Yellow
        } else {
            Color::Cyan
        }
    }

    fn fmt_bytes(bytes: u64) -> String {
        if bytes >= 1024 * 1024 * 1024 {
            format!("{:.1} GB", bytes as f64 / 1024.0 / 1024.0 / 1024.0)
        } else if bytes >= 1024 * 1024 {
            format!("{:.1} MB", bytes as f64 / 1024.0 / 1024.0)
        } else if bytes >= 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else {
            format!("{} B", bytes)
        }
    }

    fn fmt_uptime(seconds: f64) -> String {
        let total = seconds as u64;
        let days = total / 86400;
        let hours = (total % 86400) / 3600;
        let mins = (total % 3600) / 60;
        let secs = total % 60;
        if days > 0 {
            format!("{}d {:02}:{:02}:{:02}", days, hours, mins, secs)
        } else {
            format!("{:02}:{:02}:{:02}", hours, mins, secs)
        }
    }

    pub fn draw(
        &mut self,
        cpu: &CpuStats,
        mem: &MemoryStats,
        disk: &DiskStats,
        net: &NetworkStats,
        procs: &[ProcessInfo],
        uptime: f64,
        services: u32,
    ) -> anyhow::Result<()> {
        let sorted = self.sorted_procs(procs);
        let total = sorted.len();
        let selected = self.selected;
        let scroll = self.scroll;
        let sort_label = self.sort_mode.label();
        let filter = self.filter.clone();

        let cpu_usage_pct = cpu.total_usage as u16;
        let mem_pct = if mem.total > 0 {
            (mem.used as f64 / mem.total as f64 * 100.0) as u16
        } else {
            0
        };
        let swap_pct = if mem.swap_total > 0 {
            (mem.swap_used as f64 / mem.swap_total as f64 * 100.0) as u16
        } else {
            0
        };
        let disk_pct = if disk.total > 0 {
            (disk.used as f64 / disk.total as f64 * 100.0) as u16
        } else {
            0
        };

        let mem_pct_f = mem_pct as f64;
        let cpu_color = Self::cpu_color(cpu.total_usage);
        let mem_color = Self::mem_color(mem_pct_f);

        let cpu_freq = cpu.frequency;
        let cpu_temp = cpu.temperature;
        let load_one = cpu.load.one;
        let load_five = cpu.load.five;
        let load_fifteen = cpu.load.fifteen;

        let cores_snapshot: Vec<f32> = cpu.cores_usage.clone();
        let mem_used = mem.used;
        let mem_total = mem.total;
        let swap_used = mem.swap_used;
        let swap_total = mem.swap_total;
        let disk_used = disk.used;
        let disk_total = disk.total;
        let disk_read = disk.read_speed;
        let disk_write = disk.write_speed;
        let net_dl = net.download_speed;
        let net_ul = net.upload_speed;

        self.visible_rows = 0;

        self.terminal.draw(|f| {
            let size = f.size();
            let outer = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(5),
                    Constraint::Length(5),
                    Constraint::Min(10),
                    Constraint::Length(2),
                ])
                .split(size);

            // ── Header ────────────────────────────────────────────────
            let header = Paragraph::new(Line::from(vec![
                Span::styled(
                    " 🐱 LynxMonitor ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("│", Style::default().fg(Color::DarkGray)),
                Span::raw(format!(
                    " Uptime {} │ Load {:.2} {:.2} {:.2} │ Services {} │ Procs {} ",
                    Self::fmt_uptime(uptime),
                    load_one,
                    load_five,
                    load_fifteen,
                    services,
                    total,
                )),
            ]))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
            f.render_widget(header, outer[0]);

            // ── CPU Row ───────────────────────────────────────────────
            let cpu_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(outer[1]);

            let cpu_gauge = Gauge::default()
                .block(
                    Block::default()
                        .title(format!(
                            " CPU  {:.1}%  {} MHz  {:.1}°C ",
                            cpu.total_usage, cpu_freq, cpu_temp
                        ))
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::DarkGray)),
                )
                .gauge_style(Style::default().fg(cpu_color).bg(Color::Black))
                .percent(cpu_usage_pct.min(100));
            f.render_widget(cpu_gauge, cpu_chunks[0]);

            let per_core_width = if !cores_snapshot.is_empty() {
                100u16 / cores_snapshot.len() as u16
            } else {
                100
            };

            let core_constraints: Vec<Constraint> = cores_snapshot
                .iter()
                .map(|_| Constraint::Percentage(per_core_width))
                .collect();

            if !cores_snapshot.is_empty() {
                let core_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(core_constraints)
                    .split(cpu_chunks[1]);

                for (i, usage) in cores_snapshot.iter().enumerate() {
                    if i < core_chunks.len() {
                        let g = Gauge::default()
                            .block(
                                Block::default()
                                    .title(format!("C{}", i))
                                    .borders(Borders::ALL)
                                    .border_style(Style::default().fg(Color::DarkGray)),
                            )
                            .gauge_style(
                                Style::default()
                                    .fg(Self::cpu_color(*usage))
                                    .bg(Color::Black),
                            )
                            .percent((*usage as u16).min(100));
                        f.render_widget(g, core_chunks[i]);
                    }
                }
            }

            // ── Metrics Row ───────────────────────────────────────────
            let metrics = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                ])
                .split(outer[2]);

            let mem_gauge = Gauge::default()
                .block(
                    Block::default()
                        .title(format!(
                            " RAM  {}  /  {} ",
                            Self::fmt_bytes(mem_used),
                            Self::fmt_bytes(mem_total)
                        ))
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::DarkGray)),
                )
                .gauge_style(Style::default().fg(mem_color).bg(Color::Black))
                .percent(mem_pct.min(100));
            f.render_widget(mem_gauge, metrics[0]);

            let swap_color = Self::mem_color(swap_pct as f64);
            let swap_gauge = Gauge::default()
                .block(
                    Block::default()
                        .title(format!(
                            " SWAP  {}  /  {} ",
                            Self::fmt_bytes(swap_used),
                            Self::fmt_bytes(swap_total)
                        ))
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::DarkGray)),
                )
                .gauge_style(Style::default().fg(swap_color).bg(Color::Black))
                .percent(swap_pct.min(100));
            f.render_widget(swap_gauge, metrics[1]);

            let disk_lines = vec![
                Line::from(vec![
                    Span::styled("▲ ", Style::default().fg(Color::Green)),
                    Span::raw(format!("W: {}/s", Self::fmt_bytes(disk_write))),
                ]),
                Line::from(vec![
                    Span::styled("▼ ", Style::default().fg(Color::Cyan)),
                    Span::raw(format!("R: {}/s", Self::fmt_bytes(disk_read))),
                ]),
                Line::from(format!(
                    "{} / {}",
                    Self::fmt_bytes(disk_used),
                    Self::fmt_bytes(disk_total)
                )),
            ];
            let disk_block = Paragraph::new(disk_lines).block(
                Block::default()
                    .title(format!(" Disk  {}% ", disk_pct))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
            f.render_widget(disk_block, metrics[2]);

            let net_lines = vec![
                Line::from(vec![
                    Span::styled("▲ ", Style::default().fg(Color::Green)),
                    Span::raw(format!("{}/s", Self::fmt_bytes(net_ul))),
                ]),
                Line::from(vec![
                    Span::styled("▼ ", Style::default().fg(Color::Cyan)),
                    Span::raw(format!("{}/s", Self::fmt_bytes(net_dl))),
                ]),
            ];
            let net_block = Paragraph::new(net_lines).block(
                Block::default()
                    .title(" Network ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
            f.render_widget(net_block, metrics[3]);

            // ── Process Table ─────────────────────────────────────────
            let proc_area = outer[3];
            let visible = proc_area.height.saturating_sub(4) as usize;

            let header_row = Row::new(vec!["  PID", "USER", "PRI", "NI", "S", "CPU%", "MEM", "THREADS", "COMMAND"])
                .style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                );

            let mut rows = Vec::new();
            for (i, p) in sorted.iter().enumerate().skip(scroll).take(visible) {
                let is_selected = i == selected;
                let cpu_style = if p.cpu_usage > 50.0 {
                    Style::default().fg(Color::Red)
                } else if p.cpu_usage > 20.0 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::Green)
                };

                let base_style = if is_selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let row = Row::new(vec![
                    format!("  {}", p.pid),
                    p.user.clone(),
                    p.priority.to_string(),
                    p.nice.to_string(),
                    p.state.to_string(),
                    format!("{:.1}", p.cpu_usage),
                    Self::fmt_bytes(p.memory_usage),
                    p.threads.to_string(),
                    p.command.clone(),
                ])
                .style(base_style);
                rows.push(row);
            }

            let widths = [
                Constraint::Length(8),
                Constraint::Length(12),
                Constraint::Length(4),
                Constraint::Length(4),
                Constraint::Length(3),
                Constraint::Length(7),
                Constraint::Length(10),
                Constraint::Length(8),
                Constraint::Min(20),
            ];

            let selected_info = if total > 0 {
                format!(" [{}/{}] ", selected + 1, total)
            } else {
                String::from(" [0/0] ")
            };

            let table = Table::new(rows, widths)
                .header(header_row)
                .block(
                    Block::default()
                        .title(format!(
                            " Processes{}│ Sort: {} │ Filter: {} ",
                            selected_info,
                            sort_label,
                            if filter.is_empty() { "—" } else { &filter }
                        ))
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::DarkGray)),
                );

            f.render_widget(table, proc_area);

            // ── Footer ────────────────────────────────────────────────
            let footer = Paragraph::new(Line::from(vec![
                Span::styled(" q", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw(" Quit  "),
                Span::styled("↑↓/jl", Style::default().fg(Color::Cyan)),
                Span::raw(" Navigate  "),
                Span::styled("s", Style::default().fg(Color::Yellow)),
                Span::raw(" Sort  "),
                Span::styled("K", Style::default().fg(Color::Magenta)),
                Span::raw(" Kill  "),
                Span::styled("PgUp/PgDn", Style::default().fg(Color::Cyan)),
                Span::raw(" Page  "),
                Span::styled("Home/End", Style::default().fg(Color::Cyan)),
                Span::raw(" Jump "),
            ]));
            f.render_widget(footer, outer[4]);
        })?;

        Ok(())
    }

    pub fn handle_input(&mut self, procs: &[crate::models::ProcessInfo]) -> anyhow::Result<InputResult> {
        let sorted = self.sorted_procs(procs);
        let total = sorted.len();

        if self.selected >= total && total > 0 {
            self.selected = total - 1;
        }

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    return Ok(InputResult::Continue);
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(InputResult::Quit),

                    KeyCode::Down | KeyCode::Char('j') => {
                        if total > 0 && self.selected < total - 1 {
                            self.selected += 1;
                            self.clamp_scroll(total);
                        }
                    }

                    KeyCode::Up | KeyCode::Char('l') => {
                        if self.selected > 0 {
                            self.selected -= 1;
                            self.clamp_scroll(total);
                        }
                    }

                    KeyCode::PageDown => {
                        let step = self.visible_rows.max(1);
                        self.selected = (self.selected + step).min(total.saturating_sub(1));
                        self.clamp_scroll(total);
                    }

                    KeyCode::PageUp => {
                        let step = self.visible_rows.max(1);
                        self.selected = self.selected.saturating_sub(step);
                        self.clamp_scroll(total);
                    }

                    KeyCode::Home => {
                        self.selected = 0;
                        self.scroll = 0;
                    }

                    KeyCode::End => {
                        self.selected = total.saturating_sub(1);
                        self.clamp_scroll(total);
                    }

                    KeyCode::Char('s') => {
                        self.sort_mode = self.sort_mode.next();
                        self.selected = 0;
                        self.scroll = 0;
                    }

                    KeyCode::Char('K') => {
                        if total > 0 {
                            if let Some(p) = sorted.get(self.selected) {
                                return Ok(InputResult::Kill(p.pid));
                            }
                        }
                    }

                    _ => {}
                }
            }
        }

        Ok(InputResult::Continue)
    }

    fn clamp_scroll(&mut self, total: usize) {
        let visible = self.visible_rows.max(1);
        if self.selected < self.scroll {
            self.scroll = self.selected;
        } else if self.selected >= self.scroll + visible {
            self.scroll = self.selected - visible + 1;
        }
        if self.scroll + visible > total {
            self.scroll = total.saturating_sub(visible);
        }
    }
}
