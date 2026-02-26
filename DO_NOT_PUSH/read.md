
1. Build a Rust PJRT plugin/runtime implementation (not just consumer wrappers), CPU-first.
2. Wire end-to-end with JAX plugin discovery and run real JAX smoke/integration tests.
3. Move down-stack into StableHLO/MLIR program handling and compile/execute semantics parity.
4. Add backend specialization layers (CPU solid, then GPU, then TPU where feasible).
5. Expand conformance/perf testing and extension support (callbacks, topology, memory, DMA, async paths).
6. After runtime parity, start replacing higher XLA-adjacent components incrementally in Rust.