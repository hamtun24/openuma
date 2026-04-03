use crate::errors::HwProbeError;
use crate::types::RamProfile;
use std::fs;

pub fn probe_ram() -> Result<RamProfile, HwProbeError> {
    let meminfo = fs::read_to_string("/proc/meminfo").map_err(HwProbeError::Io)?;

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
