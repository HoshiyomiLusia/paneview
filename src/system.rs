use std::collections::{BTreeMap, VecDeque};
use std::time::{Duration, Instant};

use if_addrs::get_if_addrs;
use sysinfo::{Disks, Networks, System};

const REFRESH_INTERVAL: Duration = Duration::from_millis(1_000);
const HISTORY_LIMIT: usize = 60;

#[derive(Debug, Clone)]
pub struct SystemSnapshot {
    pub cpu_usage: Option<f32>,
    pub memory_total: u64,
    pub memory_used: u64,
    pub memory_percent: Option<f32>,
    pub disks: Vec<DiskSnapshot>,
    pub interfaces: Vec<InterfaceSnapshot>,
    pub rx_per_sec: Option<f64>,
    pub tx_per_sec: Option<f64>,
    pub cpu_history: Vec<u64>,
    pub rx_history: Vec<u64>,
    pub tx_history: Vec<u64>,
    pub os_name: String,
    pub kernel_version: String,
    pub host_name: String,
    pub uptime_secs: u64,
}

impl Default for SystemSnapshot {
    fn default() -> Self {
        Self {
            cpu_usage: None,
            memory_total: 0,
            memory_used: 0,
            memory_percent: None,
            disks: Vec::new(),
            interfaces: Vec::new(),
            rx_per_sec: None,
            tx_per_sec: None,
            cpu_history: Vec::new(),
            rx_history: Vec::new(),
            tx_history: Vec::new(),
            os_name: "N/A".to_string(),
            kernel_version: "N/A".to_string(),
            host_name: "N/A".to_string(),
            uptime_secs: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiskSnapshot {
    pub mount: String,
    pub used: u64,
    pub total: u64,
    pub percent: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct InterfaceSnapshot {
    pub name: String,
    pub ips: Vec<String>,
    pub is_up: Option<bool>,
}

pub struct SystemMonitor {
    system: System,
    disks: Disks,
    networks: Networks,
    last_refresh: Instant,
    snapshot: SystemSnapshot,
    cpu_history: VecDeque<u64>,
    rx_history: VecDeque<u64>,
    tx_history: VecDeque<u64>,
}

impl SystemMonitor {
    pub fn new() -> Self {
        let mut monitor = Self {
            system: System::new_all(),
            disks: Disks::new_with_refreshed_list(),
            networks: Networks::new_with_refreshed_list(),
            last_refresh: Instant::now(),
            snapshot: SystemSnapshot::default(),
            cpu_history: VecDeque::with_capacity(HISTORY_LIMIT),
            rx_history: VecDeque::with_capacity(HISTORY_LIMIT),
            tx_history: VecDeque::with_capacity(HISTORY_LIMIT),
        };
        monitor.refresh_now();
        monitor
    }

    pub fn refresh_if_due(&mut self) {
        if self.last_refresh.elapsed() >= REFRESH_INTERVAL {
            self.refresh_now();
        }
    }

    pub fn snapshot(&self) -> &SystemSnapshot {
        &self.snapshot
    }

    fn refresh_now(&mut self) {
        let elapsed = self.last_refresh.elapsed();
        self.system.refresh_cpu_usage();
        self.system.refresh_memory();
        self.disks.refresh(true);
        self.networks.refresh(true);

        let mut snapshot = self.collect_snapshot(elapsed);
        push_history(
            &mut self.cpu_history,
            snapshot.cpu_usage.map(percent_to_history).unwrap_or(0),
        );
        push_history(
            &mut self.rx_history,
            snapshot.rx_per_sec.map(rate_to_history).unwrap_or(0),
        );
        push_history(
            &mut self.tx_history,
            snapshot.tx_per_sec.map(rate_to_history).unwrap_or(0),
        );
        snapshot.cpu_history = self.cpu_history.iter().copied().collect();
        snapshot.rx_history = self.rx_history.iter().copied().collect();
        snapshot.tx_history = self.tx_history.iter().copied().collect();
        self.snapshot = snapshot;
        self.last_refresh = Instant::now();
    }

    fn collect_snapshot(&self, elapsed: Duration) -> SystemSnapshot {
        let memory_total = self.system.total_memory();
        let memory_used = self.system.used_memory();
        let memory_percent = if memory_total > 0 {
            Some(memory_used as f32 / memory_total as f32 * 100.0)
        } else {
            None
        };

        let disks = self
            .disks
            .list()
            .iter()
            .map(|disk| {
                let total = disk.total_space();
                let available = disk.available_space();
                let used = total.saturating_sub(available);
                let percent = if total > 0 {
                    Some(used as f32 / total as f32 * 100.0)
                } else {
                    None
                };

                DiskSnapshot {
                    mount: disk.mount_point().display().to_string(),
                    used,
                    total,
                    percent,
                }
            })
            .collect();

        let (rx_per_sec, tx_per_sec) = network_rates(&self.networks, elapsed);

        SystemSnapshot {
            cpu_usage: Some(self.system.global_cpu_usage()),
            memory_total,
            memory_used,
            memory_percent,
            disks,
            interfaces: collect_interfaces(&self.networks),
            rx_per_sec,
            tx_per_sec,
            cpu_history: Vec::new(),
            rx_history: Vec::new(),
            tx_history: Vec::new(),
            os_name: System::long_os_version()
                .or_else(System::name)
                .unwrap_or_else(|| "N/A".to_string()),
            kernel_version: System::kernel_version().unwrap_or_else(|| "N/A".to_string()),
            host_name: System::host_name().unwrap_or_else(|| "N/A".to_string()),
            uptime_secs: System::uptime(),
        }
    }
}

fn push_history(history: &mut VecDeque<u64>, value: u64) {
    if history.len() == HISTORY_LIMIT {
        history.pop_front();
    }
    history.push_back(value);
}

fn percent_to_history(value: f32) -> u64 {
    value.clamp(0.0, 100.0).round() as u64
}

fn rate_to_history(value: f64) -> u64 {
    value.max(0.0).round() as u64
}

fn network_rates(networks: &Networks, elapsed: Duration) -> (Option<f64>, Option<f64>) {
    let seconds = elapsed.as_secs_f64();
    if seconds <= 0.0 {
        return (None, None);
    }

    let received: u64 = networks.values().map(|network| network.received()).sum();
    let transmitted: u64 = networks.values().map(|network| network.transmitted()).sum();
    (
        Some(received as f64 / seconds),
        Some(transmitted as f64 / seconds),
    )
}

fn collect_interfaces(networks: &Networks) -> Vec<InterfaceSnapshot> {
    let mut interfaces: BTreeMap<String, InterfaceSnapshot> = BTreeMap::new();

    if let Ok(addrs) = get_if_addrs() {
        for interface in addrs {
            let entry =
                interfaces
                    .entry(interface.name.clone())
                    .or_insert_with(|| InterfaceSnapshot {
                        name: interface.name.clone(),
                        ips: Vec::new(),
                        is_up: None,
                    });

            let ip = interface.ip().to_string();
            if !entry.ips.contains(&ip) {
                entry.ips.push(ip);
            }
            entry.is_up = Some(interface.is_oper_up());
        }
    }

    for name in networks.keys() {
        interfaces
            .entry(name.clone())
            .or_insert_with(|| InterfaceSnapshot {
                name: name.clone(),
                ips: Vec::new(),
                is_up: None,
            });
    }

    interfaces.into_values().collect()
}

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = 0;

    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }

    if unit == 0 {
        format!("{bytes} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

pub fn format_duration(secs: u64) -> String {
    let days = secs / 86_400;
    let hours = (secs % 86_400) / 3_600;
    let minutes = (secs % 3_600) / 60;

    if days > 0 {
        format!("{days}d {hours}h {minutes}m")
    } else if hours > 0 {
        format!("{hours}h {minutes}m")
    } else {
        format!("{minutes}m")
    }
}
