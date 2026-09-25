#!/usr/bin/env bash
# Side-by-side look check (M20): Firefox screenshots of every component page, light and dark,
# 1280 and 420 wide, plus the shadcn/ui docs page for the same component, into target/look/.
# Needs a built demo (`cargo build -p demo`) and Firefox. Nothing here is a test: compare by eye.
# Usage: scripts/look.sh [--no-shadcn]
set -eu
cd "$(dirname "$0")/.."
out="$PWD/target/look"
mkdir -p "$out/light" "$out/dark"
echo 'user_pref("layout.css.prefers-color-scheme.content-override", 1);' > "$out/light/user.js"
printf 'user_pref("layout.css.prefers-color-scheme.content-override", 0);\nuser_pref("ui.systemUsesDarkTheme", 1);\n' > "$out/dark/user.js"

# demo path | shadcn docs component (empty: no counterpart)
pages="
dialog|/dialog?dialog=confirm|dialog
popover|/popover|dropdown-menu
tabs|/tabs?tab.demo=1|tabs
accordion|/accordion?open.faq=0,2|accordion
combobox|/combobox?q=r&sel=Zig|combobox
list|/list?page=2|pagination
form|/form|input
counter|/counter|button
inputs|/inputs|select
table|/table?q=a|data-table
wizard|/wizard?step.signup=1|
toast|/toast|sonner
nav|/nav|sheet
dashboard|/dashboard|card
palette|/palette?q=ta|command
settings|/settings|switch
stream|/stream|skeleton
chart|/chart|chart
blocks-shell|/blocks/shell|sidebar
blocks-auth|/blocks/auth|
blocks-record|/blocks/record|
blocks-error|/no-such-page|
"

PORT=3009 target/debug/demo >/dev/null 2>&1 &
server=$!
trap 'kill $server' EXIT
for _ in $(seq 20); do curl -s -o /dev/null http://127.0.0.1:3009/ && break; sleep 0.25; done

shot() { timeout 90 firefox --headless --profile "$out/$1" --window-size="$2" --screenshot "$out/$3.png" "$4" >/dev/null 2>&1 || true; }
# First visit teaches the server the browser's capabilities; screenshots come from the second.
shot light 1280,900 warm-light http://127.0.0.1:3009/
shot dark 1280,900 warm-dark http://127.0.0.1:3009/
echo "$pages" | while IFS='|' read -r name path ref; do
  [ -n "$name" ] || continue
  for scheme in light dark; do
    shot "$scheme" 1280,1100 "$name-$scheme-1280" "http://127.0.0.1:3009$path"
    shot "$scheme" 420,1100 "$name-$scheme-420" "http://127.0.0.1:3009$path"
  done
  if [ -n "$ref" ] && [ "${1:-}" != "--no-shadcn" ]; then
    shot light 1280,1100 "$name-shadcn" "https://ui.shadcn.com/docs/components/$ref"
  fi
  echo "shot $name"
done
ls "$out"/*.png | wc -l | xargs -I{} echo "{} PNGs under target/look/"
