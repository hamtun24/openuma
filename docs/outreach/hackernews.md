# Hacker News Post Draft

**Title:**
Show HN: OpenUMA – bring Apple-style unified memory to x86 AI inference (Rust, Linux)

**Body:**

Apple Silicon dominates local LLM benchmarks largely because of unified memory:
CPU and GPU share the same physical pool, no PCIe copy overhead.

The irony: AMD APUs and Intel iGPUs already do this at the silicon level.
A Ryzen 5600G with 32GB RAM can allocate 16GB to the iGPU — the same physical
DDR4 the CPU is using. The software just doesn't know how to use it.

OpenUMA is a Rust middleware daemon that:

- Detects the hardware (CPU generation, iGPU arch, RAM speed/channels)
- Computes the optimal CPU/iGPU memory split for a given model
- Generates exact configuration flags for llama.cpp, Ollama, KTransformers
- Implements a DMA-BUF zero-copy bridge so tensor data isn't copied between
  CPU and iGPU address spaces

The core idea is the same as what KTransformers does for MoE models
(attention layers → GPU, expert layers → CPU) but applied at the
configuration layer for standard llama.cpp, so you don't need to change
your inference setup.

It's not trying to replace inference engines. It's the missing configuration
and memory management layer that makes existing tools work correctly on
this class of hardware.

v0.6, Linux only, 37 hardware profiles pre-seeded (23 AMD, 14 Intel).

Code: https://github.com/hamtun24/openuma
