# git-sparta - Git Sparse Tagging

`git-sparta` is a git project management tool that enables sparse checkouts of repositories and submodules based on git attributes. Instead of checking out entire submodules, you can tag files with project identifiers and only checkout the files relevant to your project.

This is useful when you have a large repo of global assets (for simplicity we'll refer to this heavy monorepo as the 'LFS repo') that are shared between and linked to multiple, separately version-controlled projects. Normally it is difficult to cleanly handle this situation—there are multiple approaches, each with their drawbacks.

For example, say we have a few small project repos, and they both contain project files that link to and depend on a large repository of binary assets, e.g. the projects could be small 3D productions, each linking to heavy model files in the shared asset repo. You could simply reference the asset in place from a shared path (a NAS mount, a single “golden” local clone, etc.). The main problem with this approach is that the project now depends on whatever happens to be in that path on a given day. Dependencies change over time and silently affect (and likely break) previous projects that were dependent on the old versions of the assets.

At the other extreme, you can vendor/copy the LFS repo into each project. This restores reproducibility, but it can explode storage and bandwidth. Keeping multiple projects in sync with fixes is tedious and error-prone; forks silently diverge; every update means moving around gigabytes of LFS data.

A middle ground is to add the LFS repo as a git submodule. This gives you a pinned commit and predictable checkouts, but you still pay for the size of the repository: git will fetch the full object history and hydrate far more than the one project needs. Large working trees slow down everything and digging around the entire LFS repo is painful when only a small slice of assets is relevant.

You could try using subtrees or manual sparse-checkouts. This works in small, stable directory layouts (e.g. a folder for each project's assets), but this is unrealistically optimistic. Pattern files drift as directories are renamed, neat and tidy project-based organisation fails as soon as assets are used in multiple projects but not all, etc. There’s no first-class way to tell git “include everything for this specific project, plus common global assets, plus a few extra random assets from a different project that we re-used here too.”

git-sparta addresses the gap: you use `.gitattributes` files to tag files/directories/patterns with project IDs (and optionally a shared `global` tag) and then query for a project by name. The tool resolves tags into sparse-checkout patterns, filters the large LFS repo against them, and only hydrates the LFS objects you actually need for the project. The result is a small, fast working tree that remains reproducible. Pair this with the submodule SHA and you get lockfile precision without forcing you to maintain brittle external dependencies or duplicate data across repos.

## Usage & Workflow

1. Tag files in your repository using git attributes:

   ```gitattributes
   assets/project-a/* projects=project-a
   assets/shared/* projects=global
   assets/project-b/* projects=project-b,global
   ```

2. Create a JSON file at the project's root describing your submodule and project tag(s):

   ```json
   {
     "SUBMODULE_NAME": "assets",
     "SUBMODULE_PATH": "assets/shared",
     "SUBMODULE_URL": "https://github.com/example/assets.git",
     "SUBMODULE_BRANCH": "main",
     "PROJECT_TAG": "project-a"
   }
   ```

> [!NOTE]
> The filename of the json doesn't matter, `git-sparta` will scan root-level json files for ones with the correct keys.

1. Set up the sparse submodule:

   ```bash
   git-sparta setup-submodule
   ```

The tool will:
- Generate sparse-checkout patterns matching your project tag
- Initialise the submodule with proper git configuration
- Only checkout files tagged with your project identifier
- Use git alternates if you have a local mirror

## Commands

### `setup-submodule`

Set up a sparse submodule checkout based on JSON configuration.

```bash
git-sparta setup-submodule [OPTIONS]

Options:
  --config-dir <PATH>  Directory containing configuration JSON (default: current dir)
  -y, --yes            Auto-confirm all prompts
```

### `generate-sparse-list`

Generate sparse-checkout patterns for a given project tag.

```bash
git-sparta generate-sparse-list <TAG> [OPTIONS]

Arguments:
  <TAG>  Project tag to filter (substring match)

Options:
  --repo <PATH>  Repository to analyze (default: current dir)
  -y, --yes      Auto-confirm prompt
```

### `teardown-submodule`

Remove a previously configured sparse submodule.

```bash
git-sparta teardown-submodule [OPTIONS]

Options:
  --config-dir <PATH>  Directory containing configuration (default: current dir)
  -y, --yes            Auto-confirm all prompts
```

## Configuration

Create a JSON file (e.g., `myproject.json`) with the following structure:

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

### Configuration Fields

- `SUBMODULE_NAME`: Name for the submodule
- `SUBMODULE_PATH`: Relative path where the submodule will be checked out
- `SUBMODULE_URL`: Git URL of the submodule repository
- `SUBMODULE_BRANCH`: Branch to track (typically `main`)
- `PROJECT_TAG`: Tag to filter files (only files with matching attributes are checked out)
- `SHARED_MIRROR_PATH` (optional): Path to a local mirror repository to use git alternates

### Local Overrides

Create a `*.local.json` file to override configuration values locally:

```json
{
  "SUBMODULE_URL": "file:///local/path/to/repo",
  "SHARED_MIRROR_PATH": "/path/to/local/mirror"
}
```

### Environment Variables

Override configuration via environment variables:
- `SUBMODULE_URL`: Override the submodule URL
- `SHARED_MIRROR_PATH`: Override the mirror path

## Git Attributes Syntax

Tag files using the `projects` attribute in `.gitattributes`:

```gitattributes
# Single project
path/to/file.txt projects=project-a

# Multiple projects (comma-separated)
path/to/shared/* projects=project-a,project-b

# Global (always included)
common/* projects=global

# Pattern matching
assets/PROJ1-*/** projects=PROJ1
```

When you run `git-sparta setup-submodule` with `PROJECT_TAG: "PROJ1"`, it will checkout:
- Files with `projects=PROJ1*` (substring match)
- Files with `projects=global` (always included)

## Related Projects & Topics

- [git-sparse-checkout](https://git-scm.com/docs/git-sparse-checkout) - Built-in git sparse checkout
- [git-subrepo](https://github.com/ingydotnet/git-subrepo) - Alternative to git submodules
- [gitoxide](https://github.com/Byron/gitoxide) - Pure Rust implementation of git
