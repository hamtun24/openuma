# r/LocalLLaMA Post Draft

**Title:**
I built OpenUMA — auto-configure llama.cpp for AMD APUs and Intel iGPUs to mimic Apple's unified memory

**Body:**

Hey r/LocalLLaMA,

I've been frustrated that Apple Silicon gets all the attention for local AI
because of unified memory, while most of us have AMD APUs or Intel iGPUs
that actually share memory with the CPU already — the software just doesn't use it.

So I built **OpenUMA** — a Rust tool that:

1. Detects your hardware (AMD Ryzen APU / Intel iGPU / etc.)
2. Calculates the optimal memory split between CPU and iGPU
3. Generates the exact llama.cpp flags to exploit it (Vulkan backend,
   correct `--n-gpu-layers`, thread count, mmap settings)
4. Optionally runs a real benchmark to confirm the config works

**Example output on Ryzen 5 5600G:**

```
openuma configure --engine llamacpp --model llama3-8b-q4_k_m.gguf

  --n-gpu-layers 14
  --threads 6 --threads-batch 12
  --ctx-size 4096 --batch-size 512
  --backend vulkan --no-mmap --tensor-split 0.35,0.65

Estimated: ~7.2 tokens/sec
UMA pool: 10.5GB iGPU / 19.5GB CPU
```

It also has a full interactive TUI (`openuma tui`) and supports Ollama
and KTransformers config generation.

**GitHub:** https://github.com/hamtun24/openuma

It's early (v0.6, Linux only right now) but the core is solid and Clippy-clean.
Would love feedback from people with AMD APU hardware especially —
the profile database only gets better with real benchmark data.

Happy to answer questions about the architecture.
