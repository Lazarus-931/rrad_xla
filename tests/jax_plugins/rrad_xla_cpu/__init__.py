"""JAX plugin registration helper for local PJRT CPU plugin testing."""

from __future__ import annotations

import os
from pathlib import Path

_DEFAULT_BACKEND = "rrad_cpu"


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[3]


def _default_library_path() -> Path:
    root = _repo_root()
    candidates = (
        root / "xla" / "bazel-bin" / "xla" / "pjrt" / "c" / "pjrt_c_api_cpu_plugin.so",
        root / "xla" / "bazel-bin" / "xla" / "pjrt" / "c" / "pjrt_c_api_cpu_plugin.dylib",
    )
    for candidate in candidates:
        if candidate.exists():
            return candidate
    return candidates[0]


def initialize() -> str:
    """Register the local PJRT plugin with JAX and return backend name."""
    backend_name = os.environ.get("RRAD_JAX_BACKEND", _DEFAULT_BACKEND)
    library_path = Path(os.environ.get("PJRT_PLUGIN", str(_default_library_path())))

    if not library_path.exists():
        raise FileNotFoundError(
            f"PJRT plugin library not found at '{library_path}'. "
            "Build //xla/pjrt/c:pjrt_c_api_cpu_plugin.so first."
        )

    import jax._src.xla_bridge as xb

    xb.register_plugin(
        backend_name,
        priority=500,
        library_path=str(library_path),
        options=None,
    )
    return backend_name
