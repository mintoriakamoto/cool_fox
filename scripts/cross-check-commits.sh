#!/usr/bin/env bash
# Type-check commits for x86_64-pc-windows-msvc from Linux (no Windows SDK needed).
# usage: scripts/cross-check-commits.sh [git-rev-range, default: -30 of HEAD] ; env OUT=report.tsv
# blake3 is switched to its pure-Rust code in a throwaway worktree only (its MSVC
# build needs ml64.exe); the repository's Cargo.toml is not touched.
set -u
repo=$(git rev-parse --show-toplevel)
range=${1:-"-n 30 HEAD"}
out=${OUT:-$repo/cross-check-report.txt}
wt=$(mktemp -d)/wt
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$repo/target/cross-check}
git -C "$repo" worktree add --detach "$wt" HEAD >/dev/null
printf 'commit\tresult\tseconds\tsubject\n' > "$out"
for c in $(git -C "$repo" rev-list --reverse $range); do
  git -C "$wt" checkout -q --detach "$c" -- . 2>/dev/null || git -C "$wt" checkout -q --detach "$c"
  sed -i 's|^blake3 = "\([0-9.]*\)"|blake3 = { version = "\1", features = ["pure"] }|' "$wt/Cargo.toml"
  s=$(date +%s)
  if (cd "$wt" && cargo check --workspace --target x86_64-pc-windows-msvc --locked --message-format short >"$wt/../log-$c" 2>&1); then r=ok; else r=FAIL; fi
  printf '%s\t%s\t%s\t%s\n' "${c:0:8}" "$r" "$(( $(date +%s)-s ))" "$(git -C "$repo" log -1 --format=%s "$c")" | tee -a "$out"
  git -C "$wt" checkout -q -- Cargo.toml
done
git -C "$repo" worktree remove --force "$wt"
