use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UMAPartition {
    pub host_memory_mb: u64,
    pub device_memory_mb: u64,
    pub shared_memory_mb: u64,
}

pub fn compute_partition(total_memory_mb: u64, device_memory_mb: u64) -> UMAPartition {
    let host_memory = if device_memory_mb > 0 && device_memory_mb < total_memory_mb {
        total_memory_mb - device_memory_mb
    } else {
        total_memory_mb * 80 / 100
    };

    let shared = total_memory_mb - host_memory;

    UMAPartition {
        host_memory_mb: host_memory,
        device_memory_mb: device_memory_mb,
        shared_memory_mb: shared,
    }
}
