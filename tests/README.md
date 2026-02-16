# PJRT Integration Tests

This folder contains smoke tests for the two integration layers in this repo:

1. Rust loader integration (`GetPjrtApi` + `PJRT_Plugin_Initialize`)
2. JAX plugin integration (`register_plugin` + basic execution)

## Prerequisites

1. Build the PJRT CPU plugin artifact:

```bash
cd xla
bazel build //xla/pjrt/c:pjrt_c_api_cpu_plugin.so
```

2. For JAX tests, install `jax` and `jaxlib` versions compatible with your XLA commit.

## Tests

Run Rust loader smoke test:

```bash
tests/run_pjrt_loader_smoke.sh
```

Run Rust CPU runtime integration test (client/topology/device/buffer metadata):

```bash
tests/run_pjrt_cpu_runtime_test.sh
```

Run JAX plugin smoke test (add, jit, pmap):

```bash
tests/run_jax_plugin_smoke.sh
```

Optional plugin override:

```bash
tests/run_jax_plugin_smoke.sh --plugin /absolute/path/to/pjrt_c_api_cpu_plugin.so
```

## Notes

- These tests default to the CPU PJRT plugin built under `xla/bazel-bin`.
- Backend name defaults to `rrad_cpu` and can be overridden via `RRAD_JAX_BACKEND`.
- GitHub Actions runs `smoke` and `PJRT Loader Integration` on each push/PR.
- `JAX Plugin Smoke` runs on manual workflow dispatch.
