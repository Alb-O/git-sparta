#!/usr/bin/env bash
# teardown_submodule.sh — remove the configured sparse submodule completely.
# Ensures the working tree is clean, removes git metadata, and deletes the
# working directory so the superproject no longer references the submodule.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LIB_DIR="$SCRIPT_DIR/lib"

if [ ! -f "$LIB_DIR/common.sh" ] || [ ! -f "$LIB_DIR/tui.sh" ]; then
  printf '%s: helper libraries missing under %s\n' "${0##*/}" "$LIB_DIR" >&2
  exit 1
fi

# shellcheck source=lib/common.sh
source "$LIB_DIR/common.sh"
# shellcheck source=lib/tui.sh
source "$LIB_DIR/tui.sh"

require_tools git jq sed rm

# --- helper wrappers ---------------------------------------------------------
repo_git() {
  run_in_dir "$WORK_REPO" git "$@"
}

module_git() {
  run_in_dir "$WORK_REPO" env GIT_DIR="$MODULES_PATH" GIT_WORK_TREE="$SUBMODULE_PATH" git "$@"
}

submodule_tracked()
{
  repo_git ls-files --stage "$SUBMODULE_PATH_RELATIVE" | grep -q '^160000'
}

# --- load config -------------------------------------------------------------
GIT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || oops "Not in a git repository"

CONFIG_DIR="$(dirname "$SCRIPT_DIR")"
WORK_REPO="$CONFIG_DIR"

SUBMODULE_NAME=""; SUBMODULE_PATH=""; SUBMODULE_URL=""; SUBMODULE_BRANCH=""; PROJECT_TAG=""; SHARED_MIRROR_PATH=""
CONFIG_FILE=""

shopt -s nullglob
for f in "$CONFIG_DIR"/*.json; do
  line="$(extract_first_match_all_keys "$f" || true)"
  if [ -n "$line" ]; then
    IFS=$'\t' read -r SUBMODULE_NAME SUBMODULE_PATH SUBMODULE_URL SUBMODULE_BRANCH PROJECT_TAG SHARED_MIRROR_PATH <<<"$line"
    CONFIG_FILE="$f"
    break
  fi
done
shopt -u nullglob

[ -n "$SUBMODULE_NAME" ]   || oops "config not found: missing SUBMODULE_NAME"
[ -n "$SUBMODULE_PATH" ]   || oops "config not found: missing SUBMODULE_PATH"
[ -n "$SUBMODULE_BRANCH" ] || oops "config not found: missing SUBMODULE_BRANCH"

if [[ "$SUBMODULE_PATH" != /* ]]; then
  SUBMODULE_PATH="$WORK_REPO/$SUBMODULE_PATH"
  SUBMODULE_PATH="$(normalize_path <<<"$SUBMODULE_PATH")"
fi
SUBMODULE_PATH_RELATIVE="${SUBMODULE_PATH#$WORK_REPO/}"
MODULES_PATH="$GIT_ROOT/.git/modules/$SUBMODULE_PATH_RELATIVE"
GITMODULES_FILE="$WORK_REPO/.gitmodules"

was_tracked=0
if repo_git ls-files --stage "$SUBMODULE_PATH_RELATIVE" | grep -q '^160000'; then
  was_tracked=1
fi

# --- safety checks -----------------------------------------------------------
if [ ! -f "$GITMODULES_FILE" ]; then
  oops ".gitmodules not found at $GITMODULES_FILE"
fi

registered_path="$(git config --file "$GITMODULES_FILE" "submodule.${SUBMODULE_NAME}.path" || true)"
if [ -z "$registered_path" ]; then
  oops "submodule ${SUBMODULE_NAME} is not registered in .gitmodules"
fi

if [ "$registered_path" != "$SUBMODULE_PATH_RELATIVE" ]; then
  oops "submodule path mismatch: expected $registered_path, computed $SUBMODULE_PATH_RELATIVE"
fi

if [ -d "$MODULES_PATH" ] && module_git rev-parse --git-dir >/dev/null 2>&1; then
  status_output="$(module_git status --short 2>/dev/null || true)"
  if [ -n "$status_output" ]; then
    tui_init
    tui_heading "Submodule worktree not clean"
    tui_note "Cleaning is required before teardown."
    printf '%s\n' "$status_output" >&2
    exit 1
  fi
fi

if [ -d "$SUBMODULE_PATH" ] && [ -n "$(ls -A "$SUBMODULE_PATH" 2>/dev/null || true)" ]; then
  if [ -f "$SUBMODULE_PATH/.git" ] && [ -d "$MODULES_PATH" ]; then
    :
  elif [ -d "$SUBMODULE_PATH/.git" ]; then
    oops "Unexpected git directory layout under $SUBMODULE_PATH (nested repo?)"
  fi
fi

if ! tui_confirm "Remove submodule at $SUBMODULE_PATH_RELATIVE?" "N"; then
  tui_abort "Aborted by user."
fi

# --- removal ----------------------------------------------------------------
log "Removing submodule entry from .gitmodules"
repo_git config -f "$GITMODULES_FILE" --remove-section "submodule.${SUBMODULE_NAME}" || true
if ! repo_git diff --quiet -- .gitmodules; then
  repo_git add .gitmodules
fi

log "Removing submodule config from .git/config"
repo_git config --remove-section "submodule.${SUBMODULE_NAME}" 2>/dev/null || true

if submodule_tracked; then
  log "Removing gitlink from index"
  repo_git update-index --force-remove -- "$SUBMODULE_PATH_RELATIVE"
fi

log "Deleting worktree directory"
rm -rf "$SUBMODULE_PATH"

if [ -d "$MODULES_PATH" ]; then
  log "Deleting modules git directory"
  rm -rf "$MODULES_PATH"
fi

# Clean up empty parent directories under modules
parent_dir="$MODULES_PATH"
while [ "$parent_dir" != "$GIT_ROOT/.git/modules" ] && [ -n "$parent_dir" ]; do
  parent_dir="$(dirname "$parent_dir")"
  [ -d "$parent_dir" ] || continue
  if [ -z "$(ls -A "$parent_dir" 2>/dev/null || true)" ]; then
    rmdir "$parent_dir"
  else
    break
  fi
done

log "Submodule teardown complete"
if [ "$was_tracked" = "1" ]; then
  log "Staging removal of submodule path"
  repo_git add -A -- "$SUBMODULE_PATH_RELATIVE"
fi
printf '✓ Removed submodule %s at %s\n' "$SUBMODULE_NAME" "$SUBMODULE_PATH_RELATIVE"
