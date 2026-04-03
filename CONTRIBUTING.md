# Contributing to OpenUMA

Thank you for your interest in contributing! OpenUMA is at an early stage where
**community hardware profiles are the single most valuable contribution you can make.**

---

## The Easiest Way to Contribute (5 minutes)

If you have an AMD APU or Intel iGPU machine:

```bash
# Build OpenUMA
git clone https://github.com/hamtun24/openuma.git
cd openuma
cargo build --release

# Run hardware probe
./target/release/openuma probe
```

Then [open a Hardware Profile issue](https://github.com/hamtun24/openuma/issues/new?template=hardware_profile.yml)
and paste the output. That's it. You've just improved OpenUMA for everyone
with similar hardware.

---

## Contributing Code

### Prerequisites

- Rust 1.70+
- Linux with DRM support (`/dev/dri/renderD128` should exist)
- Optional: AMD APU or Intel iGPU for testing

### Setup

```bash
git clone https://github.com/hamtun24/openuma.git
cd openuma
cargo build
cargo test
```

### Before Submitting a PR

```bash
# Must pass all of these
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo build --release
```

All four commands must succeed with zero errors and zero warnings.

### Code Structure

```
crates/
├── hw_probe/     # Hardware detection — touch this to add new CPU/iGPU support
├── mem_mgr/      # Memory partitioning — touch this to improve allocation logic
├── config_gen/   # Config generation — touch this to add new inference engines
├── profile_db/   # SQLite profiles — touch this to add/modify profile seeding
├── benchmark/    # Benchmarking — touch this to improve measurement accuracy
├── api_server/   # REST API — touch this to add endpoints
├── cli/          # CLI commands — touch this to add new commands
└── tui/          # Terminal UI — touch this to add new screens
```

### Adding a New Hardware Profile

Edit `crates/profile_db/src/db.rs` in the `seed_defaults()` function.
Follow the exact format of existing entries. Include:
- `cpu_codename` — exact codename string (e.g. "Cezanne", "Phoenix")
- `igpu_arch` — GPU architecture (e.g. "Vega7", "RDNA3_780M", "IrisXe")
- `ram_gb` — standard RAM size (16, 32, 64)
- `os_class` — "linux" or "windows"
- `igpu_alloc_pct` — tested optimal iGPU allocation percentage (0.0–0.45)
- `optimal_backend` — "vulkan", "opencl", or "cpu"
- `bandwidth_gbps` — measured or spec bandwidth
- `tokens_per_sec_7b` — measured tokens/sec on a 7B Q4_K_M model if available

### Adding a New Inference Engine

1. Add a new variant to the engine enum in `crates/config_gen/src/lib.rs`
2. Implement `generate_<engine>_config()` in `crates/config_gen/src/generators/`
3. Add a new Jinja2 template in `templates/`
4. Wire it up in `crates/cli/src/commands.rs`
5. Add documentation to `docs/user-guide.md`

---

## What We Are NOT Looking For (Yet)

- Windows support patches (planned for v1.0, needs a dedicated effort)
- macOS support (Apple Silicon already solves UMA natively)
- Training-related features (OpenUMA is inference-only)
- New model format support beyond GGUF

---

## Code of Conduct

Be respectful. This is a technical project — focus on the code and the hardware.
Benchmark numbers are facts, not arguments.
