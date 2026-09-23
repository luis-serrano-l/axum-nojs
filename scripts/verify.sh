#!/usr/bin/env sh
# Full verification pass: build, clippy (zero warnings), tests (incl. the no-script test and
# the Blitz screenshots under tests/shots), plus a source grep for stray <script> tags.
# Usage: scripts/verify.sh
set -eu
cd "$(dirname "$0")/.."

echo "== build"
cargo build --workspace --all-targets

echo "== clippy (deny warnings)"
cargo clippy --workspace --all-targets -- -D warnings

echo "== tests (unit, doc, no-script, Blitz layout + screenshots)"
cargo test --workspace

echo "== no <script> in anything the crates emit"
# The only allowed mentions are the assertions in tests and docs that say there is none.
if grep -rn '<script' webonsive/src demo/src webonsive-test/src \
     | grep -v 'contains("<script")' \
     | grep -v '^\S*:\s*//' \
     | grep -v 'code { "<script>" }'; then
  echo "found a <script> mention outside the enforcement test"; exit 1
fi

echo "== screenshots"
ls tests/shots/*.png | wc -l | xargs -I{} echo "{} PNGs under tests/shots/"
echo "OK"
