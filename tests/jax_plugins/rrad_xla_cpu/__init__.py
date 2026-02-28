"""JAX plugin registration helper for the rrad_xla Rust PJRT integration.

Supports two operating modes, selected by environment variables:

Mode A — Rust proxy plugin (default)
    The Rust *rrad_xla* cdylib acts as the PJRT plugin.  JAX loads it via
    ``GetPjrtApi``.  The Rust layer then forwards every PJRT call to the
    backend plugin whose path is stored in ``RRAD_PJRT_BACKEND_PLUGIN``.

    Required env vars:
        RRAD_PJRT_PLUGIN          Path to ``librrad_xla.so`` (the Rust cdylib).
        RRAD_PJRT_BACKEND_PLUGIN  Path to the XLA CPU plugin .so that the Rust
                                  proxy will forward calls to.

Mode B — direct XLA plugin (legacy / fallback)
    The XLA-built ``pjrt_c_api_cpu_plugin.so`` is registered directly with
    JAX, bypassing the Rust layer.  Activated by setting
    ``RRAD_PLUGIN_MODE=direct`` (or by leaving ``RRAD_PJRT_PLUGIN`` unset and
    falling back to ``PJRT_PLUGIN``).
"""

from __future__ import annotations

import os
from pathlib import Path

_DEFAULT_BACKEND = "rrad_cpu"


# ---------------------------------------------------------------------------
# Path resolution helpers
# ---------------------------------------------------------------------------

def _repo_root() -> Path:
    return Path(__file__).resolve().parents[3]


def _default_rust_plugin_path() -> Path:
    """Locate the compiled Rust cdylib (librrad_xla.so / .dylib)."""
    root = _repo_root()
    candidates = (
        root / "target" / "release" / "librrad_xla.so",
        root / "target" / "release" / "librrad_xla.dylib",
        root / "target" / "debug" / "librrad_xla.so",
        root / "target" / "debug" / "librrad_xla.dylib",
    )
    for candidate in candidates:
        if candidate.exists():
            return candidate
    return candidates[0]  # will produce a clear FileNotFoundError below


def _default_xla_plugin_path() -> Path:
    """Locate the XLA-built PJRT CPU plugin."""
    root = _repo_root()
    candidates = (
        root / "xla" / "bazel-bin" / "xla" / "pjrt" / "c" / "pjrt_c_api_cpu_plugin.so",
        root / "xla" / "bazel-bin" / "xla" / "pjrt" / "c" / "pjrt_c_api_cpu_plugin.dylib",
    )
    for candidate in candidates:
        if candidate.exists():
            return candidate
    return candidates[0]


# ---------------------------------------------------------------------------
# Public API
# ---------------------------------------------------------------------------

def initialize(mode: str | None = None) -> str:
    """Register the rrad_xla PJRT plugin with JAX and return the backend name.

    Parameters
    ----------
    mode:
        ``"rust"`` — use the Rust proxy cdylib as the PJRT plugin (default).
        ``"direct"`` — register the XLA plugin directly (legacy / bypass Rust).
        ``None`` — auto-detect from ``RRAD_PLUGIN_MODE`` env var; defaults
        to ``"rust"`` when the env var is absent.

    Returns
    -------
    str
        The backend name that was registered with JAX (e.g. ``"rrad_cpu"``).
    """
    if mode is None:
        mode = os.environ.get("RRAD_PLUGIN_MODE", "rust")

    backend_name = os.environ.get("RRAD_JAX_BACKEND", _DEFAULT_BACKEND)

    if mode == "rust":
        library_path = Path(
            os.environ.get("RRAD_PJRT_PLUGIN", str(_default_rust_plugin_path()))
        )
        if not library_path.exists():
            raise FileNotFoundError(
                f"Rust PJRT plugin not found at '{library_path}'.\n"
                "Build it with:\n"
                "  cargo build --release\n"
                "or set RRAD_PJRT_PLUGIN to the correct path."
            )
        # Propagate the XLA backend path so the Rust proxy can find it.
        if "RRAD_PJRT_BACKEND_PLUGIN" not in os.environ:
            xla_plugin = _default_xla_plugin_path()
            if xla_plugin.exists():
                os.environ["RRAD_PJRT_BACKEND_PLUGIN"] = str(xla_plugin)
            # If neither path exists the Rust proxy will emit a clear error
            # when JAX calls PJRT_Plugin_Initialize.

    elif mode == "direct":
        library_path = Path(
            os.environ.get("PJRT_PLUGIN", str(_default_xla_plugin_path()))
        )
        if not library_path.exists():
            raise FileNotFoundError(
                f"XLA PJRT plugin not found at '{library_path}'.\n"
                "Build it with:\n"
                "  (cd xla && bazel build //xla/pjrt/c:pjrt_c_api_cpu_plugin.so)\n"
                "or set PJRT_PLUGIN to the correct path."
            )
    else:
        raise ValueError(
            f"Unknown RRAD_PLUGIN_MODE '{mode}'.  Use 'rust' or 'direct'."
        )

    import jax._src.xla_bridge as xb  # type: ignore[import]

    xb.register_plugin(
        backend_name,
        priority=500,
        library_path=str(library_path),
        options=None,
    )
    return backend_name
