#!/usr/bin/env bash
# Run the JAX plugin smoke test using the Rust PJRT proxy cdylib.
#
# The Rust cdylib (librrad_xla.so) is built first if it doesn't exist or is
# stale, then it is registered as the PJRT plugin.  The Rust proxy layer
# forwards every PJRT call to the XLA CPU plugin whose path is taken from the
# first positional argument or from RRAD_PJRT_BACKEND_PLUGIN / PJRT_PLUGIN.
#
# Usage:
#   tests/run_rust_pjrt_plugin_smoke.sh [xla_plugin_path]
#
# Examples:
#   tests/run_rust_pjrt_plugin_smoke.sh
#   tests/run_rust_pjrt_plugin_smoke.sh /path/to/pjrt_c_api_cpu_plugin.so
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=tests/common.sh
source "${SCRIPT_DIR}/common.sh"

PYTHON_BIN="${PYTHON_BIN:-python3}"

# ---- Resolve the XLA backend plugin path ----------------------------------
XLA_PLUGIN_OVERRIDE="${1:-}"
XLA_PLUGIN_PATH="$(resolve_pjrt_plugin_path "${XLA_PLUGIN_OVERRIDE}")"

# ---- Build the Rust PJRT proxy cdylib ------------------------------------
cd "${REPO_ROOT}"
echo "Building Rust PJRT proxy cdylib (--release, feature pjrt-plugin) …"
cargo build --release

# Locate the built shared library.
RUST_PLUGIN_PATH=""
for candidate in \
    "${REPO_ROOT}/target/release/librrad_xla.so" \
    "${REPO_ROOT}/target/release/librrad_xla.dylib"
do
    if [[ -f "${candidate}" ]]; then
        RUST_PLUGIN_PATH="${candidate}"
        break
    fi
done

if [[ -z "${RUST_PLUGIN_PATH}" ]]; then
    echo "error: could not find librrad_xla.so after cargo build" >&2
    exit 1
fi

echo "Rust PJRT proxy plugin : ${RUST_PLUGIN_PATH}"
echo "XLA backend plugin     : ${XLA_PLUGIN_PATH}"

# ---- Verify the GetPjrtApi symbol is exported ----------------------------
if command -v nm >/dev/null 2>&1; then
    if nm -D "${RUST_PLUGIN_PATH}" | grep -q 'GetPjrtApi'; then
        echo "Symbol check: GetPjrtApi ✓"
    else
        echo "warning: GetPjrtApi not found in ${RUST_PLUGIN_PATH}" >&2
    fi
fi

# ---- Sanity-check Python availability ------------------------------------
if ! command -v "${PYTHON_BIN}" >/dev/null 2>&1; then
    echo "error: python executable not found: ${PYTHON_BIN}" >&2
    exit 1
fi

if ! "${PYTHON_BIN}" -c "import jax" >/dev/null 2>&1; then
    cat >&2 <<'ERR'
error: jax is not installed.
Install compatible jax + jaxlib before running this test.
ERR
    exit 1
fi

# ---- Run the JAX smoke test in Rust proxy mode ---------------------------
export RRAD_PJRT_PLUGIN="${RUST_PLUGIN_PATH}"
export RRAD_PJRT_BACKEND_PLUGIN="${XLA_PLUGIN_PATH}"
export RRAD_PLUGIN_MODE="rust"
export PYTHONPATH="${REPO_ROOT}/tests${PYTHONPATH:+:${PYTHONPATH}}"

echo "Running JAX smoke test through Rust PJRT proxy …"
"${PYTHON_BIN}" "${SCRIPT_DIR}/jax_plugin_smoke.py"
