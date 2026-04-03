use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum ModelArchitecture {
    #[default]
    Standard,
    MoE,
    Mamba,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UMAPartition {
    pub host_memory_mb: u64,
    pub device_memory_mb: u64,
    pub shared_memory_mb: u64,
    pub attention_layers_gpu: u32,
    pub expert_layers_cpu: u32,
    pub kv_cache_mb: u64,
    pub architecture: ModelArchitecture,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionConfig {
    pub model_size_mb: u64,
    pub model_architecture: ModelArchitecture,
    pub ctx_size: u32,
    pub batch_size: u32,
    pub num_layers: u32,
    pub hidden_size: u32,
    pub num_experts: Option<u32>,
    pub num_active_experts: Option<u32>,
}

impl Default for PartitionConfig {
    fn default() -> Self {
        Self {
            model_size_mb: 4000,
            model_architecture: ModelArchitecture::Standard,
            ctx_size: 4096,
            batch_size: 512,
            num_layers: 32,
            hidden_size: 4096,
            num_experts: None,
            num_active_experts: None,
        }
    }
}

pub fn compute_partition(
    total_memory_mb: u64,
    device_memory_mb: u64,
    config: Option<PartitionConfig>,
) -> UMAPartition {
    let config = config.unwrap_or_default();

    let kv_cache_mb = estimate_kv_cache_mb(
        config.ctx_size,
        config.batch_size,
        config.num_layers,
        config.hidden_size,
    );

    let usable_device = device_memory_mb.saturating_sub(kv_cache_mb / 2);

    let (attention_layers_gpu, expert_layers_cpu) = match config.model_architecture {
        ModelArchitecture::MoE => {
            let num_experts = config.num_experts.unwrap_or(8);

            let attention_per_expert = config.hidden_size / num_experts;
            let expert_mb_per_layer =
                (attention_per_expert * config.hidden_size * 4) as u64 / (1024 * 1024);
            let expert_mb_total = expert_mb_per_layer * config.num_layers as u64;

            let gpu_can_fit = usable_device * 8 / 10;
            let experts_on_gpu =
                ((gpu_can_fit / expert_mb_total.max(1)) as u32).min(config.num_layers / 2);

            (config.num_layers - experts_on_gpu, experts_on_gpu)
        }
        ModelArchitecture::Mamba => (config.num_layers, 0),
        ModelArchitecture::Standard => {
            let model_mb = config.model_size_mb;
            let attention_per_layer = model_mb / config.num_layers as u64;
            let gpu_can_fit = usable_device * 8 / 10;
            let layers_on_gpu =
                ((gpu_can_fit / attention_per_layer.max(1)) as u32).min(config.num_layers);
            (layers_on_gpu, 0)
        }
    };

    let host_memory = if device_memory_mb > 0 && device_memory_mb < total_memory_mb {
        total_memory_mb - device_memory_mb
    } else {
        total_memory_mb * 80 / 100
    };

    let shared = total_memory_mb - host_memory;

    UMAPartition {
        host_memory_mb: host_memory,
        device_memory_mb,
        shared_memory_mb: shared,
        attention_layers_gpu,
        expert_layers_cpu,
        kv_cache_mb,
        architecture: config.model_architecture,
    }
}

fn estimate_kv_cache_mb(ctx_size: u32, batch_size: u32, num_layers: u32, hidden_size: u32) -> u64 {
    let bytes_per_token = num_layers as u64 * hidden_size as u64 * 4 * 2;
    let total_tokens = (ctx_size as u64) * batch_size as u64;
    (bytes_per_token * total_tokens) / (1024 * 1024)
}

pub fn detect_architecture(model_name: &str) -> ModelArchitecture {
    let lower = model_name.to_lowercase();
    if lower.contains("mixtral")
        || lower.contains("qwen") && lower.contains("moe")
        || lower.contains("deepseek") && lower.contains("v3")
        || lower.contains("starcoder") && lower.contains("moe")
        || lower.contains("llama") && lower.contains("-70b")
    {
        ModelArchitecture::MoE
    } else if lower.contains("mamba") || lower.contains("jamba") {
        ModelArchitecture::Mamba
    } else {
        ModelArchitecture::Standard
    }
}
