use crate::errors::HwProbeError;
use crate::types::PlatformProfile;
use std::process::Command;

pub fn probe_platform() -> Result<PlatformProfile, HwProbeError> {
    let os = if cfg!(target_os = "linux") {
        "Linux"
    } else if cfg!(target_os = "windows") {
        "Windows"
    } else if cfg!(target_os = "macos") {
        "macOS"
    } else {
        "Unknown"
    };

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
                .unwrap_or_else(|| os.to_string())
        })
        .unwrap_or_else(|_| os.to_string());

    let kernel = std::fs::read_to_string("/proc/version")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| String::new());

    let compute_backend = detect_compute_backend();

    Ok(PlatformProfile {
        os: os.to_string(),
        os_version,
        kernel,
        compute_backend,
    })
}

fn detect_compute_backend() -> String {
    if cfg!(target_os = "linux") {
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
    }
    "CPU".to_string()
}
