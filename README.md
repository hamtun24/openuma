# OpenUMA

**OpenUMA** (Unified Memory Abstraction) is a Rust middleware for detecting shared memory hardware (AMD APUs, Intel iGPUs), configuring unified memory pools, and generating optimal configs for AI inference engines.

[![Build](https://github.com/hamtun24/openuma/actions/workflows/build.yml/badge.svg)](https://github.com/hamtun24/openuma/actions/workflows/build.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-Linux-yellowgreen.svg)](https://github.com/hamtun24/openuma)
[![Version](https://img.shields.io/badge/version-v0.6.0-blue.svg)](https://github.com/hamtun24/openuma/releases)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          OpenUMA v0.6.2                                  │
│              Unified Memory Abstraction for AI Inference                    │
└─────────────────────────────────────────────────────────────────────────┘
```

## Key Features

- **Hardware Detection** - Automatic detection of AMD APUs and Intel iGPUs
- **Memory Partitioning** - Intelligent iGPU/CPU memory allocation for LLM inference
- **Zero-Copy DMA-BUF** - Direct memory transfers between CPU and iGPU
- **Multiple Engines** - Generate configs for llama.cpp, Ollama, and KTransformers
- **Interactive TUI** - Full terminal UI for hardware monitoring and configuration
- **Benchmarking** - Real inference benchmarks with llama.cpp

## Supported Hardware

| Vendor | Series | Examples |
|--------|--------|----------|
| AMD | Zen 3 (Cezanne, Renoir) | Ryzen 5 5600G, Ryzen 7 5700G |
| AMD | Zen 4 (Phoenix, Hawk Point) | Ryzen 7 7840HS, Ryzen AI 9 HX 370 |
| AMD | Zen 5 (Strix Point) | Ryzen AI 9 HX 370, Ryzen AI 7 350 |
| Intel | Alder Lake, Raptor Lake | Core i5-1240P, Core i7-12700H |
| Intel | Meteor Lake, Lunar Lake | Core Ultra 5 125H, Core Ultra 7 258V |

## Quick Start

```bash
# Build
cargo build --release

# Detect hardware
./target/release/openuma probe

# Launch interactive TUI
./target/release/openuma tui

# Generate config for llama.cpp
./target/release/openuma configure --engine llamacpp --model model.gguf
```

## Terminal UI

```
┌─────────────────────────────────────────────────────────────────────────┐
│  [D]ashboard  [M]emory  [B]enchmark  [P]rofiles  [C]onfigure  [S]ettings│
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ╔═══════════════════════════════════════════════════════════════════╗  │
│  ║                     Hardware Overview                             ║  │
│  ╠═══════════════════════════════════════════════════════════════════╣  │
│  ║  CPU    AMD Ryzen 5 5600G (Cezanne)                             ║  │
│  ║          6 cores (12 threads), AVX2, 16MB L3                     ║  │
│  ║  iGPU   AMD Vega7 (Raven Ridge)                                  ║  │
│  ║          7 CUs, 512MB / 16384MB shared VRAM                      ║  │
│  ║          Vulkan ✓  OpenCL ✓  Zero-copy ✓                         ║  │
│  ║  RAM    32GB DDR4-3200 (Dual-channel)                            ║  │
│  ║          51.2 GB/s theoretical, 46.8 GB/s measured               ║  │
│  ╠═══════════════════════════════════════════════════════════════════╣  │
│  ║  ✓ Unified Memory Available    Tier: CONSUMER_UMA                 ║  │
│  ╚═══════════════════════════════════════════════════════════════════╝  │
│                                                                         │
│  Memory Partition:  iGPU: 7168 MB (35.0%)  CPU: 13312 MB (65.0%)       │
│  [████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░] 35%            │
│                                                                         │
│  Strategy: HybridIgpu  Zero-copy: Available                            │
│                                                                         │
│  [r] Refresh    [q] Quit                                                 │
└─────────────────────────────────────────────────────────────────────────┘
```

## Commands

| Command | Description |
|---------|-------------|
| `openuma probe` | Detect hardware profile |
| `openuma tui` | Launch interactive terminal UI |
| `openuma partition --model <path>` | Show memory partition for model |
| `openuma configure --engine <engine> --model <path>` | Generate engine config |
| `openuma benchmark --model <path>` | Run inference benchmark |
| `openuma zerocopy --test` | Test DMA-BUF zero-copy |
| `openuma serve` | Start REST API server |
| `openuma profile list` | List known hardware profiles |

## Supported Inference Engines

### llama.cpp

```bash
openuma configure --engine llamacpp --model llama3-8b-q4_k_m.gguf
```

### Ollama

```bash
openuma configure --engine ollama --model llama3-8b-q4_k_m.gguf
```

### KTransformers (MoE models)

```bash
openuma configure --engine ktransformers --model deepseek-v3-q4km.gguf
```

## How It Works

### Memory Model

```
┌────────────────────────────────────────────────────────────────┐
│                      Unified Memory Pool                       │
│                                                                │
│  ┌──────────────┐                    ┌──────────────────────┐  │
│  │   iGPU VRAM  │  ◄── Zero-Copy ──►│   System RAM        │  │
│  │   (Shared)   │      DMA-BUF        │   (DDR4/DDR5)       │  │
│  └──────────────┘                    └──────────────────────┘  │
│                                                                │
│  Attention layers benefit from iGPU                             │
│  MoE experts stay on CPU                                        │
└────────────────────────────────────────────────────────────────┘
```

### Key Insight

For LLM inference on APUs:
- **Attention layers** → benefit from iGPU (parallel matrix ops)
- **MoE expert layers** → should stay on CPU (sparse activation)
- **KV cache** → benefits from unified memory zero-copy

## Benchmarking

```bash
# Quick benchmark
openuma benchmark --model llama3-8b-q4_k_m.gguf

# Full multi-backend comparison
openuma benchmark --model model.gguf --full
```

```
╔════════════════════════════════════════════════════════════════════╗
║                     OpenUMA Benchmark Report                          ║
╠════════════════════════════════════════════════════════════════════╣
║ Best Backend: vulkan (12.5 t/s)
║ Average TPS: 8.2
╠════════════════════════════════════════════════════════════════════╣
║ Test 1: model.gguf [vulkan]
║   └── 12.5 tokens/sec | 8000 ms
║ Test 2: model.gguf [opencl]
║   └── 10.2 tokens/sec | 9800 ms
║ Test 3: model.gguf [cpu]
║   └── 4.8 tokens/sec | 20800 ms
╠════════════════════════════════════════════════════════════════════╣
║ Recommendations:
║ • Best performing backend: vulkan (~12.5 tokens/sec)
║ • GPU acceleration provides 2.6x speedup over CPU
╚════════════════════════════════════════════════════════════════════╝
```

## Architecture

```
openuma/
├── crates/
│   ├── hw_probe/      # Hardware detection
│   ├── mem_mgr/       # Memory partitioning + zero-copy
│   ├── config_gen/    # Model metadata (GGUF)
│   ├── profile_db/    # Hardware profile database
│   ├── benchmark/     # Inference benchmarking
│   ├── api_server/    # REST API
│   ├── cli/           # CLI interface
│   └── tui/           # Terminal UI
└── profiles/          # Hardware profiles
```

## Installation

### Option A — Download Binary (Linux x86_64)

```bash
# Download latest release
curl -L https://github.com/hamtun24/openuma/releases/latest/download/openuma-linux-x86_64.tar.gz \
  | tar xz

# Run it
./openuma probe
```

### Option B — Build from Source

```bash
# Prerequisites: Rust 1.70+
git clone https://github.com/hamtun24/openuma.git
cd openuma
cargo build --release
./target/release/openuma probe
```

### System Requirements

| Requirement | Notes |
|---|---|
| OS | Linux (kernel 5.10+) |
| CPU | Any x86_64 with AMD APU or Intel iGPU |
| RAM | 16GB minimum, 32GB recommended |
| Optional | llama.cpp in PATH for real benchmarks |
| Optional | Vulkan drivers for iGPU acceleration |

### Install Vulkan Drivers (if missing)

```bash
# AMD iGPU
sudo apt install mesa-vulkan-drivers

# Intel iGPU  
sudo apt install intel-media-va-driver mesa-vulkan-drivers
```

### Install llama.cpp (optional)

```bash
git clone https://github.com/ggerganov/llama.cpp
cd llama.cpp
mkdir build && cd build
cmake .. -DLLAMA_BUILD_EXAMPLES=ON
make -j$(nproc)
export PATH="$PATH:$(pwd)/bin"
```

## Real World Results

OpenUMA's value is in the configuration it generates — not just detecting hardware,
but knowing the exact flags that extract maximum performance from it.

### Example: AMD Ryzen 5 5600G + 32GB DDR4

| Setup | Command | Tokens/sec |
|---|---|---|
| llama.cpp defaults | `llama-cli -m model.gguf` | ~3.1 t/s |
| OpenUMA-configured | `openuma configure --engine llamacpp --model model.gguf` | ~7.2 t/s |
| Improvement | | **+132%** |

**What OpenUMA changed:**
- Enabled Vulkan backend (default is CPU)
- Set correct `--n-gpu-layers` for available shared VRAM
- Configured dual-channel memory-aware thread count
- Disabled mmap in favor of zero-copy DMA-BUF path

> **Note:** Numbers above are estimates from the profile database for this hardware.
> Run `openuma benchmark --model your-model.gguf --full` on your machine
> to get real measured numbers and contribute them to the community database.

### Community Benchmarks

*This section will grow as users submit hardware profiles.*
[Submit your results →](https://github.com/hamtun24/openuma/issues/new?template=hardware_profile.yml)

## Contributing

Contributions welcome! Open issues and pull requests.

## License

MIT License - see [LICENSE](LICENSE) for details.

---

**OpenUMA - Making every x86 machine a first-class AI citizen.**
