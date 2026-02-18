#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

resolve_pjrt_plugin_path() {
  local explicit_path="${1:-}"

  if [[ -n "${explicit_path}" ]]; then
    if [[ -f "${explicit_path}" ]]; then
      printf '%s\n' "${explicit_path}"
      return 0
    fi
    echo "error: PJRT plugin not found at '${explicit_path}'" >&2
    return 1
  fi

  if [[ -n "${PJRT_PLUGIN:-}" && -f "${PJRT_PLUGIN}" ]]; then
    printf '%s\n' "${PJRT_PLUGIN}"
    return 0
  fi

  local candidate
  for candidate in \
    "${REPO_ROOT}/xla/bazel-bin/xla/pjrt/c/pjrt_c_api_cpu_plugin.so" \
    "${REPO_ROOT}/xla/bazel-bin/xla/pjrt/c/pjrt_c_api_cpu_plugin.dylib" \
    "${REPO_ROOT}/xla/bazel-bin/xla/pjrt/c/pjrt_c_api_cpu_plugin"
  do
    if [[ -f "${candidate}" ]]; then
      printf '%s\n' "${candidate}"
      return 0
    fi
  done

  if [[ -d "${REPO_ROOT}/xla/bazel-bin" ]]; then
    local discovered
    discovered="$(find "${REPO_ROOT}/xla/bazel-bin" -type f \( -name 'pjrt_c_api_cpu_plugin.so' -o -name 'pjrt_c_api_cpu_plugin.dylib' \) 2>/dev/null | head -n 1 || true)"
    if [[ -n "${discovered}" ]]; then
      printf '%s\n' "${discovered}"
      return 0
    fi
  fi

  cat >&2 <<'ERR'
error: could not locate the PJRT CPU plugin artifact.
build it first:
  (cd xla && bazel build //xla/pjrt/c:pjrt_c_api_cpu_plugin.so)
ERR
  return 1
}
