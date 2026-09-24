# Theming

Every colour, corner and gap in `axum-nojs` is a `--nojs-*` custom property. The components never
name a colour of their own (a test in `axum-nojs/src/lib.rs` fails if one does), so a theme is
nineteen values, not a stylesheet. `layout::Tokens` holds them; `ui.page(..).tokens(&t)` emits them once
per page.

## The tokens

The roles are shadcn/ui's, under `--nojs-*` names. The default is shadcn's neutral (zinc)
theme; the status colours are Radix Colors step 11, the step made for text.

| Token | Default light / dark | What it affects |
|---|---|---|
| `--nojs-bg` | `#ffffff` / `#09090b` | The page background. The sticky table header and the popover fallback paint it too, so they cover rows that scroll under them. |
| `--nojs-fg` | `#09090b` / `#fafafa` | Body text, headings, tab titles, sorted column header, the dialog and flash text, the wordmark. |
| `--nojs-muted` | `#71717a` / `#a1a1aa` | Secondary text: notes, table headers, the wizard's step list and legends, streamed placeholders, the "built on" line, the tagline. |
| `--nojs-line` | `#e4e4e7` / `#27272a` | Every 1px rule: dialogs, popovers, cards, accordion and wizard fieldsets, table rules, the pager list. |
| `--nojs-surface` | `#fafafa` / `#18181b` | Quiet raised areas: `<code>`, the demo stage, open accordion panels, streamed slots. |
| `--nojs-card` | `#ffffff` / `#18181b` | Cards and stat tiles. |
| `--nojs-popover` | `#ffffff` / `#18181b` | Floating layers: dialogs, drawers, popovers, menus, the command palette, toasts. |
| `--nojs-secondary` | `#f4f4f5` / `#27272a` | Secondary buttons, the tab list, chips and badges, skeleton blocks. |
| `--nojs-accent` | `#f4f4f5` / `#27272a` | The hover and highlighted surface: menu items, ghost buttons, table rows, the active palette entry. |
| `--nojs-on-accent` | `#18181b` / `#fafafa` | Text on the accent surface. |
| `--nojs-primary` | `#18181b` / `#e4e4e7` | Links, primary buttons, the current page, the current wizard step, the range slider. |
| `--nojs-on-primary` | `#fafafa` / `#18181b` | Text on the primary colour and on danger buttons. |
| `--nojs-input` | `#e4e4e7` / `#3f3f46` | Borders of inputs, selects, textareas, checkboxes and outline buttons. |
| `--nojs-ring` | `#a1a1aa` / `#71717a` | The focus ring: 3px at 50% opacity, plus the focused control's border at full strength. |
| `--nojs-danger` | `#ce2c31` / `#ff9592` | Form validation messages and `aria-invalid` / `:user-invalid` rings, danger buttons, the "no" cells on `/caps`, danger flashes. |
| `--nojs-ok` | `#218358` / `#3dd68c` | The "yes" cells on `/caps`, ok flashes; free for your own success states. |
| `--nojs-warn` | `#ab6400` / `#ffca16` | Warning flashes. |
| `--nojs-radius` | `0.5rem` | Corners of cards, dialogs and popovers. |
| `--nojs-radius-sm` | radius − 2px | Not a `Tokens` field, derived: corners of buttons, inputs, chips, `<code>`. |
| `--nojs-radius-lg` | radius + 4px | Not a `Tokens` field, derived: corners of cards, sheets and large panels. |
| `--nojs-shadow-xs`, `--nojs-shadow-lg` | shadcn's | Not `Tokens` fields: the shadow under controls and under floating layers (dialogs, popovers, menus, toasts). Same in both schemes. |
| `--nojs-space` | `8px` | The unit every gap, margin and padding is a multiple of (`calc(var(--nojs-space) * 3)`). |
| `--nojs-space-1` … `-8` | 4px steps | Not `Tokens` fields, derived from `--nojs-space`: steps 1, 2, 3, 4, 6 and 8 are that many halves of it (4, 8, 12, 16, 24, 32px by default). The gaps of `ui.stack`, `ui.cluster`, `ui.grid` and `ui.split` (`.gap(n)`). |
| `--nojs-busy` | `0.6` | Not a `Tokens` field: the opacity of a swap root or form while the enhancement script has a request in flight (`[data-nojs-busy]`). Set it to `1` on `:root` or on one root to turn the fade off. |

The dark palette applies under `prefers-color-scheme: dark` unless `<html data-theme="light">`,
and always under `data-theme="dark"`. `theme_toggle` sets that attribute through a cookie, so a
theme is server state like everything else; the components never know which palette is live.

## Contrast requirements

Pairs that carry text must reach WCAG AA (4.5:1 for body text, 3:1 for large text and for
focus indicators). The pairs to check, with the default palette's ratios:

