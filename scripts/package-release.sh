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
  cd "$script_dir/.."
}

# Determines the release tag name from arguments, CI environment, or Cargo.toml.
#
# Arguments:
#   $1 - Optional tag string passed from command line
#
# Outputs:
#   Prints resolved tag string (e.g., "v0.1.0") to stdout
#
# Example usage:
#   tag=$(resolve_release_tag "$1")
resolve_release_tag() {
  local tag_arg="${1:-}"
  if [ -n "$tag_arg" ]; then
    echo "$tag_arg"
  elif [ -n "${GITHUB_REF_NAME:-}" ]; then
    echo "$GITHUB_REF_NAME"
  else
    local ver
    ver=$(grep -m1 '^version = ' Cargo.toml | sed -E 's/version = "(.*)"/\1/')
    echo "v${ver}"
  fi
}

# Packages compiled backend binaries and migrations into a tarball.
#
# Arguments:
#   $1 - Target output directory for archive
#   $2 - Release tag string (e.g., "v0.1.0")
#
# Example usage:
#   package_backend "release-artifacts" "v0.1.0"
package_backend() {
  local out_dir="$1"
  local tag="$2"
  local pkg_name="theater-seat-booking-${tag}-linux-x86_64"
  local stage_dir="target/${pkg_name}"

  for bin_name in theater-api queue_worker seed; do
    if [ ! -f "target/release/${bin_name}" ]; then
      echo "Error: target/release/${bin_name} not found. Build release binaries first." >&2
      exit 1
    fi
  done

  if [ ! -d "migrations" ]; then
    echo "Error: migrations directory not found." >&2
    exit 1
  fi

  rm -rf "$stage_dir"
  mkdir -p "$stage_dir"

  cp target/release/theater-api "$stage_dir/"
  cp target/release/queue_worker "$stage_dir/"
  cp target/release/seed "$stage_dir/"
  cp -r migrations "$stage_dir/"

  tar -czf "${out_dir}/${pkg_name}.tar.gz" -C target "$pkg_name"
  rm -rf "$stage_dir"
  echo "Packaged backend: ${out_dir}/${pkg_name}.tar.gz"
}

# Packages built frontend distribution files into a tarball.
#
# Arguments:
#   $1 - Target output directory for archive
#   $2 - Release tag string (e.g., "v0.1.0")
#
# Example usage:
#   package_frontend "release-artifacts" "v0.1.0"
package_frontend() {
  local out_dir="$1"
  local tag="$2"
  local pkg_name="theater-seat-booking-frontend-${tag}"

  if [ ! -d "frontend/dist" ]; then
    echo "Error: frontend/dist not found. Build frontend first." >&2
    exit 1
  fi

  tar -czf "${out_dir}/${pkg_name}.tar.gz" -C frontend/dist .
  echo "Packaged frontend: ${out_dir}/${pkg_name}.tar.gz"
}

# Generates SHA256 checksums file for all archives in target directory.
#
# Arguments:
#   $1 - Directory containing release archives
#
# Example usage:
#   generate_checksums "release-artifacts"
generate_checksums() {
  local out_dir="$1"
  (
    cd "$out_dir"
    sha256sum ./*.tar.gz > SHA256SUMS.txt
  )
  echo "Generated checksums in ${out_dir}/SHA256SUMS.txt"
}

main() {
  ensure_project_root

  local tag
  tag=$(resolve_release_tag "${1:-}")

  local out_dir="release-artifacts"
  rm -rf "$out_dir"
  mkdir -p "$out_dir"

  echo "Packaging release assets for tag: $tag"
  package_backend "$out_dir" "$tag"
  package_frontend "$out_dir" "$tag"
  generate_checksums "$out_dir"
  echo "Release assets packaging complete."
}

main "$@"
