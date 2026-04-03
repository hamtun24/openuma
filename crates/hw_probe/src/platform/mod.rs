#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

use crate::errors::HwProbeError;
use crate::types::{CpuProfile, IgpuProfile, PlatformProfile, RamProfile};

pub fn probe_cpu() -> Result<CpuProfile, HwProbeError> {
    #[cfg(target_os = "linux")]
    {
        linux::probe_cpu()
    }
    #[cfg(target_os = "windows")]
    {
        windows::probe_cpu()
    }
    #[cfg(target_os = "macos")]
    {
        macos::probe_cpu()
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        Err(HwProbeError::Unsupported("Unsupported platform".to_string()))
    }
}

pub fn probe_ram() -> Result<RamProfile, HwProbeError> {
    #[cfg(target_os = "linux")]
    {
        linux::probe_ram()
    }
    #[cfg(target_os = "windows")]
    {
        windows::probe_ram()
    }
    #[cfg(target_os = "macos")]
    {
        macos::probe_ram()
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        Err(HwProbeError::Unsupported("Unsupported platform".to_string()))
    }
}

pub fn probe_igpu() -> Result<Option<IgpuProfile>, HwProbeError> {
    #[cfg(target_os = "linux")]
    {
        linux::probe_igpu()
    }
    #[cfg(target_os = "windows")]
    {
        windows::probe_igpu()
    }
    #[cfg(target_os = "macos")]
    {
        macos::probe_igpu()
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        Err(HwProbeError::Unsupported("Unsupported platform".to_string()))
    }
}

pub fn probe_platform() -> Result<PlatformProfile, HwProbeError> {
    #[cfg(target_os = "linux")]
    {
        linux::probe_platform()
    }
    #[cfg(target_os = "windows")]
    {
        windows::probe_platform()
    }
    #[cfg(target_os = "macos")]
    {
        macos::probe_platform()
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        Err(HwProbeError::Unsupported("Unsupported platform".to_string()))
    }
}
