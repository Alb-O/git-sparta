#!/usr/bin/env bash
# setup_submodules.sh — unified first-time and subsequent submodule setup with precise sparse/LFS flow
# Adds per-developer, non-committed overrides for:
#   - BLENDERGLOB_MIRROR_PATH (env or local JSON)
#   - SUBMODULE_URL (env or local JSON)
#
# - Scans ANY *.json in project root for required keys (committed config)
# - Then applies local overrides from any of these (if present):
#     ./*.local.json  ./*.dev.json  ./.project_local.json
#   (Recommend adding patterns like *.local.json and *.dev.json to .gitignore.)
# - Finally, applies environment variable overrides (highest precedence):
#     BLENDERGLOB_MIRROR_PATH, SUBMODULE_URL
#
# Idempotent; safe to re-run anytime.
set -euo pipefail

# --- common helpers ----------------------------------------------------------
log()  { [ "${VERBOSE:-0}" = "1" ] && printf '%s\n' "$*" >&2 || :; }
oops() { printf '%s: %s\n' "${0##*/}" "$*" >&2; exit 1; }
have() { command -v "$1" >/dev/null 2>&1; }

have git || oops "git not found"
have jq  || oops "jq not found (needed to read project JSON config)"
have sed || oops "sed not found"

# --- utilities ---------------------------------------------------------------
# Extract the FIRST object anywhere in a JSON file that has ALL required keys.
extract_first_match_all_keys() {
  local file="$1"
  jq -r '
    .. | objects
    | select(
        has("SUBMODULE_NAME") and has("SUBMODULE_PATH") and has("SUBMODULE_URL") and has("SUBMODULE_BRANCH") and has("PROJECT_TAG")
      )
    | [.SUBMODULE_NAME, .SUBMODULE_PATH, .SUBMODULE_URL, .SUBMODULE_BRANCH, .PROJECT_TAG]
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

# --- read BASE config from any root-level JSON ------------------------------
SUBMODULE_NAME=""; SUBMODULE_PATH=""; SUBMODULE_URL=""; SUBMODULE_BRANCH=""; PROJECT_TAG=""
shopt -s nullglob
for f in ./*.json; do
  line="$(extract_first_match_all_keys "$f" || true)"
  if [ -n "$line" ]; then
    IFS=$'\t' read -r SUBMODULE_NAME SUBMODULE_PATH SUBMODULE_URL SUBMODULE_BRANCH PROJECT_TAG <<<"$line"
    log "Loaded base config from $f"
    break
  fi
done
shopt -u nullglob

[ -n "$SUBMODULE_NAME" ]   || oops "config not found in any root JSON: missing SUBMODULE_NAME"
[ -n "$SUBMODULE_PATH" ]   || oops "config not found: missing SUBMODULE_PATH"
[ -n "$SUBMODULE_URL" ]    || oops "config not found: missing SUBMODULE_URL"
[ -n "$SUBMODULE_BRANCH" ] || oops "config not found: missing SUBMODULE_BRANCH"
[ -n "$PROJECT_TAG" ]      || oops "config not found: missing PROJECT_TAG"

# --- apply LOCAL JSON overrides (non-committed) -----------------------------
# precedence within this block: first match wins across the file list order
apply_local_overrides() {
  local key val f
  for key in SUBMODULE_URL BLENDERGLOB_MIRROR_PATH; do
    for f in ./*.local.json ./*.dev.json ./.project_local.json; do
      [ -f "$f" ] || continue
      val="$(extract_first_key "$f" "$key" || true)"
      if [ -n "$val" ] && [ "$val" != "null" ]; then
        printf -v "$key" '%s' "$val"
        log "Override from $f: $key=$val"
        break
      fi
    done
  done
}
apply_local_overrides

# --- apply ENV overrides (highest precedence) -------------------------------
if [ -n "${SUBMODULE_URL:-}" ]; then
  log "Env override: SUBMODULE_URL=$SUBMODULE_URL"
else
  # if no env, ensure variable remains the one possibly set by local override
  :
fi

if [ -n "${BLENDERGLOB_MIRROR_PATH:-}" ]; then
  log "Env override: BLENDERGLOB_MIRROR_PATH=$BLENDERGLOB_MIRROR_PATH"
fi

# Also allow mirror path to be present inside *committed* JSONs (fallback), if not set yet.
if [ -z "${BLENDERGLOB_MIRROR_PATH:-}" ]; then
  JSON_MIRROR="$(jq -r '.. | objects | select(has("BLENDERGLOB_MIRROR_PATH")) | .BLENDERGLOB_MIRROR_PATH' ./*.json 2>/dev/null | head -n1 || true)"
  if [ -n "$JSON_MIRROR" ] && [ "$JSON_MIRROR" != "null" ]; then
    BLENDERGLOB_MIRROR_PATH="$JSON_MIRROR"
    log "Mirror from base JSON: BLENDERGLOB_MIRROR_PATH=$BLENDERGLOB_MIRROR_PATH"
  fi
fi

# --- decide on mirror usage --------------------------------------------------
use_reference=0
if [ -n "${BLENDERGLOB_MIRROR_PATH:-}" ]; then
  if [ -d "$BLENDERGLOB_MIRROR_PATH/.git" ]; then
    use_reference=1
    log "Using mirror: $BLENDERGLOB_MIRROR_PATH"
  else
    log "BLENDERGLOB_MIRROR_PATH set but invalid: $BLENDERGLOB_MIRROR_PATH (ignoring)"
  fi
fi

# --- ensure BLENDERGLOB helper is present (if mirror is used) ----------------
if [ "$use_reference" = "1" ]; then
  [ -x "$BLENDERGLOB_MIRROR_PATH/scripts/generate_sparse_list.sh" ] \
    || oops "mirror missing helper script: $BLENDERGLOB_MIRROR_PATH/scripts/generate_sparse_list.sh"
fi

# --- first-time registration check ------------------------------------------
FIRST_TIME_SETUP=0
if ! git config --file .gitmodules --get-regexp "submodule.${SUBMODULE_NAME}\.path" >/dev/null 2>&1; then
  log "Registering submodule in .gitmodules"
  git config -f .gitmodules "submodule.${SUBMODULE_NAME}.path"   "$SUBMODULE_PATH"
  git config -f .gitmodules "submodule.${SUBMODULE_NAME}.url"    "$SUBMODULE_URL"
  git config -f .gitmodules "submodule.${SUBMODULE_NAME}.branch" "$SUBMODULE_BRANCH"
  git add .gitmodules
  FIRST_TIME_SETUP=1
else
  # sanity-check path and update URL/branch if changed (supports per-dev URL override)
  reg_path="$(git config --file .gitmodules "submodule.${SUBMODULE_NAME}.path" || true)"
  [ "$reg_path" = "$SUBMODULE_PATH" ] || oops "existing registration path '$reg_path' != '$SUBMODULE_PATH'"

  reg_url="$(git config --file .gitmodules "submodule.${SUBMODULE_NAME}.url" || true)"
  if [ "$reg_url" != "$SUBMODULE_URL" ]; then
    log "Updating .gitmodules URL: $reg_url -> $SUBMODULE_URL"
    git config -f .gitmodules "submodule.${SUBMODULE_NAME}.url" "$SUBMODULE_URL"
    git add .gitmodules
  fi

  reg_branch="$(git config --file .gitmodules "submodule.${SUBMODULE_NAME}.branch" || true)"
  if [ -n "$SUBMODULE_BRANCH" ] && [ "$reg_branch" != "$SUBMODULE_BRANCH" ]; then
    log "Updating .gitmodules branch: $reg_branch -> $SUBMODULE_BRANCH"
    git config -f .gitmodules "submodule.${SUBMODULE_NAME}.branch" "$SUBMODULE_BRANCH"
    git add .gitmodules
  fi
fi

# --- build sparse list (from mirror if available; otherwise directly) --------
SPARSE_LIST_FILE="$(mktemp)"; trap 'rm -f "$SPARSE_LIST_FILE"' EXIT

if [ "$use_reference" = "1" ]; then
  log "Generating sparse list via mirror"
  ( VERBOSE="${VERBOSE:-0}" "$BLENDERGLOB_MIRROR_PATH/scripts/generate_sparse_list.sh" "$PROJECT_TAG" ) >"$SPARSE_LIST_FILE"
else
  log "No mirror: will generate sparse list *after* metadata clone"
fi

# --- init/update submodule metadata (no checkout), honoring per-dev URL ------
# Ensure submodule.$name.url in local .git/config reflects our effective URL, so update uses it.
log "Syncing local .git config with effective submodule URL"
git config "submodule.${SUBMODULE_NAME}.url" "$SUBMODULE_URL"

log "Initializing submodule metadata (no checkout)"
if [ "$use_reference" = "1" ]; then
  git submodule update --init --depth 1 --no-checkout \
    --reference "$BLENDERGLOB_MIRROR_PATH" \
    "$SUBMODULE_PATH"
else
  git submodule update --init --depth 1 --no-checkout "$SUBMODULE_PATH"
fi

# If submodule dir exists and has a git repo, ensure its origin URL matches our effective SUBMODULE_URL
if [ -d "$SUBMODULE_PATH/.git" ]; then
  ( cd "$SUBMODULE_PATH" && git remote set-url origin "$SUBMODULE_URL" ) || :
fi

# If no mirror, generate sparse patterns using helper inside the submodule
if [ "$use_reference" = "0" ]; then
  if [ -x "$SUBMODULE_PATH/scripts/generate_sparse_list.sh" ]; then
    log "Generating sparse list from submodule clone"
    ( cd "$SUBMODULE_PATH" && VERBOSE="${VERBOSE:-0}" ./scripts/generate_sparse_list.sh "$PROJECT_TAG" ) >"$SPARSE_LIST_FILE"
  else
    oops "generate_sparse_list.sh not found in submodule and mirror not set"
  fi
fi

# --- apply sparse patterns, checkout, pull LFS --------------------------------
log "Applying sparse-checkout rules"
(
  cd "$SUBMODULE_PATH"

  git sparse-checkout init --no-cone >/dev/null 2>&1 || :
  git sparse-checkout set --stdin < "$SPARSE_LIST_FILE"
  git checkout

  if have git-lfs; then
    git lfs pull
  else
    log "git-lfs not installed; skipping LFS pull (pointer files remain)"
  fi
)

# --- wrap-up -----------------------------------------------------------------
if [ "$FIRST_TIME_SETUP" -eq 1 ]; then
  printf '\n'
  printf '✓ First-time setup done. Staged .gitmodules.\n'
  printf 'Now stage the submodule entry and commit:\n'
  printf '  git add %s\n' "$SUBMODULE_PATH"
  printf '  git commit -m "feat: add and configure %s submodule"\n' "$SUBMODULE_NAME"
else
  printf '✓ Submodule up to date for tag "%s".\n' "$PROJECT_TAG"
fi
