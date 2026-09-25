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
cargo clippy -p loco-ui-caps --all-targets -- -D warnings
cargo clippy -p loco-ui --all-targets -- -D warnings
cargo clippy -p loco-ui --features http --all-targets -- -D warnings
cargo clippy -p loco-ui --features loco --all-targets -- -D warnings

echo "== tests (unit, doc, only-one-script, Blitz layout + screenshots)"
cargo test --workspace
# `examples/loco-app` turns `loco` on for the workspace run; these check the crate on its own.
cargo test -p loco-ui --features loco loco
cargo test -p loco-ui --features loco --doc loco
# `cargo lui install` on a fresh `loco new` copy, then `cargo check` of the result.
cargo test -p loco-ui --test install -- --include-ignored

echo "== rustdoc (deny warnings, all features)"
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p loco-ui-caps -p loco-ui-macros -p loco-ui --all-features

echo "== no <script> outside enhance.rs"
# The enhancement tag is built in loco-ui/src/enhance.rs; nothing else may write one.
if grep -rn '<script' loco-ui/src demo/src loco-ui-test/src examples/loco-app/src \
     | grep -v '^loco-ui/src/enhance.rs:' \
     | grep -v 'matches("<script")' \
     | grep -v '^\S*:\s*//' \
     | grep -v 'code { "<script>" }'; then
  echo "found a <script> mention outside enhance.rs and its test"; exit 1
fi

echo "== enhancement script in headless Firefox"
if command -v geckodriver >/dev/null && command -v node >/dev/null; then
  # axe-core for the accessibility pass (test-only, never served).
  [ -d scripts/node_modules/axe-core ] || npm install --prefix scripts --no-audit --no-fund
  node scripts/browser-check.mjs
else
  echo "skipped: geckodriver or node not installed"
fi

echo "== screenshots"
ls tests/shots/*.png | wc -l | xargs -I{} echo "{} PNGs under tests/shots/"
echo "OK"
