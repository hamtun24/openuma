use crate::errors::HwProbeError;
use crate::types::{CpuProfile, IgpuProfile, PlatformProfile, RamProfile};
use std::process::Command;

pub fn probe_cpu() -> Result<CpuProfile, HwProbeError> {
    let mut cpu = CpuProfile {
        model: String::new(),
        vendor: String::new(),
        cores: 0,
        threads: 0,
        frequency_mhz: 0,
        flags: Vec::new(),
    };

    let cpuinfo = std::fs::read_to_string("/proc/cpuinfo").map_err(HwProbeError::Io)?;

    for line in cpuinfo.lines() {
        if line.starts_with("model name") {
            cpu.model = line
                .split(':')
                .nth(1)
                .map(|s| s.trim().to_string())
                .unwrap_or_default();
        } else if line.starts_with("vendor_id") {
            cpu.vendor = line
                .split(':')
                .nth(1)
                .map(|s| s.trim().to_string())
                .unwrap_or_default();
        } else if line.starts_with("flags") {
            cpu.flags = line
                .split(':')
                .nth(1)
                .map(|s| s.split_whitespace().map(String::from).collect())
                .unwrap_or_default();
        }
    }

    if let Ok(output) = Command::new("lscpu").output() {
        let lscpu = String::from_utf8_lossy(&output.stdout);
        for line in lscpu.lines() {
            if line.starts_with("CPU(s)") {
                cpu.cores = line
                    .split(':')
                    .nth(1)
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or(0);
            } else if line.starts_with("Thread(s)") {
                cpu.threads = line
                    .split(':')
                    .nth(1)
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or(1);
            } else if line.starts_with("CPU MHz") {
                cpu.frequency_mhz = line
                    .split(':')
                    .nth(1)
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or(0);
            } else if line.starts_with("Model name") && cpu.model.is_empty() {
                cpu.model = line
                    .split(':')
                    .nth(1)
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
            }
        }
    }

    if cpu.threads == 0 {
        cpu.threads = cpu.cores;
    }

    Ok(cpu)
}

pub fn probe_ram() -> Result<RamProfile, HwProbeError> {
    let meminfo = std::fs::read_to_string("/proc/meminfo").map_err(HwProbeError::Io)?;

    let mut total = 0u64;
    let mut available = 0u64;
    let mut swap_total = 0u64;
    let mut swap_free = 0u64;

    for line in meminfo.lines() {
        if line.starts_with("MemTotal:") {
            total = parse_meminfo_line(line);
        } else if line.starts_with("MemAvailable:") {
            available = parse_meminfo_line(line);
        } else if line.starts_with("SwapTotal:") {
            swap_total = parse_meminfo_line(line);
        } else if line.starts_with("SwapFree:") {
            swap_free = parse_meminfo_line(line);
        }
    }

    Ok(RamProfile {
        total_bytes: total * 1024,
        available_bytes: available * 1024,
        swap_total_bytes: swap_total * 1024,
        swap_free_bytes: swap_free * 1024,
    })
}

fn parse_meminfo_line(line: &str) -> u64 {
    line.split(':')
        .nth(1)
        .and_then(|s| s.split_whitespace().next())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

pub fn probe_igpu() -> Result<Option<IgpuProfile>, HwProbeError> {
    let output = Command::new("lspci")
        .args(["-vmm", "-d", "::0300"])
        .output()
        .map_err(|e| HwProbeError::Command(e.to_string()))?;

    let lspci = String::from_utf8_lossy(&output.stdout);

    let mut igpu = None;
    let mut current_name = String::new();
    let mut current_vendor = String::new();
    let mut current_driver = String::new();

    for line in lspci.lines() {
        if line.starts_with("Slot:") {
            current_name.clear();
            current_vendor.clear();
            current_driver.clear();
        } else if line.starts_with("Device:") {
            current_name = line
                .split(':')
                .nth(1)
                .map(|s| s.trim().to_string())
                .unwrap_or_default();
        } else if line.starts_with("Vendor:") {
            current_vendor = line
                .split(':')
                .nth(1)
                .map(|s| s.trim().to_string())
                .unwrap_or_default();
        } else if line.starts_with("Driver:") {
            current_driver = line
                .split(':')
                .nth(1)
                .map(|s| s.trim().to_string())
                .unwrap_or_default();
            if !current_name.is_empty() {
                igpu = Some(IgpuProfile {
                    name: current_name.clone(),
                    vendor: current_vendor.clone(),
                    driver: current_driver.clone(),
                    memory_mb: None,
                });
                break;
            }
        }
    }

    Ok(igpu)
}

pub fn probe_platform() -> Result<PlatformProfile, HwProbeError> {
    let os_version = std::fs::read_to_string("/etc/os-release")
        .map(|s| {
            s.lines()
                .find(|l| l.starts_with("PRETTY_NAME"))
                .or_else(|| s.lines().find(|l| l.starts_with("NAME")))
                .map(|l| {
                    l.split('=')
                        .nth(1)
                        .unwrap_or("")
                        .trim_matches('"')
                        .to_string()
                })
                .unwrap_or_else(|| "Linux".to_string())
        })
        .unwrap_or_else(|_| "Linux".to_string());

    let kernel = std::fs::read_to_string("/proc/version")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| String::new());

    let compute_backend = detect_compute_backend();

    Ok(PlatformProfile {
        os: "Linux".to_string(),
        os_version,
        kernel,
        compute_backend,
    })
}

fn detect_compute_backend() -> String {
    if std::path::Path::new("/dev/dri").exists() {
        return "OpenCL".to_string();
    }
    if Command::new("nvidia-smi")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return "CUDA".to_string();
    }
    "CPU".to_string()
}
