#!/usr/bin/env bash
# Latency baseline for the demo: starts the release build on port 3001 (never 3000, which may
# be the server you are looking at) and prints p50/p95 time to first byte and to the full
# response for a few routes, cold (a new connection per request) and warm (one kept-alive
# connection). Then headless Firefox reports navigation timing for the same routes.
# Usage: scripts/bench.sh [samples]   (needs curl; the Firefox part needs node and geckodriver)
set -euo pipefail
export LC_ALL=C
cd "$(dirname "$0")/.."
N=${1:-50}
ROUTES=(/ /table /stream)
BASE=http://127.0.0.1:3001

cargo build -q --release -p demo
PORT=3001 target/release/demo & SERVER=$!
trap 'kill $SERVER 2>/dev/null' EXIT
for _ in $(seq 50); do curl -s -o /dev/null "$BASE/" && break; sleep 0.1; done

# Reads "ttfb total" lines (seconds) and prints p50/p95 of each in milliseconds.
stats() {
  local tmp; tmp=$(cat)
  pick() { cut -d' ' -f"$1" <<<"$tmp" | sort -g | awk -v p="$2" '{ v[NR]=$1 } END { i=int(NR*p+0.5); if (i<1) i=1; printf "%7.2f", v[i]*1000 }'; }
  echo "ttfb p50 $(pick 1 0.5)  p95 $(pick 1 0.95)   total p50 $(pick 2 0.5)  p95 $(pick 2 0.95) ms"
}

FMT='%{time_starttransfer} %{time_total}\n'
echo "== curl, $N samples each"
for r in "${ROUTES[@]}"; do
  printf "%-8s cold  " "$r"
  for _ in $(seq "$N"); do curl -s -o /dev/null -w "$FMT" "$BASE$r"; done | stats
  # One curl process with N URLs reuses the connection; timings are per transfer.
  printf "%-8s warm  " "$r"
  urls=(); for _ in $(seq "$N"); do urls+=(-o /dev/null "$BASE$r"); done
  curl -s -w "$FMT" "${urls[@]}" | stats
done

if command -v node >/dev/null && command -v geckodriver >/dev/null; then
  echo "== Firefox navigation timing (median of 5 loads)"
  node scripts/bench-nav.mjs "${ROUTES[@]}"
fi
