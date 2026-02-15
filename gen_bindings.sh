#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="${SCRIPT_DIR}"

usage() {
  cat <<'EOF'
Usage:
  ./gen_bindings.sh [header_path] [output_path]

Defaults:
  header_path:
    1) third_party/openxla/pjrt/pjrt_c_api.h
    2) expertnal_xla/pjrt_c_api.h.c
  output_path:
    src/pjrt_bindings.rs

Examples:
  ./gen_bindings.sh
  ./gen_bindings.sh expertnal_xla/pjrt_c_api.h.c src/pjrt_bindings.rs
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

if ! command -v bindgen >/dev/null 2>&1; then
  echo "Error: bindgen CLI not found in PATH." >&2
  echo "Install with: cargo install bindgen-cli" >&2
  exit 1
fi

resolve_header() {
  local requested="${1:-}"
  if [[ -n "${requested}" ]]; then
    if [[ -f "${requested}" ]]; then
      printf '%s\n' "${requested}"
      return 0
    fi
    echo "Error: header not found: ${requested}" >&2
    exit 1
  fi

  local candidate
  for candidate in \
    "${REPO_ROOT}/third_party/openxla/pjrt/pjrt_c_api.h" \
    "${REPO_ROOT}/expertnal_xla/pjrt_c_api.h.c"
  do
    if [[ -f "${candidate}" ]]; then
      printf '%s\n' "${candidate}"
      return 0
    fi
  done

  echo "Error: could not find PJRT header." >&2
  echo "Looked for:" >&2
  echo "  - third_party/openxla/pjrt/pjrt_c_api.h" >&2
  echo "  - expertnal_xla/pjrt_c_api.h.c" >&2
  exit 1
}

HEADER_PATH="$(resolve_header "${1:-}")"
OUT_PATH="${2:-${REPO_ROOT}/src/pjrt_bindings.rs}"

mkdir -p "$(dirname "${OUT_PATH}")"

echo "Generating bindings from: ${HEADER_PATH}"
echo "Writing bindings to: ${OUT_PATH}"

BINDGEN_HELP="$(bindgen --help 2>&1 || true)"

BINDGEN_ARGS=(
  "${HEADER_PATH}"
  --allowlist-type '^PJRT_.*'
  --allowlist-function '^PJRT_.*'
  --allowlist-var '^PJRT_.*'
  --allowlist-type '^size_t$'
  --ctypes-prefix libc
  --use-core
  --no-layout-tests
  --output "${OUT_PATH}"
)

if grep -q -- "--generate-comments" <<<"${BINDGEN_HELP}"; then
  BINDGEN_ARGS+=(--generate-comments)
fi

if grep -q -- "--formatter" <<<"${BINDGEN_HELP}"; then
  BINDGEN_ARGS+=(--formatter prettyplease)
fi

bindgen "${BINDGEN_ARGS[@]}" -- -x c -std=c11

