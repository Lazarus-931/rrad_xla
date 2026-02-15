#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=tests/common.sh
source "${SCRIPT_DIR}/common.sh"

PYTHON_BIN="${PYTHON_BIN:-python3}"

PLUGIN_OVERRIDE=""
if [[ "${1:-}" == "--plugin" ]]; then
  if [[ -z "${2:-}" ]]; then
    echo "error: --plugin requires a file path" >&2
    exit 1
  fi
  PLUGIN_OVERRIDE="${2}"
  shift 2
fi

if ! command -v "${PYTHON_BIN}" >/dev/null 2>&1; then
  echo "error: python executable not found: ${PYTHON_BIN}" >&2
  exit 1
fi

if ! "${PYTHON_BIN}" -c "import jax" >/dev/null 2>&1; then
  cat >&2 <<'ERR'
error: jax is not installed for this interpreter.
Install nightly jax + jaxlib (matching your XLA commit) before running this test.
ERR
  exit 1
fi

PLUGIN_PATH="$(resolve_pjrt_plugin_path "${PLUGIN_OVERRIDE}")"

export PJRT_PLUGIN="${PLUGIN_PATH}"
export PYTHONPATH="${REPO_ROOT}/tests${PYTHONPATH:+:${PYTHONPATH}}"

echo "Running JAX PJRT smoke test with: ${PJRT_PLUGIN}"
"${PYTHON_BIN}" "${SCRIPT_DIR}/jax_plugin_smoke.py" --plugin "${PJRT_PLUGIN}" "$@"
