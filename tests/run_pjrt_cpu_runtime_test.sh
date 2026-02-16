#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=tests/common.sh
source "${SCRIPT_DIR}/common.sh"

PLUGIN_PATH="$(resolve_pjrt_plugin_path "${1:-}")"
export PJRT_PLUGIN="${PLUGIN_PATH}"

cd "${REPO_ROOT}"
echo "Running Rust PJRT CPU runtime test with: ${PJRT_PLUGIN}"
cargo test --test pjrt_cpu_runtime -- --nocapture
echo "Running Rust PJRT CPU end-to-end compile/execute test with: ${PJRT_PLUGIN}"
cargo test --test cpu -- --nocapture
