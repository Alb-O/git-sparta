#!/usr/bin/env bash
# setup_submodules.sh — unified first-time and subsequent submodule setup with precise sparse/LFS flow
# Adds per-developer, non-committed overrides for:
#   - SHARED_MIRROR_PATH (env or local JSON)
#   - SUBMODULE_URL (env or local JSON)
#
# - Scans ANY *.json in project root for required keys (committed config)
# - Then applies local overrides from any of these (if present):
#     ./*.local.json ./.project_local.json
#   (Recommend adding patterns like *.local.json to .gitignore.)
# - Finally, applies environment variable overrides (highest precedence):
#     SHARED_MIRROR_PATH, SUBMODULE_URL
#
# Idempotent; safe to re-run anytime.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LIB_DIR="$SCRIPT_DIR/lib"
SPARSE_LIST_RELATIVE="pipeline/generate_sparse_list.sh"

if [ ! -f "$LIB_DIR/common.sh" ]; then
  printf '%s: helper library missing at %s\n' "${0##*/}" "$LIB_DIR/common.sh" >&2
  exit 1
fi

# shellcheck source=lib/common.sh
source "$LIB_DIR/common.sh"

require_tools git jq sed

repo_git() {
  run_in_dir "$WORK_REPO" git "$@"
}

modules_git() {
  run_in_dir "$MODULES_PATH" git "$@"
}

module_git() {
  run_in_dir "$WORK_REPO" env GIT_DIR="$MODULES_PATH" GIT_WORK_TREE="$SUBMODULE_PATH" git "$@"
}

submodule_git() {
  run_in_dir "$SUBMODULE_PATH" git "$@"
}

# --- utilities ---------------------------------------------------------------
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

# --- determine git root directory --------------------------------------------
# If we're in a submodule, we need to find the superproject root
GIT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || oops "Not in a git repository"

# Check if we're in a submodule by looking for .git file (not directory)
if [ -f "$GIT_ROOT/.git" ]; then
  log "Detected submodule at $GIT_ROOT, finding superproject root..."
  # Try to get superproject root
  SUPER_ROOT="$(git rev-parse --show-superproject-working-tree 2>/dev/null)"
  if [ -n "$SUPER_ROOT" ]; then
    GIT_ROOT="$SUPER_ROOT"
    log "Using superproject root: $GIT_ROOT"
  else
    # Fallback: parse the gitdir path to find parent
    gitdir_line="$(cat "$GIT_ROOT/.git")"
    if [[ "$gitdir_line" =~ gitdir:\ (.+) ]]; then
      # gitdir might be like: ../../../.git/modules/foo/bar
      # Navigate up to find the actual .git directory
      relative_gitdir="${BASH_REMATCH[1]}"
      parent_git="$(cd "$GIT_ROOT" && cd "$relative_gitdir/../.." && pwd)"
      GIT_ROOT="$(dirname "$parent_git")"
      log "Derived superproject root from gitdir: $GIT_ROOT"
    fi
  fi
else
  log "Git repository root: $GIT_ROOT"
fi

# --- determine script and config location ------------------------------------
# Assume config is in parent directory of script (script in proj-scripts/, config in parent)
CONFIG_DIR="$(dirname "$SCRIPT_DIR")"
log "Script dir: $SCRIPT_DIR"
log "Config dir: $CONFIG_DIR"

