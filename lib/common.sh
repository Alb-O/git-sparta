#!/usr/bin/env bash
# Common helpers for submodule-related shell scripts.

log() {
  [ "${VERBOSE:-0}" = "1" ] && printf '%s\n' "$*" >&2 || :
}

oops() {
  printf '%s: %s\n' "${0##*/}" "$*" >&2
  exit 1
}

have() {
  command -v "$1" >/dev/null 2>&1
}

require_tools() {
  local tool
  for tool in "$@"; do
    have "$tool" || oops "required tool missing: $tool"
  done
}

# Extract the FIRST object anywhere in a JSON file that has ALL required keys.
extract_first_match_all_keys() {
  local file="$1"
  jq -r '
    .. | objects
    | select(
        has("SUBMODULE_NAME") and has("SUBMODULE_PATH") and has("SUBMODULE_URL") and has("SUBMODULE_BRANCH") and has("PROJECT_TAG")
      )
    | [.SUBMODULE_NAME, .SUBMODULE_PATH, .SUBMODULE_URL, .SUBMODULE_BRANCH, .PROJECT_TAG, (.SHARED_MIRROR_PATH // "")]
    | @tsv
  ' "$file" 2>/dev/null | head -n1
}

# Extract FIRST occurrence of a single key anywhere in a JSON file.
extract_first_key() {
  local file="$1" key="$2"
  jq -r --arg k "$key" '
    .. | objects | select(has($k)) | .[$k]
  ' "$file" 2>/dev/null | head -n1
}

# Normalize path (collapse // -> /)
normalize_path() {
  sed 's#//\+#/#g'
}

run_in_dir() {
  local dir="$1"
  shift
  ( cd "$dir" && "$@" )
}
