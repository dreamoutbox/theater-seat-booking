#!/usr/bin/env bash
set -euo pipefail

# Switches working directory to repository root.
#
# Arguments:
#   None
#
# Example usage:
#   ensure_project_root
ensure_project_root() {
  local script_dir
  script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  cd "$script_dir"
}

# Validates that git repository state is clean for releasing.
#
# Arguments:
#   None
#
# Example usage:
#   validate_git_state
validate_git_state() {
  if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "Error: Current directory is not inside a git repository." >&2
    exit 1
  fi

  if ! git diff --cached --quiet; then
    echo "Error: Git index has staged changes. Unstage or commit them before releasing." >&2
    exit 1
  fi

  local version_files=("Cargo.toml" "crates/repository/Cargo.toml" "Cargo.lock" "frontend/package.json")
  for file in "${version_files[@]}"; do
    if [ ! -f "$file" ]; then
      echo "Error: Required version file '$file' not found." >&2
      exit 1
    fi
  done

  local dirty_version_files
  dirty_version_files=$(git status --porcelain "${version_files[@]}")
  if [ -n "$dirty_version_files" ]; then
    echo "Error: Version files have uncommitted changes. Commit or stash them before releasing:" >&2
    echo "$dirty_version_files" >&2
    exit 1
  fi
}

# Reads current package version from root Cargo.toml.
#
# Arguments:
#   None
#
# Outputs:
#   Prints semver string (e.g., "0.1.0") to stdout
#
# Example usage:
#   ver=$(read_current_version)
read_current_version() {
  local ver
  ver=$(grep -m1 '^version = ' Cargo.toml | sed -E 's/version = "(.*)"/\1/')
  if [ -z "$ver" ]; then
    echo "Error: Could not read version from Cargo.toml." >&2
    exit 1
  fi
  echo "$ver"
}

# Computes the bumped semver version string.
#
# Arguments:
#   $1 - Current version string without leading 'v' (e.g., "0.1.0")
#   $2 - Bump type: "major", "minor", or "patch"
#
# Outputs:
#   Prints the bumped semver string (e.g., "0.2.0") to stdout
#
# Example usage:
#   next_ver=$(bump_semver "0.1.0" "minor")
bump_semver() {
  local current_ver="$1"
  local bump_type="$2"

  local major minor patch
  IFS='.' read -r major minor patch <<< "$current_ver"

  if [[ -z "$major" || -z "$minor" || -z "$patch" || ! "$major" =~ ^[0-9]+$ || ! "$minor" =~ ^[0-9]+$ || ! "$patch" =~ ^[0-9]+$ ]]; then
    echo "Error: Version '$current_ver' does not follow semver format (X.Y.Z)." >&2
    exit 1
  fi

  case "$bump_type" in
    major)
      major=$((major + 1))
      minor=0
      patch=0
      ;;
    minor)
      minor=$((minor + 1))
      patch=0
      ;;
    patch)
      patch=$((patch + 1))
      ;;
    *)
      echo "Error: Invalid bump type '$bump_type'. Expected 'major', 'minor', or 'patch'." >&2
      exit 1
      ;;
  esac

  echo "${major}.${minor}.${patch}"
}

# Updates version numbers across Rust workspace and frontend package files.
#
# Arguments:
#   $1 - New semver string (e.g., "0.2.0")
#
# Example usage:
#   update_version_files "0.2.0"
update_version_files() {
  local new_ver="$1"

  echo "Updating Cargo.toml to version $new_ver"
  sed -i "0,/^version = .*/s//version = \"$new_ver\"/" Cargo.toml

  echo "Updating crates/repository/Cargo.toml to version $new_ver"
  sed -i "0,/^version = .*/s//version = \"$new_ver\"/" crates/repository/Cargo.toml

  echo "Updating frontend/package.json to version $new_ver"
  sed -i -E "s/\"version\": \"[^\"]+\"/\"version\": \"$new_ver\"/" frontend/package.json

  echo "Updating Cargo.lock via cargo check"
  cargo check --workspace --quiet
}

# Creates git commit and annotated tag for release, and pushes if requested.
#
# Arguments:
#   $1 - New semver string (e.g., "0.2.0")
#   $2 - Boolean string ("true" or "false") indicating whether to push to remote
#
# Example usage:
#   commit_and_tag "0.2.0" "false"
commit_and_tag() {
  local new_ver="$1"
  local should_push="$2"
  local tag_name="v${new_ver}"

  if git rev-parse "$tag_name" >/dev/null 2>&1; then
    echo "Error: Git tag '$tag_name' already exists." >&2
    exit 1
  fi

  git add Cargo.toml crates/repository/Cargo.toml Cargo.lock frontend/package.json
  git commit -m "chore(release): ${tag_name}"
  git tag -a "${tag_name}" -m "Release ${tag_name}"

  echo "Created commit for ${tag_name}"
  echo "Created tag ${tag_name}"

  if [ "$should_push" = "true" ]; then
    echo "Pushing commit and tag to origin..."
    git push origin HEAD "${tag_name}"
    echo "Push completed successfully."
  else
    echo "No git push performed (--push was not specified)."
    echo "To push manually run: git push origin HEAD ${tag_name}"
  fi
}

main() {
  ensure_project_root

  local bump_type=""
  local push_flag="false"

  for arg in "$@"; do
    case "$arg" in
      --push)
        push_flag="true"
        ;;
      major|minor|patch)
        if [ -n "$bump_type" ]; then
          echo "Error: Multiple bump types specified: '$bump_type' and '$arg'." >&2
          exit 1
        fi
        bump_type="$arg"
        ;;
      -h|--help)
        echo "Usage: ./release.sh <major|minor|patch> [--push]"
        exit 0
        ;;
      *)
        echo "Error: Unknown argument '$arg'." >&2
        echo "Usage: ./release.sh <major|minor|patch> [--push]" >&2
        exit 1
        ;;
    esac
  done

  if [ -z "$bump_type" ]; then
    echo "Error: Missing bump type. Expected 'major', 'minor', or 'patch'." >&2
    echo "Usage: ./release.sh <major|minor|patch> [--push]" >&2
    exit 1
  fi

  validate_git_state

  local current_ver
  current_ver=$(read_current_version)

  local new_ver
  new_ver=$(bump_semver "$current_ver" "$bump_type")

  echo "Bumping version: $current_ver -> $new_ver ($bump_type)"
  update_version_files "$new_ver"
  commit_and_tag "$new_ver" "$push_flag"
}

main "$@"
