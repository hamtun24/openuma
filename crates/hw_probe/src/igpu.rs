use crate::errors::HwProbeError;
use crate::types::IgpuProfile;
use std::process::Command;

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
