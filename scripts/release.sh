#!/usr/bin/env bash
set -euo pipefail

usage() {
  printf '%s\n' "Usage:" "  scripts/release.sh workspace-version [manifest-path]" "  scripts/release.sh validate-tag <tag> [manifest-path]" "  scripts/release.sh package <version> <target> <binary-path> <dist-dir>" >&2
}

fail() {
  printf 'error: %s\n' "$1" >&2
  exit 64
}

workspace_version() {
  local manifest_path="${1:-Cargo.toml}"

  cargo metadata --locked --no-deps --format-version 1 --manifest-path "$manifest_path" | python3 -c '
import json
import sys

metadata = json.load(sys.stdin)
versions = {
    package["version"]
    for package in metadata["packages"]
    if package["name"] in {"bowser", "bowser-cli"}
}
if len(versions) != 1:
    raise SystemExit("bowser and bowser-cli must share one workspace version")
print(versions.pop())
'
}

validate_tag() {
  if [ "$#" -lt 1 ] || [ "$#" -gt 2 ]; then
    usage
    exit 64
  fi

  local tag="${1#refs/tags/}"
  local manifest_path="${2:-Cargo.toml}"
  local version
  version="$(workspace_version "$manifest_path")"

  if [ "$tag" != "v${version}" ]; then
    fail "tag '$tag' must match workspace version 'v${version}'"
  fi

  printf 'version=%s\n' "$version"
  printf 'tag=%s\n' "$tag"
  printf 'release_title=Bowser v%s\n' "$version"
}

package_binary() {
  if [ "$#" -ne 4 ]; then
    usage
    exit 64
  fi

  local version="$1"
  local target="$2"
  local binary_path="$3"
  local dist_dir="$4"

  case "$version" in
    "" | v* | */* | *" "* | *$'\t'*)
      fail "invalid release version '$version'"
      ;;
  esac
  case "$target" in
    "" | *[!A-Za-z0-9._-]*)
      fail "invalid release target '$target'"
      ;;
  esac
  if [ ! -f "$binary_path" ]; then
    fail "binary path '$binary_path' does not exist"
  fi

  local artifact_name="bowser-${version}-${target}"
  local archive_name="${artifact_name}.tar.gz"
  local checksum_name="${archive_name}.sha256"
  local staging_dir
  staging_dir="$(mktemp -d)"
  trap 'rm -rf "$staging_dir"' EXIT

  mkdir -p "$staging_dir/$artifact_name" "$dist_dir"
  cp "$binary_path" "$staging_dir/$artifact_name/bowser"
  chmod +x "$staging_dir/$artifact_name/bowser"
  tar -C "$staging_dir" -czf "$dist_dir/$archive_name" "$artifact_name"

  if command -v sha256sum >/dev/null 2>&1; then
    (
      cd "$dist_dir"
      sha256sum "$archive_name" >"$checksum_name"
    )
  else
    (
      cd "$dist_dir"
      shasum -a 256 "$archive_name" >"$checksum_name"
    )
  fi

  printf 'asset_name=%s\n' "$archive_name"
  printf 'asset_path=%s\n' "$dist_dir/$archive_name"
  printf 'checksum_path=%s\n' "$dist_dir/$checksum_name"

  rm -rf "$staging_dir"
  trap - EXIT
}

case "${1:-}" in
  workspace-version)
    shift
    if [ "$#" -gt 1 ]; then
      usage
      exit 64
    fi
    workspace_version "${1:-Cargo.toml}"
    ;;
  validate-tag)
    shift
    validate_tag "$@"
    ;;
  package)
    shift
    package_binary "$@"
    ;;
  *)
    usage
    exit 64
    ;;
esac
