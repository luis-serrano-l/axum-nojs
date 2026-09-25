#!/usr/bin/env sh
# Static snapshot of the demo for GitHub Pages: every GET page in PATHS rendered to HTML, the
# enhancement script taken out, links made relative, and a banner on each page saying that
# forms, cookies and paging need the real server. Writes target/site/ (or the directory given).
# Usage: scripts/snapshot.sh [dir]
set -eu
cd "$(dirname "$0")/.."
cargo run --quiet -p demo -- snapshot "${1:-target/site}"