# --- read BASE config from local JSON files ----------------------------------
SUBMODULE_NAME=""; SUBMODULE_PATH=""; SUBMODULE_URL=""; SUBMODULE_BRANCH=""; PROJECT_TAG=""; SHARED_MIRROR_PATH=""
CONFIG_FILE=""
shopt -s nullglob
for f in "$CONFIG_DIR"/*.json; do
  line="$(extract_first_match_all_keys "$f" || true)"
  if [ -n "$line" ]; then
    IFS=$'\t' read -r SUBMODULE_NAME SUBMODULE_PATH SUBMODULE_URL SUBMODULE_BRANCH PROJECT_TAG SHARED_MIRROR_PATH <<<"$line"
    CONFIG_FILE="$f"
    log "Loaded base config from $f"
    [ -n "$SHARED_MIRROR_PATH" ] && log "Base config includes SHARED_MIRROR_PATH=$SHARED_MIRROR_PATH"
    break
  fi
done
shopt -u nullglob

[ -n "$SUBMODULE_NAME" ]   || oops "config not found in any JSON: missing SUBMODULE_NAME"
[ -n "$SUBMODULE_PATH" ]   || oops "config not found: missing SUBMODULE_PATH"
[ -n "$SUBMODULE_URL" ]    || oops "config not found: missing SUBMODULE_URL"
[ -n "$SUBMODULE_BRANCH" ] || oops "config not found: missing SUBMODULE_BRANCH"
[ -n "$PROJECT_TAG" ]      || oops "config not found: missing PROJECT_TAG"
[ -n "$CONFIG_FILE" ]      || oops "config file not found"

# --- determine the working repository (where .gitmodules lives) -------------
# CONFIG_DIR is already set (the directory containing our config JSON)
# We'll work with .gitmodules in CONFIG_DIR, treating it as the repository root
WORK_REPO="$CONFIG_DIR"
log "Working repository: $WORK_REPO"

# --- convert SUBMODULE_PATH to absolute if relative -------------------------
# Paths in JSON are relative to the JSON file's directory (WORK_REPO)
if [[ "$SUBMODULE_PATH" != /* ]]; then
  # Relative path: resolve from config directory
  SUBMODULE_PATH="$WORK_REPO/$SUBMODULE_PATH"
  SUBMODULE_PATH="$(normalize_path <<<"$SUBMODULE_PATH")"
  log "Resolved SUBMODULE_PATH from config dir to absolute: $SUBMODULE_PATH"
else
  # Already absolute
  log "Using absolute SUBMODULE_PATH from config: $SUBMODULE_PATH"
fi

# Derive relative path from WORK_REPO (for .gitmodules and git commands within that repo)
SUBMODULE_PATH_RELATIVE="${SUBMODULE_PATH#$WORK_REPO/}"
log "SUBMODULE_PATH_RELATIVE (from work repo): $SUBMODULE_PATH_RELATIVE"

# --- apply LOCAL JSON overrides (non-committed) -----------------------------
# precedence within this block: first match wins across the file list order
apply_local_overrides() {
  local key val f
  for key in SUBMODULE_URL SHARED_MIRROR_PATH; do
    for f in "$CONFIG_DIR"/*.local.json "$CONFIG_DIR"/.project_local.json; do
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

if [ -n "${SHARED_MIRROR_PATH:-}" ]; then
  log "Env override: SHARED_MIRROR_PATH=$SHARED_MIRROR_PATH"
fi

# SHARED_MIRROR_PATH is now loaded from base config along with other values above

# --- decide on mirror usage --------------------------------------------------
use_reference=0
if [ -n "${SHARED_MIRROR_PATH:-}" ]; then
  if [ -d "$SHARED_MIRROR_PATH/.git" ]; then
    use_reference=1
    log "Using mirror: $SHARED_MIRROR_PATH"
  else
    log "SHARED_MIRROR_PATH set but invalid: $SHARED_MIRROR_PATH (ignoring)"
  fi
fi

# --- ensure BLENDERGLOB helper is present (if mirror is used) ----------------
if [ "$use_reference" = "1" ]; then
  helper_script="$SHARED_MIRROR_PATH/$SPARSE_LIST_RELATIVE"
  
  # Check if the script exists first
  if [ ! -f "$helper_script" ]; then
    oops "mirror missing helper script: $helper_script"
  fi
  
  # Check if it's executable
  if [ ! -x "$helper_script" ]; then
    oops "helper script exists but is not executable: $helper_script (run: chmod +x \"$helper_script\")"
  fi
fi

# --- first-time registration check ------------------------------------------
# Work with .gitmodules in the WORK_REPO directory
GITMODULES_FILE="$WORK_REPO/.gitmodules"
FIRST_TIME_SETUP=0

if ! git config --file "$GITMODULES_FILE" --get-regexp "submodule.${SUBMODULE_NAME}\.path" >/dev/null 2>&1; then
  log "Registering submodule in $GITMODULES_FILE"
  git config -f "$GITMODULES_FILE" "submodule.${SUBMODULE_NAME}.path"   "$SUBMODULE_PATH_RELATIVE"
  git config -f "$GITMODULES_FILE" "submodule.${SUBMODULE_NAME}.url"    "$SUBMODULE_URL"
  git config -f "$GITMODULES_FILE" "submodule.${SUBMODULE_NAME}.branch" "$SUBMODULE_BRANCH"
  repo_git add .gitmodules
  FIRST_TIME_SETUP=1
else
  # sanity-check path and update URL/branch if changed (supports per-dev URL override)
  # .gitmodules stores relative paths, so compare with relative version
  reg_path="$(git config --file "$GITMODULES_FILE" "submodule.${SUBMODULE_NAME}.path" || true)"
  [ "$reg_path" = "$SUBMODULE_PATH_RELATIVE" ] || oops "existing registration path '$reg_path' != '$SUBMODULE_PATH_RELATIVE'"

  reg_url="$(git config --file "$GITMODULES_FILE" "submodule.${SUBMODULE_NAME}.url" || true)"
  if [ "$reg_url" != "$SUBMODULE_URL" ]; then
    log "Updating .gitmodules URL: $reg_url -> $SUBMODULE_URL"
    git config -f "$GITMODULES_FILE" "submodule.${SUBMODULE_NAME}.url" "$SUBMODULE_URL"
    repo_git add .gitmodules
  fi

  reg_branch="$(git config --file "$GITMODULES_FILE" "submodule.${SUBMODULE_NAME}.branch" || true)"
  if [ -n "$SUBMODULE_BRANCH" ] && [ "$reg_branch" != "$SUBMODULE_BRANCH" ]; then
    log "Updating .gitmodules branch: $reg_branch -> $SUBMODULE_BRANCH"
    git config -f "$GITMODULES_FILE" "submodule.${SUBMODULE_NAME}.branch" "$SUBMODULE_BRANCH"
    repo_git add .gitmodules
  fi
fi

# --- build sparse list (from mirror if available; otherwise directly) --------
SPARSE_LIST_FILE="$(mktemp)"; trap 'rm -f "$SPARSE_LIST_FILE"' EXIT

if [ "$use_reference" = "1" ]; then
  log "Generating sparse list via mirror"
  env VERBOSE="${VERBOSE:-0}" "$SHARED_MIRROR_PATH/$SPARSE_LIST_RELATIVE" "$PROJECT_TAG" >"$SPARSE_LIST_FILE"
else
  log "No mirror: will generate sparse list *after* metadata clone"
fi

# --- init/update submodule metadata (no checkout), honoring per-dev URL ------
# Ensure submodule.$name.url in local .git/config reflects our effective URL, so update uses it.
# Work from the WORK_REPO directory
log "Syncing local .git config with effective submodule URL"
repo_git config "submodule.${SUBMODULE_NAME}.url" "$SUBMODULE_URL"

# Check if the submodule path exists in git's index
# If not, we need to add it first (even if .gitmodules exists)
GITLINK_EXISTS=0
if repo_git ls-files --stage "$SUBMODULE_PATH_RELATIVE" | grep -q '^160000'; then
  GITLINK_EXISTS=1
  log "Gitlink already exists in index"
else
  log "Submodule not in git index, adding gitlink manually..."
  
  # We need to create a gitlink entry without doing a checkout
  # First, fetch the remote to get the commit SHA
  mkdir -p "$SUBMODULE_PATH"
  
  # Initialize a minimal git repo at the submodule path
  if [ ! -d "$SUBMODULE_PATH/.git" ] && [ ! -f "$SUBMODULE_PATH/.git" ]; then
    submodule_git init -q
    submodule_git remote add origin "$SUBMODULE_URL"
    
    # If using mirror, set up alternates to use objects from mirror
    if [ "$use_reference" = "1" ]; then
      mkdir -p "$SUBMODULE_PATH/.git/objects/info"
      echo "$SHARED_MIRROR_PATH/.git/objects" > "$SUBMODULE_PATH/.git/objects/info/alternates"
      log "Configured git alternates to use mirror"
    fi
  fi
  
  # Fetch just the branch ref without checking out
  log "Fetching remote ref for branch: $SUBMODULE_BRANCH"
  submodule_git fetch --depth=1 origin "$SUBMODULE_BRANCH"
  
  # Get the commit SHA
  COMMIT_SHA="$(submodule_git rev-parse FETCH_HEAD)"
  log "Fetched commit: $COMMIT_SHA"
  
  # Remove the .git directory temporarily to add as gitlink
  rm -rf "$SUBMODULE_PATH/.git"
  
  # Create a gitlink entry in the index using git update-index
  repo_git update-index --add --cacheinfo 160000 "$COMMIT_SHA" "$SUBMODULE_PATH_RELATIVE"
  log "Added gitlink to index"
fi

mkdir -p "$SUBMODULE_PATH"
log "Initializing submodule metadata"
# git submodule init will set up the config without checking out
repo_git submodule init "$SUBMODULE_PATH_RELATIVE"

# Determine the modules path - it might be in parent's modules or already exist
if [ -f "$SUBMODULE_PATH/.git" ]; then
  # Extract gitdir path from .git file
  GITDIR_LINE="$(cat "$SUBMODULE_PATH/.git")"
  if [[ "$GITDIR_LINE" =~ gitdir:\ (.+) ]]; then
    RELATIVE_GITDIR="${BASH_REMATCH[1]}"
    MODULES_PATH="$(cd "$SUBMODULE_PATH" && cd "$RELATIVE_GITDIR" && pwd)"
    log "Found existing git directory at: $MODULES_PATH"
  fi
else
  # Create new modules path
  MODULES_PATH="$GIT_ROOT/.git/modules/$SUBMODULE_PATH_RELATIVE"
  log "Creating new git directory at: $MODULES_PATH"
fi

# Ensure the modules directory is properly configured
if [ -d "$MODULES_PATH" ]; then
  log "Configuring git directory: $MODULES_PATH"

  modules_git config core.bare false
  modules_git config core.worktree "$SUBMODULE_PATH"
  log "Set core.bare=false and core.worktree=$SUBMODULE_PATH"

  if [ "$use_reference" = "1" ]; then
    mkdir -p "$MODULES_PATH/objects/info"
    if ! grep -q "$SHARED_MIRROR_PATH/.git/objects" "$MODULES_PATH/objects/info/alternates" 2>/dev/null; then
      echo "$SHARED_MIRROR_PATH/.git/objects" >> "$MODULES_PATH/objects/info/alternates"
      log "Configured alternates for mirror"
    fi
  fi

  if ! modules_git remote get-url origin >/dev/null 2>&1; then
    modules_git remote add origin "$SUBMODULE_URL"
    log "Added remote origin"
  fi

  COMMIT_SHA="$(repo_git ls-files --stage "$SUBMODULE_PATH_RELATIVE" | awk '{print $2}')"
  if [ -n "$COMMIT_SHA" ]; then
    if ! modules_git cat-file -e "$COMMIT_SHA" 2>/dev/null; then
      log "Fetching commit $COMMIT_SHA"
      modules_git fetch --depth=1 origin "$SUBMODULE_BRANCH"
    else
      log "Commit $COMMIT_SHA already exists in repository"
    fi

    if [ ! -f "$MODULES_PATH/HEAD" ] || ! grep -q "^$COMMIT_SHA" "$MODULES_PATH/HEAD" 2>/dev/null; then
      log "Setting HEAD to $COMMIT_SHA"
      modules_git update-ref HEAD "$COMMIT_SHA"
    fi
  else
    log "Warning: No commit SHA found in gitlink, fetching branch head"
    modules_git fetch --depth=1 origin "$SUBMODULE_BRANCH"
    modules_git update-ref HEAD FETCH_HEAD
  fi
else
  log "Git directory does not exist, creating: $MODULES_PATH"
  mkdir -p "$(dirname "$MODULES_PATH")"
  git init -q "$MODULES_PATH"
  modules_git remote add origin "$SUBMODULE_URL"
  modules_git config core.bare false
  modules_git config core.worktree "$SUBMODULE_PATH"

  RELATIVE_MODULES="$(realpath --relative-to="$SUBMODULE_PATH" "$MODULES_PATH")"
  echo "gitdir: $RELATIVE_MODULES" > "$SUBMODULE_PATH/.git"
  log "Created .git file pointing to: $RELATIVE_MODULES"

  log "Fetching branch $SUBMODULE_BRANCH"
  modules_git fetch --depth=1 origin "$SUBMODULE_BRANCH"
  modules_git update-ref HEAD FETCH_HEAD
fi

# If submodule dir exists and has a git repo, ensure its origin URL matches our effective SUBMODULE_URL
if [ -d "$SUBMODULE_PATH/.git" ]; then
  submodule_git remote set-url origin "$SUBMODULE_URL" || :
fi

# If no mirror, generate sparse patterns using helper inside the submodule
if [ "$use_reference" = "0" ]; then
  if [ -x "$SUBMODULE_PATH/$SPARSE_LIST_RELATIVE" ]; then
    log "Generating sparse list from submodule clone"
    run_in_dir "$SUBMODULE_PATH" env VERBOSE="${VERBOSE:-0}" "$SUBMODULE_PATH/$SPARSE_LIST_RELATIVE" "$PROJECT_TAG" >"$SPARSE_LIST_FILE"
  else
    oops "$SPARSE_LIST_RELATIVE not found in submodule and mirror not set"
  fi
fi

# --- apply sparse patterns, checkout, pull LFS --------------------------------
log "Applying sparse-checkout rules"

# Work from outside the submodule to avoid directory deletion issues
log "Configuring sparse checkout in git directory: $MODULES_PATH"
modules_git config core.sparseCheckout true
run_in_dir "$MODULES_PATH" mkdir -p info
run_in_dir "$MODULES_PATH" cp "$SPARSE_LIST_FILE" info/sparse-checkout

pattern_count="$(run_in_dir "$MODULES_PATH" awk 'END {print NR}' info/sparse-checkout)"
log "Patterns count: $pattern_count"

log "First 5 patterns:"
mapfile -t __pattern_preview < <(run_in_dir "$MODULES_PATH" head -5 info/sparse-checkout)
for line in "${__pattern_preview[@]}"; do
  log "  $line"
done

log "Updating working tree with sparse patterns"
log "Running from: $WORK_REPO"
module_git read-tree -mu HEAD
log "Working tree updated"

# LFS fetch/checkout from outside to avoid directory issues while respecting sparse state
if have git-lfs; then
  log "Fetching LFS objects without triggering smudge"
  GIT_LFS_SKIP_SMUDGE=1 module_git lfs fetch

  __lfs_paths=()
  mapfile -t __lfs_paths < <(module_git lfs ls-files --name-only)

  if [ "${#__lfs_paths[@]}" -gt 0 ]; then
    included_lfs_paths=()
    for __path in "${__lfs_paths[@]}"; do
      entry="$(module_git ls-files -v -- "$__path")"

      [ -n "$entry" ] || continue
      prefix="${entry%% *}"
      if [ "$prefix" != "S" ]; then
        included_lfs_paths+=("$__path")
      fi
    done

    if [ "${#included_lfs_paths[@]}" -gt 0 ]; then
      log "Checking out ${#included_lfs_paths[@]} LFS file(s) within sparse checkout"
      module_git lfs checkout -- "${included_lfs_paths[@]}"
    else
      log "No LFS paths intersect sparse checkout; skipping checkout step"
    fi
  else
    log "No LFS-tracked paths detected"
  fi
else
  log "git-lfs not installed; skipping LFS fetch (pointer files remain)"
fi

# --- wrap-up -----------------------------------------------------------------
if [ "$FIRST_TIME_SETUP" -eq 1 ]; then
  printf '\n'
  printf '✓ First-time setup done. Staged .gitmodules.\n'
  printf 'Now stage the submodule entry and commit:\n'
  printf '  git add %s\n' "$SUBMODULE_PATH_RELATIVE"
  printf '  git commit -m "feat: add and configure %s submodule"\n' "$SUBMODULE_NAME"
else
  printf '✓ Submodule up to date for tag "%s".\n' "$PROJECT_TAG"
fi
