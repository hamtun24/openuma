use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DmaBufCapabilities {
    pub available: bool,
    pub unified_memory: bool,
    pub zero_copy_threshold_mb: u64,
}

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

pub fn detect_capabilities() -> DmaBufCapabilities {
    #[cfg(target_os = "linux")]
    {
        DmaBufCapabilities {
            available: linux::is_dmabuf_available(),
            unified_memory: linux::is_dmabuf_available(),
            zero_copy_threshold_mb: 64,
        }
    }
    #[cfg(target_os = "windows")]
    {
        windows::detect_capabilities()
    }
    #[cfg(target_os = "macos")]
    {
        macos::detect_capabilities()
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        DmaBufCapabilities::default()
    }
}

#[derive(Debug, Clone)]
pub struct DmaBufBridge;

impl DmaBufBridge {
    pub fn new() -> Self {
        Self
    }

    pub fn is_available(&self) -> bool {
        detect_capabilities().available
    }

    pub fn export_fd(&self, buffer: &mut [u8]) -> Result<u32, String> {
        #[cfg(target_os = "linux")]
        {
            linux::export_dmabuf(buffer)
        }
        #[cfg(target_os = "windows")]
        {
            windows::export_buffer(buffer)
        }
        #[cfg(target_os = "macos")]
        {
            macos::export_buffer(buffer)
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            Err("Zero-copy not supported on this platform".to_string())
        }
    }

    pub fn import_fd(&self, fd: u32) -> Result<Vec<u8>, String> {
        #[cfg(target_os = "linux")]
        {
            linux::import_dmabuf(fd)
        }
        #[cfg(target_os = "windows")]
        {
            windows::import_buffer(fd)
        }
        #[cfg(target_os = "macos")]
        {
            macos::import_buffer(fd)
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            Err("Zero-copy not supported on this platform".to_string())
        }
    }
}

impl Default for DmaBufBridge {
    fn default() -> Self {
        Self::new()
    }
}
