#!/usr/bin/env python3
"""Smoke tests for PJRT plugin registration and basic JAX execution."""

from __future__ import annotations

import argparse
import os
import sys


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--plugin",
        default=os.environ.get("PJRT_PLUGIN", ""),
        help="Path to PJRT plugin shared library (defaults to $PJRT_PLUGIN).",
    )
    parser.add_argument(
        "--backend",
        default=os.environ.get("RRAD_JAX_BACKEND", "rrad_cpu"),
        help="Backend name to register and select via jax_platforms.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()

    if args.plugin:
        os.environ["PJRT_PLUGIN"] = args.plugin
    os.environ["RRAD_JAX_BACKEND"] = args.backend

    try:
        from jax_plugins.rrad_xla_cpu import initialize
        backend_name = initialize()

        import jax
        import jax.numpy as jnp

        jax.config.update("jax_platforms", backend_name)

        devices = jax.devices()
        print(f"backend={backend_name} device_count={len(devices)}")

        add_result = int(jnp.add(1, 1))
        assert add_result == 2, f"jax.numpy.add failed: got {add_result}"
        print("add test passed")

        jit_result = float(jax.jit(lambda x: x * 2.0)(1.0))
        assert jit_result == 2.0, f"jit test failed: got {jit_result}"
        print("jit test passed")

        arr = jnp.arange(jax.device_count(), dtype=jnp.int32)
        pmap_result = jax.pmap(
            lambda x: x + jax.lax.psum(x, "i"), axis_name="i"
        )(arr)
        expected = arr + jnp.sum(arr)
        assert pmap_result.tolist() == expected.tolist(), (
            f"pmap test failed: got {pmap_result.tolist()}, "
            f"expected {expected.tolist()}"
        )
        print("pmap test passed")
    except Exception as exc:  # pylint: disable=broad-except
        print(f"PJRT JAX integration smoke test failed: {exc}", file=sys.stderr)
        return 1

    print("PJRT JAX integration smoke test passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
