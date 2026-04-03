pub mod linux;

#[derive(Debug, Clone)]
pub struct DmaBufBridge;

impl DmaBufBridge {
    pub fn new() -> Self {
        Self
    }

    pub fn export_fd(&self, _buffer: &mut [u8]) -> Result<u32, String> {
        #[cfg(target_os = "linux")]
        {
            linux::export_dmabuf(_buffer)
        }
        #[cfg(not(target_os = "linux"))]
        {
            Err("DMA-BUF not supported on this platform".to_string())
        }
    }

    pub fn import_fd(&self, _fd: u32) -> Result<Vec<u8>, String> {
        #[cfg(target_os = "linux")]
        {
            linux::import_dmabuf(_fd)
        }
        #[cfg(not(target_os = "linux"))]
        {
            Err("DMA-BUF not supported on this platform".to_string())
        }
    }
}

impl Default for DmaBufBridge {
    fn default() -> Self {
        Self::new()
    }
}
