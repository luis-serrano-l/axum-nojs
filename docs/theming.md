# Theming

Every colour, corner and gap in `webonsive` is a `--wo-*` custom property. The components never
name a colour of their own (a test in `webonsive/src/lib.rs` fails if one does), so a theme is
eleven values, not a stylesheet. `layout::Tokens` holds them; `layout_with` emits them once per
page.

## The tokens

| Token | Default light / dark | What it affects |
|---|---|---|
| `--wo-bg` | `#eef1ec` / `#0f1512` | The page background. The sticky table header and the popover fallback paint it too, so they cover rows that scroll under them. |
| `--wo-fg` | `#14201a` / `#e4ebe6` | Body text, headings, tab titles, sorted column header, the dialog and flash text, the wordmark. |
| `--wo-muted` | `#566158` / `#97a59c` | Secondary text: notes, table headers, the wizard's step list and legends, streamed placeholders, the "built on" line, the tagline. |
| `--wo-line` | `#c9d2cb` / `#2b3630` | Every 1px border: inputs, buttons, dialogs, popovers, accordion and wizard fieldsets, table rules, the pager list, the theme switch. |
| `--wo-surface` | `#ffffff` / `#171f1b` | Raised things: inputs, buttons, `<code>`, dialogs, popovers, open accordion panels, streamed slots, the flash banner. |
| `--wo-accent` | `#1f6f5f` / `#62c9a8` | Links, primary buttons, the active tab's underline, the current page, the current wizard step, the range slider, the focus ring, hover borders. |
| `--wo-on-accent` | `#ffffff` / `#08110d` | Text on the accent: primary buttons, current page link, current wizard step, the pager's "Load more". |
| `--wo-danger` | `#b3261e` / `#ff8a80` | Form validation messages and `:user-invalid` borders, the "no" cells on `/caps`. |
| `--wo-ok` | `#2f7a3a` / `#7bd389` | The "yes" cells on `/caps`; free for your own success states. |
| `--wo-radius` | `6px` | Corners of buttons, inputs, dialogs, popovers, chips, `<code>`, the colour swatch. |
| `--wo-space` | `8px` | The unit every gap, margin and padding is a multiple of (`calc(var(--wo-space) * 3)`). |

The dark palette applies under `prefers-color-scheme: dark` unless `<html data-theme="light">`,
and always under `data-theme="dark"`. `theme_toggle` sets that attribute through a cookie, so a
theme is server state like everything else; the components never know which palette is live.

## Contrast requirements

Pairs that carry text must reach WCAG AA (4.5:1 for body text, 3:1 for large text and for
focus indicators). The pairs to check, with the default palette's ratios:

| Pair | Where it shows | Light | Dark | Needs |
|---|---|---|---|---|
| `fg` on `bg` | body text | 14.7 | 15.2 | 4.5 |
| `fg` on `surface` | text in inputs, dialogs, popovers | 16.8 | 13.9 | 4.5 |
| `muted` on `bg` | notes, table headers | 5.7 | 7.2 | 4.5 |
| `accent` on `bg` | links, focus ring | 5.3 | 9.2 | 4.5 (3 for the ring) |
| `accent` on `surface` | links inside dialogs and panels | 6.0 | 8.4 | 4.5 |
| `on-accent` on `accent` | primary buttons | 6.0 | 9.5 | 4.5 |
| `danger` on `bg` | validation messages | 5.7 | 8.1 | 4.5 |
| `ok` on `bg` | success text | 4.6 | 10.1 | 4.5 |
| `line` on `bg` | borders | 1.4 | 1.5 | none: borders are not the only cue |

`muted` is the one most palettes get wrong: a grey that reads fine on white drops under 4.5
on a tinted background. `on-accent` is the second: a mid-tone accent has no colour that
contrasts with it on both sides, so pick the accent dark (light scheme) or light (dark
scheme), never in the middle. The ratios above come from the standard relative-luminance
formula; any contrast checker gives the same numbers.

## A different palette

`Tokens` is a plain struct with `Default`, so a theme is a value. This one is "linen and
copper": warm paper, near-black ink, a copper accent that turns to amber in the dark scheme.

```rust
use maud::html;
use webonsive::{Caps, Theme, layout::{Palette, Tokens, layout_with}};

const LINEN: Tokens = Tokens {
    light: Palette {
        bg: "#f4efe6", fg: "#1d1a17", muted: "#5d574f", line: "#d6cdbf", surface: "#fffdf9",
        accent: "#8a3b12", on_accent: "#ffffff", danger: "#a0261c", ok: "#2f6b3a",
    },
    dark: Palette {
        bg: "#161311", fg: "#ece6dc", muted: "#a59c90", line: "#3a332c", surface: "#1f1b18",
        accent: "#e8965a", on_accent: "#1a0f06", danger: "#ff8f85", ok: "#8fd39a",
    },
    radius: "3px",
    space: "8px",
};

let page = layout_with(&Caps::all(), "Hello", Theme::Auto, &LINEN, html! { p { "Warm." } });
```

Its ratios: `fg`/`bg` 15.1 and 14.9, `muted`/`bg` 6.2 and 6.8, `accent`/`bg` 6.8 and 7.9,
`on-accent`/`accent` 7.7 and 8.0, `ok`/`bg` 5.6 and 10.5. Every text pair clears 4.5.

To change one value, spread the default: `Tokens { radius: "0", ..Default::default() }` or
`Palette { accent: "#7a3b1e", ..Tokens::default().light }`.

`layout_with` puts a second `<style class="wo-tokens">` right after the stylesheet with the
same three rule blocks the default palette uses, so it wins by source order and nothing else
changes. The demo shows the pair: `/` is the default, `/?palette=linen` is this one.

## Beyond the tokens

Fonts, the type scale and the page width are base rules in `layout.rs`, not tokens. To change
them, put your own `<style>` after `layout`'s (or use your own shell and call
`webonsive::stylesheet()` for the component CSS). A component's parts are addressable by class,
`wo-<component>` on the root and `wo-<component>-<part>` inside, so overriding a single part is
one selector.
