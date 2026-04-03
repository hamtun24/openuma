use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlamaCppConfig {
    pub n_gpu_layers: u32,
    pub n_threads: u32,
    pub n_threads_batch: u32,
    pub ctx_size: u32,
    pub batch_size: u32,
    pub backend: String,
    pub tensor_split: String,
    pub mmap: bool,
    pub flash_attn: bool,
}

impl LlamaCppConfig {
    pub fn generate(
        total_memory_mb: u64,
        device_memory_mb: u64,
        model_size_mb: u64,
        cpu_cores: u32,
        has_vulkan: bool,
    ) -> Self {
        let tensor_ratio = if device_memory_mb > 0 {
            let ratio = device_memory_mb as f64 / total_memory_mb as f64;
            format!("{:.2},{:.2}", ratio, 1.0 - ratio)
        } else {
            "0,0".to_string()
        };

        let n_gpu_layers = Self::calculate_gpu_layers(device_memory_mb, model_size_mb);

        let threads = cpu_cores.max(4);
        let n_threads_batch = threads * 2;

        Self {
            n_gpu_layers,
            n_threads: threads,
            n_threads_batch,
            ctx_size: match model_size_mb {
                m if m < 2000 => 2048,
                m if m < 6000 => 4096,
                _ => 8192,
            },
            batch_size: 512,
            backend: if has_vulkan {
                "vulkan".to_string()
            } else {
                "cpu".to_string()
            },
            tensor_split: tensor_ratio,
            mmap: false,
            flash_attn: true,
        }
    }

    fn calculate_gpu_layers(device_memory_mb: u64, model_size_mb: u64) -> u32 {
        if device_memory_mb == 0 {
            return 0;
        }
        let usable = (device_memory_mb as f64 * 0.7) as u64;
        let layers = match model_size_mb {
            m if m < 1000 => 32,
            m if m < 4000 => 35,
            m if m < 8000 => 40,
            _ => 48,
        };
        std::cmp::min(layers, ((usable / 200) as u32).max(14))
    }

    pub fn for_moe(
        total_memory_mb: u64,
        device_memory_mb: u64,
        model_size_mb: u64,
        cpu_cores: u32,
        has_vulkan: bool,
        expert_layers_cpu: u32,
    ) -> Self {
        let mut config = Self::generate(
            total_memory_mb,
            device_memory_mb,
            model_size_mb,
            cpu_cores,
            has_vulkan,
        );

        let total_layers = match model_size_mb {
            m if m < 1000 => 32,
            m if m < 4000 => 35,
            m if m < 8000 => 40,
            _ => 48,
        };

        if expert_layers_cpu > 0 {
            config.n_gpu_layers = total_layers - expert_layers_cpu;
            config.tensor_split = format!(
                "{:.2},{:.2}",
                1.0 - (expert_layers_cpu as f64 / total_layers as f64),
                expert_layers_cpu as f64 / total_layers as f64
            );
        }

        config
    }

    pub fn to_flags(&self) -> Vec<String> {
        let mut flags = vec![
            format!("--n-gpu-layers {}", self.n_gpu_layers),
            format!("--threads {}", self.n_threads),
            format!("--threads-batch {}", self.n_threads_batch),
            format!("--ctx-size {}", self.ctx_size),
            format!("--batch-size {}", self.batch_size),
        ];

        if self.n_gpu_layers > 0 {
            flags.push(format!("--backend {}", self.backend));
            if !self.mmap {
                flags.push("--no-mmap".to_string());
            }
            if self.tensor_split != "0,0" {
                flags.push(format!("--tensor-split {}", self.tensor_split));
            }
        }

        if self.flash_attn {
            flags.push("--flash-attn".to_string());
        }

        flags
    }

    pub fn to_command(&self, model_path: &str) -> String {
        let flags = self.to_flags();
        let mut cmd = String::from("llama-cli -m ");
        cmd.push_str(model_path);
        for flag in flags {
            cmd.push(' ');
            cmd.push_str(&flag);
        }
        cmd
    }
}
