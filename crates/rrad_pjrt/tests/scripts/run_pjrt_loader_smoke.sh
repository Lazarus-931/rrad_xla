#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=tests/scripts/common.sh
source "${SCRIPT_DIR}/common.sh"

PLUGIN_PATH="$(resolve_pjrt_plugin_path "${1:-}")"

export PJRT_PLUGIN="${PLUGIN_PATH}"

cd "${REPO_ROOT}"
echo "Running Rust PJRT loader smoke test with: ${PJRT_PLUGIN}"
cargo run --bin rrad_pjrt
