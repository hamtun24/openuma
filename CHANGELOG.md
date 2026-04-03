# Changelog

All notable changes to OpenUMA are documented here.

---

## [v0.6.0] — 2026-04-03

### Added
- **Terminal UI (TUI)** — Full interactive terminal interface with 6 screens:
  Dashboard, Memory, Benchmark, Profiles, Configure, Settings
- `openuma tui` command to launch the interactive UI
- Tab-based navigation and real-time hardware display
- Memory partition visualization with progress bar
- Profile browser with keyboard navigation

### Changed
- Zero Clippy warnings across all crates (production-grade code quality)
- Workspace resolver upgraded to resolver = "2"

### Fixed
- Field reassignment with default patterns replaced with proper struct init
- Unnecessary casts and format string issues cleaned up
- Dead code annotated with `#[allow(dead_code)]` where appropriate

---

## [v0.5.0] — Initial v0.5 Roadmap

### Added
- **Benchmark module** (`crates/benchmark/`) — real inference benchmarking via llama-bench
  - Auto-detects best backend (vulkan / opencl / cpu)
  - Full multi-backend comparison with `--full` flag
  - Fallback to estimated performance when llama.cpp not installed
- **Hardware profile database expanded** from 10 to 37 profiles:
  - 23 AMD profiles (Zen 3 through Zen 5, Cezanne to Strix Point)
  - 14 Intel profiles (Alder Lake through Arrow Lake / Lunar Lake)
- **KTransformers YAML config generation** for MoE models
- **Ollama Modelfile generation**
- Jinja2 templates for all engine configs (`templates/`)

---

## [v0.3.0] — Zero-Copy Bridge

### Added
- **DMA-BUF + PRIME zero-copy bridge** (`crates/mem_mgr/src/zero_copy/linux.rs`)
  - Opens `/dev/dri/renderD128` DRM device
  - Allocates GEM buffer with `drm_mode_create_dumb` ioctl
  - Exports to PRIME fd and maps into CPU address space
  - `write_tensor()` and `read_tensor()` for zero-copy tensor operations
- `openuma zerocopy --test` command to verify zero-copy availability
- Platform stub modules for Windows and unsupported platforms

---

## [v0.2.0] — Quick Wins

### Added
- Integration test suite (`tests/integration/`)
- 10 initial hardware profiles seeded in the profile database

### Fixed
- Compiler warnings cleaned up across all crates

---

## [v0.1.0] — MVP (Phase 0–4)

### Added
- Full 7-crate Rust workspace structure
- **hw_probe** — CPU, iGPU, RAM detection on Linux
  - AVX level detection (AVX2, AVX-512, AMX)
  - Vulkan compute capability check
  - DMA-BUF / zero-copy support detection
  - Memory bandwidth microbenchmark
- **mem_mgr** — Greedy UMA partition algorithm
  - Attention-aware layer split (attention → iGPU, MoE experts → CPU)
  - PureCpu / HybridIgpu / HybridDgpu / FullUma strategies
- **config_gen** — GGUF metadata reader + llama.cpp config generation
  - GGUF magic validation and metadata KV parsing
  - MoE detection via `llm.expert_count`
  - Quantization inference from filename
- **profile_db** — SQLite hardware profile database with TOML loader
- **api_server** — axum REST API with 7 endpoints
- **cli** — Full CLI with `probe`, `configure`, `partition`, `recommend`,
  `status`, `benchmark`, `profile`, `serve` commands
- MIT License
