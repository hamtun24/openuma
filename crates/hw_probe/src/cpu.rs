use crate::errors::HwProbeError;
use crate::types::CpuProfile;
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
