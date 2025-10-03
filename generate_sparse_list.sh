#!/usr/bin/env bash
# generate_sparse_list.sh — echo newline-separated sparse-checkout patterns for a given project tag
# Usage: ./scripts/generate_sparse_list.sh P0010-snb
# Interactive confirmation shows which tags were matched (including "global").
# This version allows SUBSTRING matches for the user-provided TAG (e.g., TAG=P0010 matches token P0010-snb).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
LIB_DIR="$SCRIPT_DIR/lib"

if [ ! -f "$LIB_DIR/common.sh" ] || [ ! -f "$LIB_DIR/tui.sh" ]; then
  printf '%s: helper libraries missing under %s\n' "${0##*/}" "$LIB_DIR" >&2
  exit 1
fi

# shellcheck source=lib/common.sh
source "$LIB_DIR/common.sh"
# shellcheck source=lib/tui.sh
source "$LIB_DIR/tui.sh"

TAG="${1-}"; [ -n "$TAG" ] || oops "missing project tag argument"

require_tools git awk

# Find the repo root relative to the script location, not the current directory
REPO_ROOT="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel 2>/dev/null || true)"
[ -n "$REPO_ROOT" ] || oops "script not inside a git repo"

log "Scanning git attributes for tag '$TAG' (and 'global') in repo: $REPO_ROOT"

repo_git() {
  run_in_dir "$REPO_ROOT" git "$@"
}

# Use git check-attr --all to query all attributes for all tracked files.
# This is more robust and efficient than querying each file individually.
generate_matches() {
  # Run git check-attr --all from the repo root, reading all tracked files
  repo_git ls-files -z | \
    repo_git check-attr --stdin -z --all | \
    awk -v tag="$TAG" -v RS='\0' '
      # Format: <file>\0<attr>\0<value>\0<file>\0<attr>\0<value>...
      # We get triplets: filename, attribute name, attribute value
      NR % 3 == 1 { file = $0 }
      NR % 3 == 2 { attr = $0 }
      NR % 3 == 0 {
        if (attr == "projects" && $0 != "unspecified" && $0 != "unset") {
          # Split comma-separated project tags
          n = split($0, arr, /,/)
          for (i = 1; i <= n; i++) {
            t = arr[i]
            # Trim whitespace
            gsub(/^[ \t]+|[ \t]+$/, "", t)
            # Match if token is "global" or contains TAG as substring
            if (t == "global" || index(t, tag) > 0) {
              print file "\t" t
            }
          }
        }
      }
    ' | sort -u
}

# Collect matches
matches="$(generate_matches || true)"
[ -n "$matches" ] || oops "no matching entries found for tag '$TAG'"

# Compute the set of matched tags (unique, one per line)
matched_tags_lines="$(printf '%s\n' "$matches" | awk -F '\t' '{print $2}' | awk 'NF' | sort -u)"
matched_count="$(printf '%s\n' "$matched_tags_lines" | awk 'NF{c++} END{print c+0}')"
pattern_count="$(printf '%s\n' "$matches" | awk -F '\t' '{print $1}' | awk 'NF' | sort -u | awk 'END{print NR+0}')"

tui_init
tui_divider
tui_heading "Matched tags for input tag: $TAG"
tui_note "(including \"global\" when present)"
tui_label_value "Tags" "$matched_count"
if [ -n "$matched_tags_lines" ]; then
  printf '%s\n' "$matched_tags_lines" | tui_bullet_list
else
  tui_note "  <none>"
fi
tui_label_value "Patterns" "$pattern_count"
tui_divider

if ! tui_confirm "Proceed?" "N"; then
  tui_abort "Aborted by user."
fi

# Print unique patterns only (for piping to: git sparse-checkout set --stdin)
printf '%s\n' "$matches" | awk -F '\t' '{print $1}' | awk 'NF' | sort -u
