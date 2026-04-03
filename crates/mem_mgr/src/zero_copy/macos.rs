#[derive(Debug, Clone, Default)]
pub struct DmaBufCapabilities {
    pub available: bool,
    pub unified_memory: bool,
    pub zero_copy_threshold_mb: u64,
}

pub fn detect_capabilities() -> DmaBufCapabilities {
    DmaBufCapabilities {
        available: true,
        unified_memory: true,
        zero_copy_threshold_mb: 512,
    }
}

pub fn export_buffer(_buffer: &mut [u8]) -> Result<u32, String> {
    Err("macOS shared memory export not yet implemented".to_string())
}

pub fn import_buffer(_fd: u32) -> Result<Vec<u8>, String> {
    Err("macOS shared memory import not yet implemented".to_string())
}
