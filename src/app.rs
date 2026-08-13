use std::collections::VecDeque;
use std::time::Instant;

use ratatui::widgets::TableState;
use sysinfo::{Disks, Pid, System};

pub const HISTORY_LEN: usize = 128;
const MAX_PROCESSES: usize = 300;

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
    pub cpu_usage: f32,
    pub cores: Vec<f32>,
    pub cpu_history: VecDeque<u64>,
    pub memory_history: VecDeque<u64>,
    pub disk_info: Vec<DiskInfo>,
    pub processes: Vec<ProcessInfo>,
    pub process_count: usize,
    pub process_state: TableState,
    pub sort: SortKey,
    pub sort_desc: bool,
    pub should_quit: bool,
    pub paused: bool,
    pub interval_ms: u64,
    pub confirm: Option<(u32, String)>,
    last_refresh: Instant,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let system = System::new_all();
        let disks = Disks::new_with_refreshed_list();

        let mut app = App {
            system,
            disks,
            cpu_usage: 0.0,
            cores: Vec::new(),
            cpu_history: VecDeque::new(),
            memory_history: VecDeque::new(),
            disk_info: Vec::new(),
            processes: Vec::new(),
            process_count: 0,
            process_state: TableState::default(),
            sort: SortKey::Cpu,
            sort_desc: true,
            should_quit: false,
            paused: false,
            interval_ms: 1000,
            confirm: None,
            last_refresh: Instant::now(),
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

        self.cpu_usage = self.system.global_cpu_usage();
        self.cores = self.system.cpus().iter().map(|c| c.cpu_usage()).collect();
        self.process_count = self.system.processes().len();

        self.update_history();
        self.update_disks();
        self.update_processes();

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

        self.processes = list.into_iter().take(MAX_PROCESSES).collect();

        let selected = self.process_state.selected().unwrap_or(0);
        if !self.processes.is_empty() {
            self.process_state
                .select(Some(selected.min(self.processes.len() - 1)));
        } else {
            self.process_state.select(None);
        }
    }

    pub fn move_selection(&mut self, delta: isize) {        if self.processes.is_empty() {
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
        self.sort_desc = !matches!(self.sort, SortKey::Name | SortKey::Pid);
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
}
