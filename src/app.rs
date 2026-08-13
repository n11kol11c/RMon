use std::collections::VecDeque;
use std::str::FromStr;
use std::time::Instant;

use ratatui::layout::Rect;
use ratatui::widgets::TableState;
use sysinfo::{Disks, Networks, Pid, System};

use crate::config::Config;
use crate::theme::{self, Theme};

pub const HISTORY_LEN: usize = 128;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SortKey {
    Cpu,
    Memory,
    Name,
    Pid,
}

impl SortKey {
    pub fn label(self) -> &'static str {
        match self {
            SortKey::Cpu => "CPU",
            SortKey::Memory => "MEM",
            SortKey::Name => "NAME",
            SortKey::Pid => "PID",
        }
    }

    pub fn is_numeric(self) -> bool {
        matches!(self, SortKey::Cpu | SortKey::Memory)
    }
}

impl FromStr for SortKey {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "cpu" => Ok(SortKey::Cpu),
            "memory" | "mem" => Ok(SortKey::Memory),
            "name" => Ok(SortKey::Name),
            "pid" => Ok(SortKey::Pid),
            _ => Err(()),
        }
    }
}

pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu: f32,
    pub mem: u64,
    pub status: String,
}

pub struct DiskInfo {
    pub name: String,
    pub mount: String,
    pub kind: String,
    pub fs: String,
    pub total: u64,
    pub used: u64,
}

