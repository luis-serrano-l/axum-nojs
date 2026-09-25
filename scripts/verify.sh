#!/usr/bin/env sh
# Full verification pass: build, clippy (zero warnings), tests (incl. the only-one-script test
# and the Blitz screenshots under tests/shots), a source grep for stray <script> tags, and,
# when geckodriver and node are installed, the headless Firefox check of the enhancement script.
# Usage: scripts/verify.sh
set -eu
cd "$(dirname "$0")/.."

echo "== format (CI runs the same check)"
cargo fmt --all --check

echo "== build"
cargo build --workspace --all-targets

echo "== clippy (deny warnings)"
cargo clippy --workspace --all-targets -- -D warnings
# The library must build and be clean at every feature level: none, http, axum.
cargo clippy -p axum-nojs-caps --all-targets -- -D warnings
cargo clippy -p axum-nojs --all-targets -- -D warnings
cargo clippy -p axum-nojs --features http --all-targets -- -D warnings
cargo clippy -p axum-nojs --features loco --all-targets -- -D warnings

echo "== tests (unit, doc, only-one-script, Blitz layout + screenshots)"
cargo test --workspace
# Nothing in the workspace turns `loco` on, so its tests and doctests run here.
cargo test -p axum-nojs --features loco loco
cargo test -p axum-nojs --features loco --doc loco

echo "== rustdoc (deny warnings, all features)"
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p axum-nojs-caps -p axum-nojs --all-features

echo "== no <script> outside enhance.rs"
# The enhancement tag is built in axum-nojs/src/enhance.rs; nothing else may write one.
if grep -rn '<script' axum-nojs/src demo/src axum-nojs-test/src \
     | grep -v '^axum-nojs/src/enhance.rs:' \
     | grep -v 'matches("<script")' \
     | grep -v '^\S*:\s*//' \
     | grep -v 'code { "<script>" }'; then
  echo "found a <script> mention outside enhance.rs and its test"; exit 1
fi

echo "== enhancement script in headless Firefox"
if command -v geckodriver >/dev/null && command -v node >/dev/null; then
  node scripts/browser-check.mjs
else
  echo "skipped: geckodriver or node not installed"
fi

echo "== screenshots"
ls tests/shots/*.png | wc -l | xargs -I{} echo "{} PNGs under tests/shots/"
echo "OK"
