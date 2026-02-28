"""TensorFlow PJRT plugin registration helper for the rrad_xla Rust integration.

Registers the Rust PJRT proxy cdylib (``librrad_xla.so``) with TensorFlow so
that XLA-backed TF operations execute through the Rust layer.

Usage
-----
.. code-block:: python

    import os
    os.environ["RRAD_PJRT_PLUGIN"]         = "/path/to/librrad_xla.so"
    os.environ["RRAD_PJRT_BACKEND_PLUGIN"] = "/path/to/pjrt_c_api_cpu_plugin.so"

    from tf_pjrt import initialize
    device_type = initialize()
    # device_type == "rrad_cpu"

    import tensorflow as tf
    with tf.device(f"/{device_type}:0"):
        result = tf.add(1.0, 1.0)
    print(result)  # tf.Tensor(2.0, ...)
"""

from __future__ import annotations

import os
from pathlib import Path


_DEFAULT_DEVICE_TYPE = "rrad_cpu"


# ---------------------------------------------------------------------------
# Path resolution helpers  (shared with the JAX plugin)
# ---------------------------------------------------------------------------

def _repo_root() -> Path:
    return Path(__file__).resolve().parents[3]


def _default_rust_plugin_path() -> Path:
    root = _repo_root()
    candidates = (
        root / "target" / "release" / "librrad_xla.so",
        root / "target" / "release" / "librrad_xla.dylib",
        root / "target" / "debug" / "librrad_xla.so",
        root / "target" / "debug" / "librrad_xla.dylib",
    )
    for c in candidates:
        if c.exists():
            return c
    return candidates[0]


def _default_xla_plugin_path() -> Path:
    root = _repo_root()
    candidates = (
        root / "xla" / "bazel-bin" / "xla" / "pjrt" / "c" / "pjrt_c_api_cpu_plugin.so",
        root / "xla" / "bazel-bin" / "xla" / "pjrt" / "c" / "pjrt_c_api_cpu_plugin.dylib",
    )
    for c in candidates:
        if c.exists():
            return c
    return candidates[0]


# ---------------------------------------------------------------------------
# Public API
# ---------------------------------------------------------------------------

def initialize(device_type: str | None = None) -> str:
    """Register the rrad_xla Rust PJRT plugin with TensorFlow.

    TensorFlow 2.14+ supports ``tf.config.experimental.register_pjrt_plugin``
    which loads a PJRT plugin `.so` (via ``GetPjrtApi``) and makes it
    available as a custom device type.

    Parameters
    ----------
    device_type:
        The device-type string to register (default: ``"rrad_cpu"`` or the
        value of ``RRAD_TF_DEVICE_TYPE``).

    Returns
    -------
    str
        The registered device-type string.

    Raises
    ------
    ImportError
        If TensorFlow is not installed.
    FileNotFoundError
        If the Rust plugin .so cannot be located.
    RuntimeError
        If the TF PJRT plugin API is not available (TF < 2.14).
    """
    if device_type is None:
        device_type = os.environ.get("RRAD_TF_DEVICE_TYPE", _DEFAULT_DEVICE_TYPE)

    # Resolve the Rust plugin path.
    plugin_path = Path(
        os.environ.get("RRAD_PJRT_PLUGIN", str(_default_rust_plugin_path()))
    )
    if not plugin_path.exists():
        raise FileNotFoundError(
            f"Rust PJRT plugin not found at '{plugin_path}'.\n"
            "Build it with:  cargo build --release\n"
            "or set RRAD_PJRT_PLUGIN to the correct path."
        )

    # Propagate the XLA backend path so the Rust proxy can find it.
    if "RRAD_PJRT_BACKEND_PLUGIN" not in os.environ:
        xla_plugin = _default_xla_plugin_path()
        if xla_plugin.exists():
            os.environ["RRAD_PJRT_BACKEND_PLUGIN"] = str(xla_plugin)

    try:
        import tensorflow as tf  # type: ignore[import]
    except ImportError as exc:
        raise ImportError(
            "TensorFlow is required for tf_pjrt integration.\n"
            "Install with:  pip install tensorflow>=2.14"
        ) from exc

    # TF 2.14+ exposes register_pjrt_plugin.
    register = getattr(
        tf.config.experimental, "register_pjrt_plugin", None
    )
    if register is None:
        raise RuntimeError(
            "tf.config.experimental.register_pjrt_plugin is not available.\n"
            "Upgrade TensorFlow to >= 2.14."
        )

    register(device_type, library_path=str(plugin_path))
    return device_type
