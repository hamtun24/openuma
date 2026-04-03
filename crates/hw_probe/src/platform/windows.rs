use crate::errors::HwProbeError;
use crate::types::{CpuProfile, IgpuProfile, PlatformProfile, RamProfile};
use std::process::Command;

pub fn probe_cpu() -> Result<CpuProfile, HwProbeError> {
    let output = Command::new("wmic")
        .args([
            "cpu",
            "get",
            "Name,Manufacturer,NumberOfCores,NumberOfLogicalProcessors,MaxClockSpeed",
            "/format:csv",
        ])
        .output()
        .map_err(|e| HwProbeError::Command(e.to_string()))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut cpu = CpuProfile {
        model: String::new(),
        vendor: String::new(),
        cores: 0,
        threads: 0,
        frequency_mhz: 0,
        flags: Vec::new(),
    };

    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 5 {
            cpu.model = parts.get(1).unwrap_or(&"").trim().to_string();
            cpu.vendor = parts.get(2).unwrap_or(&"").trim().to_string();
            cpu.cores = parts.get(3).unwrap_or(&"0").trim().parse().unwrap_or(0);
            cpu.threads = parts.get(4).unwrap_or(&"0").trim().parse().unwrap_or(0);
            cpu.frequency_mhz = parts.get(5).unwrap_or(&"0").trim().parse().unwrap_or(0);
        }
    }

    if cpu.threads == 0 {
        cpu.threads = cpu.cores;
    }

    Ok(cpu)
}

pub fn probe_ram() -> Result<RamProfile, HwProbeError> {
    let output = Command::new("wmic")
        .args([
            "OS",
            "get",
            "TotalVisibleMemorySize,FreePhysicalMemory,TotalVirtualMemorySize,FreeVirtualMemory",
            "/format:csv",
        ])
        .output()
        .map_err(|e| HwProbeError::Command(e.to_string()))?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut total = 0u64;
    let mut available = 0u64;
    let mut swap_total = 0u64;
    let mut swap_free = 0u64;

    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 4 {
            total = parts.get(1).unwrap_or(&"0").trim().parse().unwrap_or(0);
            available = parts.get(2).unwrap_or(&"0").trim().parse().unwrap_or(0);
            swap_total = parts.get(3).unwrap_or(&"0").trim().parse().unwrap_or(0);
            swap_free = parts.get(4).unwrap_or(&"0").trim().parse().unwrap_or(0);
        }
    }

    Ok(RamProfile {
        total_bytes: total * 1024,
        available_bytes: available * 1024,
        swap_total_bytes: swap_total * 1024,
        swap_free_bytes: swap_free * 1024,
    })
}

pub fn probe_igpu() -> Result<Option<IgpuProfile>, HwProbeError> {
    let output = Command::new("wmic")
        .args([
            "path",
            "Win32_VideoController",
            "get",
            "Name,AdapterRAM,DriverVersion",
            "/format:csv",
        ])
        .output()
        .map_err(|e| HwProbeError::Command(e.to_string()))?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut igpu = None;

    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 3 {
            let name = parts.get(1).unwrap_or(&"").trim().to_string();
            let memory_str = parts.get(2).unwrap_or(&"0").trim();
            let driver = parts.get(3).unwrap_or(&"").trim().to_string();

            if name.is_empty() {
                continue;
            }

            let memory_mb = memory_str.parse::<u64>().ok().map(|m| m / (1024 * 1024));

            let is_igpu = name.to_lowercase().contains("intel")
                || name.to_lowercase().contains("amd")
                || name.to_lowercase().contains("radeon")
                || name.to_lowercase().contains("integrated");

            if is_igpu {
                igpu = Some(IgpuProfile {
                    name,
                    vendor: detect_vendor_from_name(&name),
                    driver,
                    memory_mb,
                });
                break;
            } else if igpu.is_none() {
                igpu = Some(IgpuProfile {
                    name,
                    vendor: detect_vendor_from_name(&name),
                    driver,
                    memory_mb,
                });
            }
        }
    }

    Ok(igpu)
}

fn detect_vendor_from_name(name: &str) -> String {
    let lower = name.to_lowercase();
    if lower.contains("intel") {
        "Intel".to_string()
    } else if lower.contains("amd") || lower.contains("radeon") {
        "AMD".to_string()
    } else if lower.contains("nvidia") || lower.contains("geforce") {
        "NVIDIA".to_string()
    } else {
        "Unknown".to_string()
    }
}

pub fn probe_platform() -> Result<PlatformProfile, HwProbeError> {
    let output = Command::new("systeminfo")
        .output()
        .map_err(|e| HwProbeError::Command(e.to_string()))?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut os_version = "Windows".to_string();
    let mut kernel = String::new();

    for line in stdout.lines() {
        if line.starts_with("OS Name:") {
            os_version = line.replace("OS Name:", "").trim().to_string();
        } else if line.starts_with("OS Version:") {
            let version = line.replace("OS Version:", "").trim().to_string();
            if !version.is_empty() {
                os_version = format!("{} {}", os_version, version);
            }
        } else if line.starts_with("OS Configuration:") {
            kernel = line.replace("OS Configuration:", "").trim().to_string();
        }
    }

    let compute_backend = detect_compute_backend();

    Ok(PlatformProfile {
        os: "Windows".to_string(),
        os_version,
        kernel,
        compute_backend,
    })
}

fn detect_compute_backend() -> String {
    if Command::new("nvidia-smi")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return "CUDA".to_string();
    }

    let output = Command::new("wmic")
        .args([
            "path",
            "Win32_VideoController",
            "get",
            "Name",
            "/format:csv",
        ])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines().skip(1) {
            let name = line.split(',').nth(1).unwrap_or("").to_lowercase();
            if name.contains("intel") || name.contains("amd") || name.contains("radeon") {
                return "DirectX".to_string();
            }
        }
    }

    "CPU".to_string()
}
