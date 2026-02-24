#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="${SCRIPT_DIR}"

usage() {
  cat <<'USAGE'
Usage:
  ./gen_bindings.sh [stable_header_path] [output_path] [latest_header_path]

Defaults:
  stable_header_path:
    1) third_party/openxla/pjrt/pjrt_c_api.h
    2) expertnal_xla/pjrt_c_api.h.c
  output_path:
    crates/rrad_pjrt/src/ffi/pjrt_bindings.rs
  latest_header_path:
    optional; if omitted, stable header is used for both sides of diff

Examples:
  ./gen_bindings.sh
  ./gen_bindings.sh xla/xla/pjrt/c/pjrt_c_api.h crates/rrad_pjrt/src/ffi/pjrt_bindings.rs xla_latest/xla/pjrt/c/pjrt_c_api.h
USAGE
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

BINDGEN_HELP="$(bindgen --help 2>&1 || true)"

run_bindgen() {
  local header_path="$1"
  local out_path="$2"

  local -a args=(
    "${header_path}"
    --allowlist-type '^PJRT_.*'
    --allowlist-function '^PJRT_.*'
    --allowlist-var '^PJRT_.*'
    --allowlist-type '^size_t$'
    --ctypes-prefix libc
    --use-core
    --no-layout-tests
    --output "${out_path}"
  )

  if grep -q -- "--generate-comments" <<<"${BINDGEN_HELP}"; then
    args+=(--generate-comments)
  fi

  if grep -q -- "--formatter" <<<"${BINDGEN_HELP}"; then
    args+=(--formatter prettyplease)
  fi

  bindgen "${args[@]}" -- -x c -std=c11
}

STABLE_HEADER_PATH="$(resolve_header "${1:-}")"
OUT_PATH="${2:-${REPO_ROOT}/crates/rrad_pjrt/src/ffi/pjrt_bindings.rs}"
LATEST_HEADER_PATH="${3:-}"

mkdir -p "$(dirname "${OUT_PATH}")"

echo "Generating stable bindings from: ${STABLE_HEADER_PATH}"
echo "Writing stable bindings to: ${OUT_PATH}"
run_bindgen "${STABLE_HEADER_PATH}" "${OUT_PATH}"

latest_bindings_tmp=""
if [[ -n "${LATEST_HEADER_PATH}" ]]; then
  if [[ ! -f "${LATEST_HEADER_PATH}" ]]; then
    echo "Error: latest header not found: ${LATEST_HEADER_PATH}" >&2
    exit 1
  fi

  latest_bindings_tmp="$(mktemp)"
  trap '[[ -n "${latest_bindings_tmp}" && -f "${latest_bindings_tmp}" ]] && rm -f "${latest_bindings_tmp}"' EXIT

  echo "Generating latest comparison bindings from: ${LATEST_HEADER_PATH}"
  run_bindgen "${LATEST_HEADER_PATH}" "${latest_bindings_tmp}"
else
  latest_bindings_tmp="${OUT_PATH}"
fi

"${REPO_ROOT}/crates/rrad_pjrt/scripts/update_pjrt_binding_diff.sh" \
  "${OUT_PATH}" \
  "${latest_bindings_tmp}" \
  "${REPO_ROOT}/crates/rrad_pjrt/PJRT_BINDING_DIFF.md"
