# git-sparta

Git Sparse Tagging — sparse checkouts of repositories and submodules based on git attributes.

## Installation

```bash
cargo install --path .
```

Once installed, use as a git subcommand:

```bash
git sparta --help
```

> Git automatically discovers executables named `git-<name>` in your PATH as subcommands.

This is intended for the specific scenario when you have a huge, storage-heavy repo of global assets (we'll refer to this as the 'LFS repo') that are shared as dependencies between multiple separate projects. git and git-lfs were not built for this workflow style. There are multiple workarounds, each with their drawbacks.

## The Problem

- For example, say we have a few small production projects, and they depend on large binary assets, e.g. short animated clips, each linking to heavy model files in the LFS repo. You could simply reference the asset in place from a shared path (a NAS mount or a single “golden” local repo). The problem with this approach is that the project is now dependent on brittle links, which are linking to whatever happens to be in that path on a given day. Dependencies change over time and silently break previous projects that were linked to old versions of the assets.

- At the opposite extreme, you can vendor/copy the LFS repo into each project. You pay a large toll in redundant storage, and the burden of manually syncing updates between copied LFS repos if the 'source of truth' of an asset changes and therefore the consumers must also be updated.

- A slightly better solution is to add the LFS repo as a git submodule to each project. This gives you a pinned commit and predictable checkouts, but you still pay for the size of the repository: git will pull object history and hydrate far more than the one project needs. Large working trees slow things down, digging around the entire LFS repo becomes a chore when only a small slice of assets is relevant.

- You could try more sophisticated git methods like using subtrees or manually configuring sparse-checkouts. This works assuming directory organisation in the LFS repo is perfectly separated based on project subtrees, i.e. project folders that contain only assets for that single project, but this is unrealistically optimistic. Assets get moved around & shared, directories get renamed, tidy project-based organisation falls apart as soon as dependencies overlap. There’s no simple way for git to “checkout all assets for this specific project, plus some common global assets, plus extra assets from a different project that we're re-using.”

## The Solution?

`git-sparta` utilises `.gitattributes` files that define 'tags' (custom meta attributes) for the file patterns you choose. These `.gitattributes` can be colocated with the relevant assets in the LFS repo. The program queries for files that have a specific tag attribute, and from this list, generates the sparse-checkout pattern. As a submodule, the LFS repo is only hydrated with the objects actually needed for the dependent project.

## Quick Start

1. Tag files in your repository using git attributes:

   ```gitattributes
   assets/project-a/** projects=project-a
   assets/shared/** projects=global
   assets/project-b/** projects=project-b,global
   ```

2. Create a JSON config file describing your submodule:

   ```json
   {
     "SUBMODULE_NAME": "assets",
     "SUBMODULE_PATH": "assets/shared",
     "SUBMODULE_URL": "https://github.com/example/assets.git",
     "SUBMODULE_BRANCH": "main",
     "PROJECT_TAG": "project-a"
   }
   ```

3. Set up the sparse submodule:

   ```bash
   git sparta setup-submodule
   ```

The tool will generate sparse-checkout patterns, initialize the submodule, and checkout only files tagged with your project identifier.

## Commands

### `generate-sparse-list`

Generate or interactively select sparse-checkout patterns for a project tag.

```bash
# Interactive: shows picker to select from available tags
git sparta generate-sparse-list --repo /path/to/repo

# Direct: generate patterns for a specific tag
git sparta generate-sparse-list my-project --repo /path/to/repo -y
```

**Options:**
- `[TAG]` — Project tag to filter (optional; shows picker if omitted)
- `--repo <PATH>` — Repository to analyze (default: current dir)
- `--attribute <NAME>` — Attribute name to scan (default: `projects`)
- `-y, --yes` — Skip interactive prompts

### `setup-submodule`

Set up a sparse submodule checkout based on JSON configuration.

```bash
git sparta setup-submodule [--config-dir <PATH>] [-y]
```

**Options:**
- `--config-dir <PATH>` — Directory containing configuration JSON (default: current dir)
- `-y, --yes` — Auto-confirm all prompts

### `teardown-submodule`

Remove a previously configured sparse submodule.

```bash
git sparta teardown-submodule [--config-dir <PATH>] [-y]
```

## Configuration

Create a JSON file (e.g., `sparta.json`) with:

```json
{
  "SUBMODULE_NAME": "globdeps",
  "SUBMODULE_PATH": "assets/dev/globdeps",
  "SUBMODULE_URL": "https://github.com/example/globdeps.git",
  "SUBMODULE_BRANCH": "main",
  "PROJECT_TAG": "PROJ1",
  "SHARED_MIRROR_PATH": "/path/to/local/mirror"
}
```

| Field | Description |
|-------|-------------|
| `SUBMODULE_NAME` | Name for the submodule |
| `SUBMODULE_PATH` | Relative path where the submodule will be checked out |
| `SUBMODULE_URL` | Git URL of the submodule repository |
| `SUBMODULE_BRANCH` | Branch to track |
| `PROJECT_TAG` | Tag to filter files |
| `SHARED_MIRROR_PATH` | (Optional) Path to local mirror for git alternates |

### Local Overrides

Create a `*.local.json` file to override values locally (not committed):

```json
{
  "SUBMODULE_URL": "file:///local/path/to/repo",
  "SHARED_MIRROR_PATH": "/path/to/local/mirror"
}
```

Environment variables `SUBMODULE_URL` and `SHARED_MIRROR_PATH` also work as overrides.

## Git Attributes Syntax

Tag files using any attribute name (default: `projects`) in `.gitattributes`:

```gitattributes
# Single project
path/to/file.txt projects=project-a

# Multiple projects (comma-separated)
path/to/shared/** projects=project-a,project-b

# Global (always included when any tag matches)
common/** projects=global

# Pattern matching
assets/PROJ1-*/** projects=PROJ1
```

Files with `projects=global` are always included. Tags use substring matching, so `PROJECT_TAG: "PROJ1"` matches both `PROJ1` and `PROJ1-extra`.

## Related Projects

- [git-sparse-checkout](https://git-scm.com/docs/git-sparse-checkout) — Built-in git sparse checkout
- [gitoxide](https://github.com/Byron/gitoxide) — Pure Rust implementation of git (used by git-sparta)
