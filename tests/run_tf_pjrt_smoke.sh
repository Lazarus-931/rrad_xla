#!/usr/bin/env bash
# Run the TensorFlow PJRT smoke test using the Rust PJRT proxy cdylib.
#
# Builds the Rust cdylib if needed, then registers it with TensorFlow via
# tf.config.experimental.register_pjrt_plugin and runs basic op execution
# through the Rust proxy layer.
#
# Usage:
#   tests/run_tf_pjrt_smoke.sh [xla_plugin_path]
#
# Examples:
#   tests/run_tf_pjrt_smoke.sh
#   tests/run_tf_pjrt_smoke.sh /path/to/pjrt_c_api_cpu_plugin.so
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=tests/common.sh
source "${SCRIPT_DIR}/common.sh"

PYTHON_BIN="${PYTHON_BIN:-python3}"

# ---- Resolve the XLA backend plugin path ---------------------------------
XLA_PLUGIN_OVERRIDE="${1:-}"
XLA_PLUGIN_PATH="$(resolve_pjrt_plugin_path "${XLA_PLUGIN_OVERRIDE}")"

# ---- Build the Rust PJRT proxy cdylib -----------------------------------
cd "${REPO_ROOT}"
echo "Building Rust PJRT proxy cdylib (--release, feature pjrt-plugin) …"
cargo build --release

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

# ---- Sanity-check Python + TensorFlow ------------------------------------
if ! command -v "${PYTHON_BIN}" >/dev/null 2>&1; then
    echo "error: python executable not found: ${PYTHON_BIN}" >&2
    exit 1
fi

if ! "${PYTHON_BIN}" -c "import tensorflow" >/dev/null 2>&1; then
    cat >&2 <<'ERR'
error: tensorflow is not installed for this interpreter.
Install TensorFlow >= 2.14:
  pip install "tensorflow>=2.14"
ERR
    exit 1
fi

# ---- Run the TF smoke test -----------------------------------------------
export RRAD_PJRT_PLUGIN="${RUST_PLUGIN_PATH}"
export RRAD_PJRT_BACKEND_PLUGIN="${XLA_PLUGIN_PATH}"
export PYTHONPATH="${REPO_ROOT}/tests${PYTHONPATH:+:${PYTHONPATH}}"

echo "Running TF PJRT smoke test through Rust PJRT proxy …"
"${PYTHON_BIN}" "${SCRIPT_DIR}/tf_pjrt_smoke.py"
