#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
release_script="$root/scripts/release.sh"
tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

assert_line() {
  local file="$1"
  local expected="$2"

  if ! grep -Fx "$expected" "$file" >/dev/null; then
    printf 'missing expected line: %s\n' "$expected" >&2
    sed -n '1,120p' "$file" >&2
    exit 1
  fi
}

assert_fails() {
  if "$@" >/dev/null 2>&1; then
    printf 'expected command to fail: %s\n' "$*" >&2
    exit 1
  fi
}

bash -n "$release_script"

version="$("$release_script" workspace-version "$root/Cargo.toml")"
"$release_script" validate-tag "v${version}" "$root/Cargo.toml" >"$tmp_dir/tag.out"
"$release_script" validate-tag "refs/tags/v${version}" "$root/Cargo.toml" >"$tmp_dir/ref.out"

assert_line "$tmp_dir/tag.out" "version=$version"
assert_line "$tmp_dir/tag.out" "tag=v$version"
assert_line "$tmp_dir/tag.out" "release_title=Bowser v$version"
assert_line "$tmp_dir/ref.out" "tag=v$version"
assert_fails "$release_script" validate-tag "bowser-v${version}" "$root/Cargo.toml"
assert_fails "$release_script" validate-tag "v999.999.999" "$root/Cargo.toml"

binary="$tmp_dir/bowser"
printf '#!/usr/bin/env sh\nexit 0\n' >"$binary"
chmod +x "$binary"

"$release_script" package "$version" test-target "$binary" "$tmp_dir/dist" >"$tmp_dir/package.out"

archive="bowser-${version}-test-target.tar.gz"
checksum="${archive}.sha256"
assert_line "$tmp_dir/package.out" "asset_name=$archive"
test -f "$tmp_dir/dist/$archive"
test -f "$tmp_dir/dist/$checksum"
tar -tzf "$tmp_dir/dist/$archive" | grep -Fx "bowser-${version}-test-target/bowser" >/dev/null

if command -v sha256sum >/dev/null 2>&1; then
  (
    cd "$tmp_dir/dist"
    sha256sum -c "$checksum"
  )
else
  (
    cd "$tmp_dir/dist"
    shasum -a 256 -c "$checksum"
  )
fi