pub struct App {
    pub system: System,
    pub disks: Disks,
    pub networks: Networks,
    pub theme: Theme,
    pub cpu_usage: f32,
    pub cores: Vec<f32>,
    pub cpu_history: VecDeque<u64>,
    pub memory_history: VecDeque<u64>,
    pub core_history: Vec<VecDeque<u64>>,
    pub disk_info: Vec<DiskInfo>,
    pub processes: Vec<ProcessInfo>,
    pub max_processes: usize,
    pub process_count: usize,
    pub process_state: TableState,
    pub table_area: Option<Rect>,
    pub sort: SortKey,
    pub sort_desc: bool,
    pub should_quit: bool,
    pub paused: bool,
    pub interval_ms: u64,
    pub confirm: Option<(u32, String)>,
    pub net_down_speed: u64,
    pub net_up_speed: u64,
    pub net_down_total: u64,
    pub net_up_total: u64,
    last_refresh: Instant,
    last_net_time: Instant,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self::with_config(Config::default())
    }

    pub fn with_config(cfg: Config) -> Self {
        let mut sort = SortKey::Cpu;
        let mut refresh_ms = 1000u64;
        let mut max_procs = 300usize;

        if let Some(file) = &cfg.file {
            if let Some(mon) = &file.monitor {
                if let Some(s) = &mon.sort {
                    if let Ok(k) = s.parse::<SortKey>() {
                        sort = k;
                    }
                }
                if let Some(ms) = mon.refresh_ms {
                    refresh_ms = ms;
                }
                if let Some(mp) = mon.max_processes {
                    max_procs = mp;
                }
            }
        }
        let sort_desc = sort.is_numeric();

        let theme = cfg
            .file
            .as_ref()
            .and_then(|f| f.theme.as_ref())
            .map(|t| theme::resolve(t.name.as_deref(), t.colors.as_ref()))
            .unwrap_or_else(theme::dark);

        if let Some(name) = cfg.file.as_ref().and_then(|f| f.theme.as_ref()).and_then(|t| t.name.as_deref()) {
            if theme::from_name(name).is_none() {
                eprintln!(
                    "rmon: unknown theme '{name}', using dark (available: {})",
                    theme::all_names()
                );
            }
        }

        let system = System::new_all();
        let disks = Disks::new_with_refreshed_list();
        let networks = Networks::new_with_refreshed_list();

        let mut app = App {
            system,
            disks,
            networks,
            theme,
            cpu_usage: 0.0,
            cores: Vec::new(),
            cpu_history: VecDeque::new(),
            memory_history: VecDeque::new(),
            core_history: Vec::new(),
            disk_info: Vec::new(),
            processes: Vec::new(),
            max_processes: max_procs.max(1),
            process_count: 0,
            process_state: TableState::default(),
            table_area: None,
            sort,
            sort_desc,
            should_quit: false,
            paused: false,
            interval_ms: refresh_ms.clamp(200, 5000),
            confirm: None,
            net_down_speed: 0,
            net_up_speed: 0,
            net_down_total: 0,
            net_up_total: 0,
            last_refresh: Instant::now(),
            last_net_time: Instant::now(),
        };

        // Two refreshes with a small delay so CPU usage has a meaningful
        // baseline on the very first frame.
        app.refresh();
        std::thread::sleep(std::time::Duration::from_millis(300));
        app.refresh();
        app
    }

    pub fn refresh(&mut self) {
        self.system.refresh_all();
        self.disks.refresh();
        self.networks.refresh();

        self.cpu_usage = self.system.global_cpu_usage();
        self.cores = self.system.cpus().iter().map(|c| c.cpu_usage()).collect();
        self.process_count = self.system.processes().len();

        self.update_history();
        self.update_disks();
        self.update_processes();
        self.update_network();

        self.last_refresh = Instant::now();
    }

    fn update_history(&mut self) {
        self.cpu_history
            .push_back((self.cpu_usage.clamp(0.0, 100.0) * 2.55) as u64);
        if self.cpu_history.len() > HISTORY_LEN {
            self.cpu_history.pop_front();
        }

        let mem_pct = if self.system.total_memory() > 0 {
            (self.system.used_memory() as f64 / self.system.total_memory() as f64 * 255.0) as u64
        } else {
            0
        };
        self.memory_history.push_back(mem_pct);
        if self.memory_history.len() > HISTORY_LEN {
            self.memory_history.pop_front();
        }

        if self.core_history.len() != self.cores.len() {
            self.core_history = self.cores.iter().map(|_| VecDeque::new()).collect();
        }
        for (i, hist) in self.core_history.iter_mut().enumerate() {
            let usage = self.cores.get(i).copied().unwrap_or(0.0).clamp(0.0, 100.0) as u64;
            hist.push_back(usage);
            if hist.len() > HISTORY_LEN {
                hist.pop_front();
            }
        }
    }

    fn update_disks(&mut self) {
        self.disk_info.clear();
        for disk in self.disks.list() {
            let total = disk.total_space();
            let available = disk.available_space();
            self.disk_info.push(DiskInfo {
                name: disk.name().to_string_lossy().into_owned(),
                mount: disk.mount_point().to_string_lossy().into_owned(),
                kind: disk.kind().to_string(),
                fs: disk.file_system().to_string_lossy().into_owned(),
                total,
                used: total.saturating_sub(available),
            });
        }
    }

    fn update_network(&mut self) {
        let down = self
            .networks
            .list()
            .values()
            .map(|n| n.received())
            .sum::<u64>();
        let up = self
            .networks
            .list()
            .values()
            .map(|n| n.transmitted())
            .sum::<u64>();

        let now = Instant::now();
        let secs = now.duration_since(self.last_net_time).as_secs_f64();
        self.last_net_time = now;
        self.net_down_speed = if secs > 0.0 { (down as f64 / secs) as u64 } else { 0 };
        self.net_up_speed = if secs > 0.0 { (up as f64 / secs) as u64 } else { 0 };
        self.net_down_total = self
            .networks
            .list()
            .values()
            .map(|n| n.total_received())
            .sum();
        self.net_up_total = self
            .networks
            .list()
            .values()
            .map(|n| n.total_transmitted())
            .sum();
    }

    fn update_processes(&mut self) {
        let mut list: Vec<ProcessInfo> = self
            .system
            .processes()
            .iter()
            .map(|(pid, p)| ProcessInfo {
                pid: pid.as_u32(),
                name: p.name().to_string_lossy().into_owned(),
                cpu: p.cpu_usage(),
                mem: p.memory(),
                status: p.status().to_string(),
            })
            .collect();

        sort_processes(&mut list, self.sort, self.sort_desc);

        self.processes = list.into_iter().take(self.max_processes).collect();

        let selected = self.process_state.selected().unwrap_or(0);
        if !self.processes.is_empty() {
            self.process_state
                .select(Some(selected.min(self.processes.len() - 1)));
        } else {
            self.process_state.select(None);
        }
    }

    pub fn move_selection(&mut self, delta: isize) {
        if self.processes.is_empty() {
            return;
        }
        let n = self.processes.len() as isize;
        let cur = self.process_state.selected().unwrap_or(0) as isize;
        let next = (cur + delta).clamp(0, n - 1);
        self.process_state.select(Some(next as usize));
    }

    pub fn jump_to_first(&mut self) {
        if !self.processes.is_empty() {
            self.process_state.select(Some(0));
        }
    }

    pub fn jump_to_last(&mut self) {
        if !self.processes.is_empty() {
            self.process_state.select(Some(self.processes.len() - 1));
        }
    }

    pub fn request_kill(&mut self) {
        if let Some(idx) = self.process_state.selected() {
            if let Some(p) = self.processes.get(idx) {
                self.confirm = Some((p.pid, p.name.clone()));
            }
        }
    }

    pub fn confirm_kill(&mut self) {
        if let Some((pid, _)) = self.confirm.take() {
            if let Some(process) = self.system.process(Pid::from_u32(pid)) {
                process.kill();
            }
            self.refresh();
        }
    }

    pub fn cancel_kill(&mut self) {
        self.confirm = None;
    }

    pub fn cycle_sort(&mut self) {
        self.sort = match self.sort {
            SortKey::Cpu => SortKey::Memory,
            SortKey::Memory => SortKey::Name,
            SortKey::Name => SortKey::Pid,
            SortKey::Pid => SortKey::Cpu,
        };
        self.sort_desc = self.sort.is_numeric();
        self.update_processes();
    }

    pub fn speed_up(&mut self) {
        self.interval_ms = (self.interval_ms / 2).clamp(200, 5000);
    }

    pub fn speed_down(&mut self) {
        self.interval_ms = (self.interval_ms * 2).clamp(200, 5000);
    }
}