| Pair | Where it shows | Light | Dark | Needs |
|---|---|---|---|---|
| `fg` on `bg` | body text | 19.9 | 19.1 | 4.5 |
| `fg` on `popover` | text in dialogs, popovers, menus | 19.9 | 17.0 | 4.5 |
| `muted` on `bg` | notes, table headers | 4.8 | 7.8 | 4.5 |
| `muted` on `secondary` | inactive tabs in the tab list | 4.4 | 5.8 | 4.5 (3 at 18px+) |
| `primary` on `bg` | links | 17.7 | 15.7 | 4.5 |
| `on-primary` on `primary` | primary buttons, current page | 17.0 | 14.0 | 4.5 |
| `on-accent` on `accent` | hovered menu items and rows | 16.1 | 14.3 | 4.5 |
| `danger` on `bg` | validation messages | 5.2 | 9.4 | 4.5 |
| `on-primary` on `danger` | danger buttons | 5.0 | 8.4 | 4.5 |
| `ok` on `bg` | success text | 4.7 | 10.6 | 4.5 |
| `warn` on `bg` | warning text | 4.6 | 13.0 | 4.5 |
| `ring` on `bg` | focus ring | 2.6 | 4.1 | 3 |
| `line`, `input` on `bg` | borders | 1.3 | 1.3–1.9 | none: borders are not the only cue |

Two pairs sit under the line on purpose, both copied from shadcn: inactive tab labels in the
light scheme (4.4, half a point short; the active tab is `fg` on `bg`), and the light ring
(2.6). The ring is never the only focus cue: the focused control's border turns `ring` too,
and buttons, which have no border change, get the ring on top of the pressed surface. If you
need strict AA for focus, set `ring` to `muted` (`#71717a`, 4.8).

`muted` is the one most palettes get wrong: a grey that reads fine on white drops under 4.5
on a tinted background. `on-primary` is the second: a mid-tone primary has no colour that
contrasts with it on both sides, so pick the primary dark (light scheme) or light (dark
scheme), never in the middle. The ratios above come from the standard relative-luminance
formula; any contrast checker gives the same numbers.

## A different palette

`Tokens` is a plain struct with `Default`, so a theme is a value. This one is "linen and
copper": warm paper, near-black text, a copper primary that turns to amber in the dark scheme.

```rust
use axum_nojs::prelude::*;
use axum_nojs::layout::{Palette, Tokens};

const LINEN: Tokens = Tokens {
    light: Palette {
        bg: "#f4efe6", fg: "#1d1a17", muted: "#5d574f", line: "#d6cdbf", surface: "#fffdf9",
        card: "#fffdf9", popover: "#fffdf9", secondary: "#ebe3d6", accent: "#ebe3d6",
        on_accent: "#1d1a17", primary: "#8a3b12", on_primary: "#ffffff", input: "#d6cdbf",
        ring: "#b5764f", danger: "#a0261c", ok: "#2f6b3a", warn: "#7a5500",
    },
    dark: Palette {
        bg: "#161311", fg: "#ece6dc", muted: "#a59c90", line: "#3a332c", surface: "#1f1b18",
        card: "#1f1b18", popover: "#1f1b18", secondary: "#2b2521", accent: "#2b2521",
        on_accent: "#ece6dc", primary: "#e8965a", on_primary: "#1a0f06", input: "#4a4038",
        ring: "#a8683a", danger: "#ff8f85", ok: "#8fd39a", warn: "#f0c060",
    },
    radius: "3px",
    space: "8px",
};

let page = Ui::default().page("Hello", html! { p { "Warm." } }).tokens(&LINEN);
```

Its ratios: `fg`/`bg` 15.1 and 14.9, `muted`/`bg` 6.2 and 6.8, `primary`/`bg` 6.8 and 7.9,
`on-primary`/`primary` 7.7 and 8.0, `ok`/`bg` 5.6 and 10.5. Every text pair clears 4.5.

To change one value, spread the default: `Tokens { radius: "0px", ..Default::default() }` (keep the unit: `--nojs-radius-sm` subtracts 2px from it) or
`Palette { primary: "#7a3b1e", ..Tokens::default().light }`.

`.tokens(..)` puts a second `<style class="nojs-tokens">` right after the stylesheet with the
same three rule blocks the default palette uses, so it wins by source order and nothing else
changes. The demo shows the pair: `/` is the default, `/?palette=linen` is this one.

## Beyond the tokens

Fonts are the system stack in two custom properties on `:root`, `--nojs-font-sans` and
`--nojs-font-mono`; set either to change every component. The type scale and the page width are
base rules in `layout.rs`, not tokens. To change
them, put your own `<style>` after `layout`'s (or use your own shell and call
`axum_nojs::stylesheet()` for the component CSS). A component's parts are addressable by class,
`nojs-<component>` on the root and `nojs-<component>-<part>` inside, so overriding a single part is
one selector.
