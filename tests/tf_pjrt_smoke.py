#!/usr/bin/env python3
"""Smoke tests for the rrad_xla Rust PJRT plugin with TensorFlow.

Registers the Rust cdylib as a TF PJRT device and runs a basic scalar
addition to verify end-to-end execution through the Rust proxy layer.

Environment variables
---------------------
RRAD_PJRT_PLUGIN          Path to ``librrad_xla.so`` (the Rust cdylib).
RRAD_PJRT_BACKEND_PLUGIN  Path to the XLA backend PJRT plugin .so.
RRAD_TF_DEVICE_TYPE       Device type name to register (default: rrad_cpu).
"""

from __future__ import annotations

import argparse
import os
import sys


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--plugin",
        default=os.environ.get("RRAD_PJRT_PLUGIN", ""),
        help="Path to the Rust PJRT plugin .so (librrad_xla.so).",
    )
    parser.add_argument(
        "--backend",
        default=os.environ.get("RRAD_PJRT_BACKEND_PLUGIN", ""),
        help="Path to the XLA PJRT backend plugin .so.",
    )
    parser.add_argument(
        "--device-type",
        default=os.environ.get("RRAD_TF_DEVICE_TYPE", "rrad_cpu"),
        help="TF device type name to register.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()

    if args.plugin:
        os.environ["RRAD_PJRT_PLUGIN"] = args.plugin
    if args.backend:
        os.environ["RRAD_PJRT_BACKEND_PLUGIN"] = args.backend
    if args.device_type:
        os.environ["RRAD_TF_DEVICE_TYPE"] = args.device_type

    try:
        # Register the Rust PJRT plugin with TF.
        from tf_pjrt import initialize  # type: ignore[import]
        device_type = initialize()
        print(f"Registered TF PJRT device: {device_type}")

        import tensorflow as tf  # type: ignore[import]

        # ----------------------------------------------------------------
        # Test 1: scalar addition
        # ----------------------------------------------------------------
        with tf.device(f"/{device_type}:0"):
            a = tf.constant(1.0)
            b = tf.constant(1.0)
            result = tf.add(a, b)
        assert float(result) == 2.0, f"scalar add failed: got {float(result)}"
        print("scalar add test passed")

        # ----------------------------------------------------------------
        # Test 2: matrix multiply
        # ----------------------------------------------------------------
        with tf.device(f"/{device_type}:0"):
            m = tf.constant([[1.0, 2.0], [3.0, 4.0]])
            identity = tf.eye(2)
            product = tf.matmul(m, identity)
        expected = [[1.0, 2.0], [3.0, 4.0]]
        assert product.numpy().tolist() == expected, (
            f"matmul test failed: got {product.numpy().tolist()}"
        )
        print("matmul test passed")

        # ----------------------------------------------------------------
        # Test 3: @tf.function (XLA JIT compilation through Rust proxy)
        # ----------------------------------------------------------------
        @tf.function(jit_compile=True)
        def add_and_square(x: tf.Tensor) -> tf.Tensor:
            return tf.square(x + tf.constant(1.0))

        with tf.device(f"/{device_type}:0"):
            out = add_and_square(tf.constant(3.0))
        assert float(out) == 16.0, f"jit_compile test failed: got {float(out)}"
        print("@tf.function(jit_compile=True) test passed")

    except Exception as exc:  # pylint: disable=broad-except
        print(f"TF PJRT smoke test FAILED: {exc}", file=sys.stderr)
        return 1

    print("TF PJRT smoke test PASSED")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