fn sort_processes(list: &mut [ProcessInfo], sort: SortKey, desc: bool) {
    list.sort_by(|a, b| match sort {
        SortKey::Cpu => a.cpu.partial_cmp(&b.cpu).unwrap_or(std::cmp::Ordering::Equal),
        SortKey::Memory => a.mem.cmp(&b.mem),
        SortKey::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        SortKey::Pid => a.pid.cmp(&b.pid),
    });
    if desc {
        list.reverse();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<ProcessInfo> {
        vec![
            ProcessInfo { pid: 3, name: "Zebra".into(), cpu: 10.0, mem: 100, status: "R".into() },
            ProcessInfo { pid: 1, name: "alpha".into(), cpu: 90.0, mem: 900, status: "S".into() },
            ProcessInfo { pid: 2, name: "Beta".into(), cpu: 50.0, mem: 500, status: "T".into() },
        ]
    }

    fn pids(list: &[ProcessInfo]) -> Vec<u32> {
        list.iter().map(|p| p.pid).collect()
    }

    #[test]
    fn cpu_sorted_descending_by_default() {
        let mut list = sample();
        sort_processes(&mut list, SortKey::Cpu, true);
        assert_eq!(pids(&list), vec![1, 2, 3]);
    }

    #[test]
    fn memory_sorted_descending() {
        let mut list = sample();
        sort_processes(&mut list, SortKey::Memory, true);
        assert_eq!(pids(&list), vec![1, 2, 3]);
    }

    #[test]
    fn name_sorted_ascending_case_insensitive() {
        let mut list = sample();
        sort_processes(&mut list, SortKey::Name, false);
        assert_eq!(pids(&list), vec![1, 2, 3]);
    }

    #[test]
    fn pid_sorted_ascending() {
        let mut list = sample();
        sort_processes(&mut list, SortKey::Pid, false);
        assert_eq!(pids(&list), vec![1, 2, 3]);
    }

    #[test]
    fn ascending_flips_order() {
        let mut list = sample();
        sort_processes(&mut list, SortKey::Cpu, false);
        assert_eq!(pids(&list), vec![3, 2, 1]);
    }

    #[test]
    fn cycle_sort_toggles_descending_only_for_numeric_keys() {
        let mut app = App {
            sort: SortKey::Cpu,
            sort_desc: true,
            ..Default::default()
        };
        app.cycle_sort();
        assert_eq!(app.sort, SortKey::Memory);
        assert!(app.sort_desc);
        app.cycle_sort();
        assert_eq!(app.sort, SortKey::Name);
        assert!(!app.sort_desc);
        app.cycle_sort();
        assert_eq!(app.sort, SortKey::Pid);
        assert!(!app.sort_desc);
        app.cycle_sort();
        assert_eq!(app.sort, SortKey::Cpu);
        assert!(app.sort_desc);
    }

    #[test]
    fn sort_key_parsing() {
        assert_eq!("cpu".parse(), Ok(SortKey::Cpu));
        assert_eq!("MEM".parse(), Ok(SortKey::Memory));
        assert_eq!("name".parse(), Ok(SortKey::Name));
        assert_eq!("pid".parse(), Ok(SortKey::Pid));
        assert_eq!("bogus".parse::<SortKey>(), Err(()));
        assert!(SortKey::Cpu.is_numeric());
        assert!(!SortKey::Name.is_numeric());
    }

    #[test]
    fn config_applies_sort_refresh_and_theme() {
        use crate::config::{Config, ConfigFile, MonitorSection, ThemeSection};

        let cfg = Config {
            file: Some(ConfigFile {
                theme: Some(ThemeSection {
                    name: Some("nord".into()),
                    colors: None,
                }),
                monitor: Some(MonitorSection {
                    refresh_ms: Some(250),
                    sort: Some("name".into()),
                    max_processes: Some(50),
                }),
            }),
            path: None,
        };
        let app = App::with_config(cfg);
        assert_eq!(app.sort, SortKey::Name);
        assert!(!app.sort_desc);
        assert_eq!(app.interval_ms, 250);
        assert_eq!(app.max_processes, 50);
        assert_eq!(app.theme.name, "nord");
    }
}
