# llama.cpp Discussion Reply Draft

Post this as a reply in the llama.cpp hybrid CPU/GPU inference discussion thread:
https://github.com/ggml-org/llama.cpp/discussions/8721

---

For anyone running llama.cpp on AMD APUs or Intel iGPUs specifically,
I built a complementary tool called **OpenUMA** that handles the configuration
side of this problem.

It detects your shared memory hardware, computes the optimal iGPU/CPU memory
split for your model, and generates the exact `--n-gpu-layers`, `--backend`,
`--tensor-split`, and thread settings automatically.

The core idea is the same as what KTransformers does for MoE models
(attention layers → GPU, expert layers → CPU) but applied at the
configuration layer for standard llama.cpp, so you don't need to change
your inference setup.

https://github.com/hamtun24/openuma

Happy to discuss the memory partitioning algorithm if useful for
upstream llama.cpp improvements.
