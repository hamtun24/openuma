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

    if let Ok(output) = Command::new("sysctl")
        .args(["-n", "machdep.cpu.brand_string"])
        .output()
    {
        cpu.model = String::from_utf8_lossy(&output.stdout).trim().to_string();
    }

    if let Ok(output) = Command::new("sysctl").args(["-n", "hw.ncpu"]).output() {
        if let Ok(s) = String::from_utf8(output.stdout) {
            cpu.threads = s.trim().parse().unwrap_or(0);
        }
    }

    if let Ok(output) = Command::new("sysctl")
        .args(["-n", "hw.physicalcpu"])
        .output()
    {
        if let Ok(s) = String::from_utf8(output.stdout) {
            cpu.cores = s.trim().parse().unwrap_or(0);
        }
    }

    if let Ok(output) = Command::new("sysctl")
        .args(["-n", "hw.cpufrequency"])
        .output()
    {
        if let Ok(s) = String::from_utf8(output.stdout) {
            cpu.frequency_mhz = s.trim().parse::<u64>().unwrap_or(0) / 1_000_000;
        }
    }

    if cpu.cores == 0 {
        cpu.cores = cpu.threads;
    }

    if cpu.threads == 0 {
        cpu.threads = cpu.cores;
    }

    let vendor = if cpu.model.to_lowercase().contains("intel") {
        "Intel"
    } else if cpu.model.to_lowercase().contains("apple") {
        "Apple"
    } else if cpu.model.to_lowercase().contains("amd") {
        "AMD"
    } else {
        "Unknown"
    };
    cpu.vendor = vendor.to_string();

    Ok(cpu)
}

pub fn probe_ram() -> Result<RamProfile, HwProbeError> {
    let output = Command::new("sysctl")
        .args(["-n", "hw.memsize"])
        .output()
        .map_err(|e| HwProbeError::Command(e.to_string()))?;

    let total = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<u64>()
        .unwrap_or(0);

    let output = Command::new("sysctl").args(["-n", "hw.usermem"]).output();

    let available = output
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(total);

    Ok(RamProfile {
        total_bytes: total,
        available_bytes: available,
        swap_total_bytes: 0,
        swap_free_bytes: 0,
    })
}

pub fn probe_igpu() -> Result<Option<IgpuProfile>, HwProbeError> {
    let output = Command::new("system_profiler")
        .args(["SPDisplaysDataType", "-json"])
        .output()
        .map_err(|e| HwProbeError::Command(e.to_string()))?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut igpu = None;

    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
        if let Some(displays) = json.get("SPDisplaysDataType").and_then(|v| v.as_array()) {
            for display in displays {
                let name = display
                    .get("sppci_model")
                    .and_then(|v| v.as_str())
                    .or_else(|| display.get("_name").and_then(|v| v.as_str()))
                    .unwrap_or("Unknown")
                    .to_string();

                let vendor = display
                    .get("spdisplays_vendor")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string();

                let memory_str = display
                    .get("spdisplays_memory")
                    .and_then(|v| v.as_str())
                    .unwrap_or("0 MB");

                let memory_mb = parse_macos_memory(memory_str);

                let driver = display
                    .get("kCGDriverName")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Metal")
                    .to_string();

                let is_integrated = name.to_lowercase().contains("intel")
                    || name.to_lowercase().contains("apple")
                    || (vendor.to_lowercase().contains("intel")
                        && !name.to_lowercase().contains("discrete"));

                if is_integrated || igpu.is_none() {
                    igpu = Some(IgpuProfile {
                        name,
                        vendor,
                        driver,
                        memory_mb: Some(memory_mb),
                    });

                    if is_integrated {
                        break;
                    }
                }
            }
        }
    }

    Ok(igpu)
}

fn parse_macos_memory(s: &str) -> u64 {
    let s = s.trim();
    if s.ends_with("GB") {
        s.trim_end_matches("GB")
            .trim()
            .parse::<f64>()
            .map(|v| (v * 1024.0) as u64)
            .unwrap_or(0)
    } else if s.ends_with("MB") {
        s.trim_end_matches("MB").trim().parse::<u64>().unwrap_or(0)
    } else {
        0
    }
}

pub fn probe_platform() -> Result<PlatformProfile, HwProbeError> {
    let mut os_version = String::new();
    let mut kernel = String::new();

    if let Ok(output) = Command::new("sw_vers").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.starts_with("ProductName:") {
                os_version = line.replace("ProductName:", "").trim().to_string();
            } else if line.starts_with("ProductVersion:") {
                let version = line.replace("ProductVersion:", "").trim().to_string();
                if !os_version.is_empty() {
                    os_version = format!("{} {}", os_version, version);
                } else {
                    os_version = version;
                }
            }
        }
    }

    if let Ok(output) = Command::new("uname").args(["-r"]).output() {
        kernel = String::from_utf8_lossy(&output.stdout).trim().to_string();
    }

    let compute_backend = detect_compute_backend();

    Ok(PlatformProfile {
        os: "macOS".to_string(),
        os_version,
        kernel,
        compute_backend,
    })
}

fn detect_compute_backend() -> String {
    let output = Command::new("system_profiler")
        .args(["SPDisplaysDataType", "-json"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
            if let Some(displays) = json.get("SPDisplaysDataType").and_then(|v| v.as_array()) {
                for display in displays {
                    let name = display
                        .get("sppci_model")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_lowercase();
                    let vendor = display
                        .get("spdisplays_vendor")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_lowercase();

                    if name.contains("apple") || vendor.contains("apple") {
                        return "Metal".to_string();
                    }
                    if name.contains("intel") || vendor.contains("intel") {
                        return "Metal".to_string();
                    }
                    if name.contains("amd") || name.contains("radeon") || vendor.contains("amd") {
                        return "Metal".to_string();
                    }
                }
            }
        }
    }

    "CPU".to_string()
}
