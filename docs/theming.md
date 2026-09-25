# Theming

Every colour, corner and gap in `loco-ui` is a `--lui-*` custom property. The components never
name a colour of their own (a test in `loco-ui/src/lib.rs` fails if one does), so a theme is
two colour scales and a handful of values, not a stylesheet. `layout::Tokens` holds them; `ui.page(..).tokens(&t)` emits them once
per page.

To pick them by eye, the demo's theme builder (`/theme`, `cargo run -p demo`) asks for two
colours, a brand and a gray (a picker, or a swatch of Radix Colors' step 9), and a radius. Each
colour becomes step 9 of a 12-step scale for light and dark (`layout::Scale::derive`, shaped
like `Scale::INDIGO` and `Scale::SLATE`), so the roles that alias the scales follow. Text on
the brand fill turns near-black when white would fall under 4.5:1. The builder previews a
card, a form, buttons, a badge, an alert, the four shadows and the primary gradient in both
schemes, downloads the result as `theme.css`, and shows the two scales as `Scale` consts to
paste into a `Tokens`. Paste the file after the stylesheet (`page.css(include_str!("theme.css"))`).
It needs no script: the form sends the colours in the URL, so a theme can be shared as a link.

## The tokens

Colour starts from two 12-step scales after Radix Colors, `Tokens::gray` and
`Tokens::brand` (`layout::Scale`), written in oklch and emitted as hex (Chrome 109 has no
`oklch()`). Each step has one job: 1–2 backgrounds, 3–5 component surfaces (normal, hover,
pressed), 6–8 borders and the focus ring, 9–10 solid fills, 11–12 text. The default is a
Linear-like theme: Radix slate for the grays (`Scale::SLATE`) and indigo for the brand
(`Scale::INDIGO`). Swap a scale and every role built on it follows:
`Tokens { brand: my_scale, ..Default::default() }`.

The roles are shadcn/ui's, under `--lui-*` names, and by default each is an alias onto a step
(`--lui-bg: var(--lui-gray-1)`). The status colours are Radix Colors step 11, the step made for
text (light green and amber a shade darker). A test in `layout.rs` checks every text role on
every surface in both schemes clears 4.5:1.

| Token | Default light / dark | What it affects |
|---|---|---|
| `--lui-gray-1` … `-12` | slate | The neutral scale the roles below alias. |
| `--lui-brand-1` … `-12` | indigo | The brand scale: `primary` is step 9, `ring` step 8, `link` step 11. |
| `--lui-bg` | gray 1: `#fcfcfd` / `#111113` | The page background. The sticky table header and the popover fallback paint it too, so they cover rows that scroll under them. |
| `--lui-fg` | gray 12: `#1c2024` / `#edeef0` | Body text, headings, tab titles, sorted column header, the dialog and flash text, the wordmark. |
| `--lui-muted` | gray 11: `#60646c` / `#b0b4ba` | Secondary text: notes, table headers, the wizard's step list and legends, streamed placeholders, the "built on" line, the tagline. |
| `--lui-line` | gray 6: `#d9d9e0` / `#363a3f` | Every 1px rule: dialogs, popovers, cards, accordion and wizard fieldsets, table rules, the pager list. |
| `--lui-surface` | gray 2: `#f9f9fb` / `#18191b` | Quiet raised areas: `<code>`, the demo stage, open accordion panels, streamed slots. |
| `--lui-card` | gray 1 / gray 2 | Cards and stat tiles. |
| `--lui-popover` | gray 1 / gray 2 | Floating layers: dialogs, drawers, popovers, menus, the command palette, toasts. |
| `--lui-secondary` | gray 3: `#f0f0f3` / `#212225` | Secondary buttons, the tab list, chips and badges, skeleton blocks. |
| `--lui-accent` | gray 3 / gray 4 (`#272a2d`) | The hover and highlighted surface: menu items, ghost buttons, table rows, the active palette entry. |
| `--lui-on-accent` | gray 12 | Text on the accent surface. |
| `--lui-primary` | brand 9: `#3e63dd` | Primary buttons, the current page, the current wizard step, the range slider, checked controls. |
| `--lui-on-primary` | `#ffffff` | Text on the primary colour. |
| `--lui-link` | brand 11: `#3a5bc7` / `#9eb1ff` | Link text: the brand's text step, since the solid step 9 is too dark to read on the dark background. |
| `--lui-input` | gray 7: `#cdced6` / `#43484e` | Borders of inputs, selects, textareas, checkboxes and outline buttons. |
| `--lui-ring` | brand 8: `#8da4ef` / `#435db1` | The focus ring: 3px at 50% opacity, plus the focused control's border at full strength. |
| `--lui-danger` | `#ce2c31` / `#ff9592` | Form validation messages and `aria-invalid` / `:user-invalid` rings, danger buttons, the "no" cells on `/caps`, danger flashes. |
| `--lui-on-danger` | gray 1 | Text on danger buttons and badges. |
| `--lui-ok` | `#1f7d53` / `#3dd68c` | The "yes" cells on `/caps`, ok flashes; free for your own success states. |
| `--lui-warn` | `#9c5b00` / `#ffca16` | Warning flashes. |
| `--lui-radius` | `0.5rem` | Corners of cards, dialogs and popovers. |
| `--lui-radius-sm` | radius − 2px | Not a `Tokens` field, derived: corners of buttons, inputs, chips, `<code>`. |
| `--lui-radius-lg` | radius + 4px | Not a `Tokens` field, derived: corners of cards, sheets and large panels. |
| `--lui-shadow-xs` … `-lg` | stacked | Not `Tokens` fields, per scheme (`layout::DEPTH_LIGHT`, `DEPTH_DARK`): a tight contact shadow over soft ambient ones. `xs` under controls, `sm` under cards, `md` under popovers and menus, `lg` under dialogs, sheets and toasts. Dark ones are deeper, since a soft shadow barely shows on a near-black page. |
| `--lui-highlight` | inset top edge | Not a `Tokens` field: `inset 0 1px 0` white at 60% (light) or 7% (dark), added to a raised surface's `box-shadow` so it reads as lit from above. |
| `--lui-gradient-primary` | brand 9 → 11 / 9 → 8 | Not a `Tokens` field: an oklch gradient for primary fills. Use it as `background-image` over `background-color: var(--lui-primary)`, so Chrome before 111 (no `in oklch`) keeps the flat fill. White text clears 4.5:1 over both stops (tested). |
| `--lui-gradient-ring` | brand 11 → 8 | Not a `Tokens` field: the gradient of focus rings and gradient borders. |
| `--lui-shimmer` | white 45% / 30% | Not a `Tokens` field, per scheme (`layout::EFFECTS_LIGHT`, `EFFECTS_DARK`): the light that `.shimmer()` sweeps across a filled button or badge (primary, danger). |
| `--lui-shimmer-surface` | brand 9 at 18% / white 10% | Not a `Tokens` field: the `.shimmer()` sweep over an outline, ghost, secondary or tinted button or badge, where white would not show. |
| `--lui-glow` | brand 9 at 22% / 40% | Not a `Tokens` field: the spotlight of a card's `.glow()`, a radial light at the top centre that brightens and grows on hover. |
| `--lui-beam` | brand 9 / brand 11 | Not a `Tokens` field: the lit arc of a card's `.beam()` running round its border. |
| `--lui-shimmer-duration`, `--lui-beam-duration`, `--lui-marquee-duration` | `2.5s`, `6s`, `40s` | Not `Tokens` fields, one value for both schemes: one sweep, one lap of the beam, one loop of `ui.marquee` (a marquee's `.duration(..)` sets its own). |
| `--lui-space` | `8px` | The unit every gap, margin and padding is a multiple of (`calc(var(--lui-space) * 3)`). |
| `--lui-space-1` … `-8` | 4px steps | Not `Tokens` fields, derived from `--lui-space`: steps 1, 2, 3, 4, 6 and 8 are that many halves of it (4, 8, 12, 16, 24, 32px by default). The gaps of `ui.stack`, `ui.cluster`, `ui.grid` and `ui.split` (`.gap(n)`). |
| `--lui-busy` | `0.6` | Not a `Tokens` field: the opacity of a swap root or form while the enhancement script has a request in flight (`[data-lui-busy]`). Set it to `1` on `:root` or on one root to turn the fade off. |

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
use loco_ui::prelude::*;
use loco_ui::layout::{Palette, Scale, Tokens};

const LINEN: Tokens = Tokens {
    gray: Scale::SLATE,
    brand: Scale::INDIGO,
    light: Palette {
        bg: "#f4efe6", fg: "#1d1a17", muted: "#5d574f", line: "#d6cdbf", surface: "#fffdf9",
        card: "#fffdf9", popover: "#fffdf9", secondary: "#ebe3d6", accent: "#ebe3d6",
        on_accent: "#1d1a17", primary: "#8a3b12", on_primary: "#ffffff", input: "#d6cdbf",
        ring: "#b5764f", link: "#8a3b12", danger: "#a0261c", on_danger: "#ffffff",
        ok: "#2f6b3a", warn: "#7a5500",
    },
    dark: Palette {
        bg: "#161311", fg: "#ece6dc", muted: "#a59c90", line: "#3a332c", surface: "#1f1b18",
        card: "#1f1b18", popover: "#1f1b18", secondary: "#2b2521", accent: "#2b2521",
        on_accent: "#ece6dc", primary: "#e8965a", on_primary: "#1a0f06", input: "#4a4038",
        ring: "#a8683a", link: "#e8965a", danger: "#ff8f85", on_danger: "#1a0f06",
        ok: "#8fd39a", warn: "#f0c060",
    },
    radius: "3px",
    space: "8px",
};

let page = Ui::default().page("Hello", html! { p { "Warm." } }).tokens(&LINEN);
```

Its ratios: `fg`/`bg` 15.1 and 14.9, `muted`/`bg` 6.2 and 6.8, `primary`/`bg` 6.8 and 7.9,
`on-primary`/`primary` 7.7 and 8.0, `ok`/`bg` 5.6 and 10.5. Every text pair clears 4.5.

To change one value, spread the default: `Tokens { radius: "0px", ..Default::default() }` (keep the unit: `--lui-radius-sm` subtracts 2px from it) or
`Palette { primary: "#7a3b1e", ..Tokens::default().light }`.

`.tokens(..)` puts a second `<style class="lui-tokens">` right after the stylesheet with the
same three rule blocks the default palette uses, so it wins by source order and nothing else
changes. The demo shows the pair: `/` is the default, `/?palette=linen` is this one.

## Beyond the tokens

Fonts are the system stack in two custom properties on `:root`, `--lui-font-sans` and
`--lui-font-mono`; set either to change every component. The type scale and the page width are
base rules in `layout.rs`, not tokens; the look of buttons and native form controls lives in
one place each, `button.rs` and `input.rs`, and every component draws its buttons and fields
through them, so one override there reaches every dialog, table and form. To change
them, put your own `<style>` after `layout`'s (or use your own shell and call
`loco_ui::stylesheet()` for the component CSS). A component's parts are addressable by class,
`lui-<component>` on the root and `lui-<component>-<part>` inside, so overriding a single part is
one selector.
