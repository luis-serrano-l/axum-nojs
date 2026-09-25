//! # Spec
//!
//! The machine-readable component spec: for every component, the platform features it uses
//! with per-browser first-supporting versions, its fallback, and whether it needs script.
//! `spec/components.json` and the README feature matrix are generated from [`SPECS`] and
//! tests keep all three in sync, so this file is the single place to edit.
//!
//! Versions come from MDN browser-compat-data (checked September 2026). `"no"` means the
//! browser has not shipped the feature; the component then relies on its fallback there.
//!
//! ```rust
//! use loco_ui::spec::{SPECS, to_json, markdown_table};
//! assert!(SPECS.iter().any(|c| c.module == "dialog"));
//! assert!(to_json().starts_with("{\n  \"components\": ["));
//! assert!(markdown_table().starts_with("| Component |"));
//! ```

/// First version of each engine that supports a feature; `"no"` when unshipped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Baseline {
    /// First Chrome version, or `"no"`.
    pub chrome: &'static str,
    /// First Firefox version, or `"no"`.
    pub firefox: &'static str,
    /// First Safari version, or `"no"`.
    pub safari: &'static str,
}

/// One platform feature a component relies on. `name` must appear verbatim in the
/// component's `//!` doc header (a test checks it).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Feature {
    /// The feature as written in the component's doc header.
    pub name: &'static str,
    /// Where it shipped.
    pub baseline: Baseline,
}

/// Whether the component's job can be done without script.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NeedsJs {
    /// Fully scriptless.
    No,
    /// Scriptless for the stated part; the rest needs script.
    Partial(&'static str),
}

/// A component's entry in the spec.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ComponentSpec {
    /// Human name, as in the README matrix.
    pub name: &'static str,
    /// File name under `loco-ui/src/` without `.rs`.
    pub module: &'static str,
    /// Platform features it relies on, in the order the header lists them.
    pub features: &'static [Feature],
    /// What happens in a browser missing any of the features.
    pub fallback: &'static str,
    /// The verdict.
    pub needs_js: NeedsJs,
}

const fn b(chrome: &'static str, firefox: &'static str, safari: &'static str) -> Baseline {
    Baseline {
        chrome,
        firefox,
        safari,
    }
}

const fn f(name: &'static str, baseline: Baseline) -> Feature {
    Feature { name, baseline }
}

const ALWAYS: Baseline = b("1", "1", "1");
const DETAILS_NAME: Feature = f("<details name", b("120", "130", "17.2"));
const DETAILS_CONTENT: Feature = f("::details-content", b("131", "143", "18.4"));
const COOKIE: Feature = f("cookie", ALWAYS);
const PREFERS_COLOR_SCHEME: Feature = f("prefers-color-scheme", b("76", "67", "12.1"));

/// Every component, in README order.
pub const SPECS: &[ComponentSpec] = &[
    ComponentSpec {
        name: "Enhancement script",
        module: "enhance",
        features: &[
            f("fetch", b("42", "39", "10.1")),
            f("history.pushState", b("5", "4", "5")),
            f("document.startViewTransition", b("111", "144", "18")),
            f("CustomEvent", b("15", "11", "6")),
        ],
        fallback: "none needed: without the script every form and link is a normal navigation and every data-lui-* attribute is inert",
        needs_js: NeedsJs::No,
    },
    ComponentSpec {
        name: "Layout",
        module: "layout",
        features: &[
            f("@view-transition", b("126", "no", "18.2")),
            PREFERS_COLOR_SCHEME,
            f("custom properties", b("49", "31", "9.1")),
        ],
        fallback: "plain navigations (root never cross-fades); colours still switch by media query and data-theme",
        needs_js: NeedsJs::No,
    },
    ComponentSpec {
        name: "Capability beacons",
        module: "caps",
        features: &[
            f("@supports", b("28", "22", "9")),
            f("selector()", b("83", "69", "14.1")),
            f("background images", ALWAYS),
            f("cookies", ALWAYS),
        ],
        fallback: "unknown browser gets every fallback; the first view always does",
        needs_js: NeedsJs::No,
    },
    ComponentSpec {
        name: "Button",
        module: "button",
        features: &[
            f("invoker commands", b("135", "144", "26.2")),
            f("popovertarget", b("114", "125", "17")),
            f("aria-busy", ALWAYS),
            f("prefers-reduced-motion", b("74", "63", "10.1")),
        ],
        fallback: "a popover command becomes popovertarget; other commands need the component's own fallback",
        needs_js: NeedsJs::No,
    },
    ComponentSpec {
        name: "Input, checkbox, switch, radio group",
        module: "input",
        features: &[
            f("constraint validation", b("10", "4", "10.1")),
            f("type=date", b("20", "57", "14.1")),
            f(":user-invalid", b("119", "88", "16.5")),
            f("role=\"switch\"", ALWAYS),
            f("appearance: none", b("84", "80", "15.4")),
            f("<fieldset>", ALWAYS),
        ],
        fallback: "none needed: native controls; the switch stays a checkbox without appearance: none",
        needs_js: NeedsJs::Partial(
            "a live character count while typing needs the enhancement script",
        ),
    },
    ComponentSpec {
        name: "Badge",
        module: "badge",
        features: &[f("<span>", ALWAYS)],
        fallback: "none needed",
        needs_js: NeedsJs::No,
    },
    ComponentSpec {
        name: "Card",
        module: "card",
        features: &[f("grid", b("57", "52", "10.1"))],
        fallback: "none needed",
        needs_js: NeedsJs::No,
    },
    ComponentSpec {
        name: "Icon",
        module: "icon",
        features: &[f("<svg>", b("4", "3", "3.2")), f("role=\"img\"", ALWAYS)],
        fallback: "none needed",
        needs_js: NeedsJs::No,
    },
    ComponentSpec {
        name: "Avatar",
        module: "avatar",
        features: &[
            f("alt=\"\"", ALWAYS),
            f("loading=\"lazy\"", b("77", "75", "15.4")),
            f("role=\"img\"", ALWAYS),
        ],
        fallback: "the initials are the fallback",
        needs_js: NeedsJs::No,
    },
    ComponentSpec {
        name: "Stack",
        module: "stack",
        features: &[f("gap", b("84", "63", "14.1"))],
        fallback: "none needed",
        needs_js: NeedsJs::No,
    },
    ComponentSpec {
        name: "Cluster",
        module: "cluster",
        features: &[
            f("flex-wrap", b("29", "28", "9")),
            f("gap", b("84", "63", "14.1")),
        ],
        fallback: "none needed",
        needs_js: NeedsJs::No,
    },
    ComponentSpec {
        name: "Grid",
        module: "grid",
        features: &[
            f("repeat(auto-fill", b("57", "52", "10.1")),
            f("@media", ALWAYS),
        ],
        fallback: "none needed",
        needs_js: NeedsJs::No,
    },
    ComponentSpec {
        name: "Split",
        module: "split",
        features: &[
            f("flex-wrap", b("29", "28", "9")),
            f("min-inline-size", b("57", "41", "12.1")),
        ],
        fallback: "none needed",
        needs_js: NeedsJs::No,
    },
    ComponentSpec {
        name: "Calendar",
        module: "calendar",
        features: &[
            f("<table>", ALWAYS),
            f("aria-current=\"date\"", ALWAYS),
            f("role=\"radiogroup\"", ALWAYS),
            f(":has(:checked)", b("105", "121", "15.4")),
        ],
        fallback: "without :has() the picked radio's day is not filled in; it is still checked and posts",
        needs_js: NeedsJs::Partial(
            "changing month in place and arrow-key moves between days need script",
        ),
    },
    ComponentSpec {
        name: "Date picker",
        module: "date_picker",
        features: &[
            f("popover", b("114", "125", "17")),
            f("anchor-name", b("125", "147", "26")),
            f("<input type=\"date\">", b("20", "57", "14.1")),
        ],
        fallback: "without popover the calendar is laid out in the form; .native() is the browser's own control",
        needs_js: NeedsJs::Partial(
            "writing the picked day onto the button before the form is sent needs script",
        ),
    },
    ComponentSpec {
        name: "Dialog",
        module: "dialog",
        features: &[
            f("<dialog>", b("37", "98", "15.4")),
            f("command=\"show-modal\"", b("135", "144", "26.2")),
            f("<form method=\"dialog\">", b("37", "98", "15.4")),
            f("closedby", b("134", "141", "26")),
        ],
        fallback: "link to #id opens it through a :target rule, chosen server-side; the confirm footer is a plain form either way",
        needs_js: NeedsJs::No,
    },
    ComponentSpec {
        name: "Popover menu",
        module: "popover",
        features: &[
            f("popover", b("114", "125", "17")),
            f("anchor-name", b("125", "147", "26")),
        ],
        fallback: "no anchor: UA-centred popover; no popover: <details> dropdown (submenus nested); actions are plain post forms either way",
        needs_js: NeedsJs::No,
    },
    ComponentSpec {
        name: "Tabs",
        module: "tabs",
        features: &[
            DETAILS_NAME,
            f("display: contents", b("65", "37", "11.1")),
            DETAILS_CONTENT,
            f("view-transition-name", b("111", "144", "18")),
        ],
        fallback: "accordion markup, chosen server-side; the narrow-screen select has a Go button",
        needs_js: NeedsJs::No,
    },
    ComponentSpec {
        name: "Accordion",
        module: "accordion",
        features: &[
            DETAILS_NAME,
            DETAILS_CONTENT,
            f("interpolate-size", b("129", "no", "no")),
        ],
        fallback: "plain <details>: no exclusivity, no animation; expand/collapse and every toggle are links either way",
        needs_js: NeedsJs::No,
    },
    ComponentSpec {
        name: "Combobox",
        module: "combobox",
        features: &[
            f("<datalist>", b("20", "4", "12.1")),
            f("<optgroup>", b("20", "4", "12.1")),
            f("<search>", b("118", "118", "17")),
            f("aria-live", b("1", "1", "1")),
        ],
        fallback: "none needed: chips, results and the create row are links and forms",
        needs_js: NeedsJs::Partial(
            "static suggestions and per-submit results; live filtering and arrow keys into the results need script",
        ),
    },
    ComponentSpec {
        name: "Load-more list",
        module: "pager",
        features: &[
            f("view-transition-name", b("111", "144", "18")),
            f("scroll-margin", b("69", "90", "14.1")),
        ],
        fallback: "plain navigation to ?page=n#more",
        needs_js: NeedsJs::Partial("click-to-load; scroll-to-load needs script"),
    },
    ComponentSpec {
        name: "Table",
        module: "table",
        features: &[
            f("?sort=<col>&dir=asc|desc", b("1", "1", "1")),
            f("<search>", b("118", "118", "17")),
            f("aria-sort", b("1", "1", "1")),
            f("form attribute", b("10", "4", "5.1")),
            f("<details>", b("12", "49", "6")),
            f("<colgroup>", b("1", "1", "1")),
            f("tabular-nums", b("52", "34", "9.1")),
            f("position: sticky", b("56", "32", "13")),
            f("view-transition-name", b("111", "144", "18")),
            f("aria-busy", b("1", "1", "1")),
        ],
        fallback: "none needed: sorting, filtering, column choice and the bulk form are plain navigations and posts; no select-all without script",
        needs_js: NeedsJs::No,
    },
    ComponentSpec {
        name: "Paged table",
        module: "paged_table",
        features: &[
            f("?page=n", b("1", "1", "1")),
            f("<select>", b("1", "1", "1")),
            f("<input type=\"number\">", b("6", "29", "5.1")),
            f("<output>", b("10", "4", "7")),
        ],
        fallback: "none needed: every control is a link or a form",
        needs_js: NeedsJs::No,
    },
    ComponentSpec {
        name: "Wizard",
        module: "wizard",
        features: &[
            f("<form method=\"post\">", b("1", "1", "1")),
            f("aria-current=\"step\"", b("1", "1", "1")),
            f("<fieldset>", b("1", "1", "1")),
            f("formnovalidate", b("4", "4", "5")),
            f("<progress>", b("8", "16", "6")),
        ],
        fallback: "none needed: one form per step, PRG between them",
        needs_js: NeedsJs::No,
    },
    ComponentSpec {
        name: "Validated form",
        module: "form",
        features: &[
            f("required", b("4", "4", "5")),
            f("pattern", b("4", "4", "5")),
            f(":user-invalid", b("119", "88", "16.5")),
            f("<fieldset>", b("1", "1", "1")),
            f("<output>", b("10", "4", "7")),
            f("type=date", b("20", "57", "14.1")),
            f("accept", b("1", "1", "1")),
            f("field-sizing", b("123", "no", "no")),
        ],
        fallback: "server re-renders with messages; no early styling; textareas keep their rows; the counter shows the submitted length",
        needs_js: NeedsJs::No,
    },
    ComponentSpec {
        name: "Error summary",
        module: "error_summary",
        features: &[
            f("role=\"alert\"", ALWAYS),
            f("aria-labelledby", ALWAYS),
            f("autofocus", b("79", "110", "15.4")),
        ],
        fallback: "where autofocus only works on form controls the summary is still first in the form and read out as an alert",
        needs_js: NeedsJs::Partial("moving the focus into the field a link points to needs script"),
    },
    ComponentSpec {
        name: "Counter",
        module: "counter",
        features: &[
            f("<form method=\"post\">", ALWAYS),
            f("<button name value>", ALWAYS),
            f("<input type=\"number\">", b("6", "29", "5.1")),
            COOKIE,
        ],
        fallback: "none needed",
        needs_js: NeedsJs::No,
    },
    ComponentSpec {
        name: "Theme toggle",
        module: "theme",
        features: &[
            PREFERS_COLOR_SCHEME,
            f("color-scheme", b("81", "96", "13")),
            COOKIE,
        ],
        fallback: "OS preference",
        needs_js: NeedsJs::No,
    },
    ComponentSpec {
        name: "Flash",
        module: "flash",
        features: &[
            COOKIE,
            f("role=\"status\"", ALWAYS),
            f("role=\"alert\"", ALWAYS),
            f("@keyframes", b("43", "16", "9")),
            f("prefers-reduced-motion", b("74", "63", "10.1")),
        ],
        fallback: "without CSS animations the message stays",
        needs_js: NeedsJs::No,
    },
    ComponentSpec {
        name: "UI state",
        module: "state",
        features: &[
            f("links", ALWAYS),
            f("cookies", ALWAYS),
            f("303 See Other", ALWAYS),
        ],
        fallback: "without cookies, state still travels in links on one page",
        needs_js: NeedsJs::No,
    },
    ComponentSpec {
        name: "Select",
        module: "select",
        features: &[
            f("<selectedcontent>", b("135", "no", "27")),
            f("appearance: base-select", b("135", "no", "27")),
            f("<optgroup label>", ALWAYS),
            f("formmethod", b("9", "4", "5.1")),
        ],
        fallback: "plain <select>, chosen server-side",
        needs_js: NeedsJs::No,
    },
    ComponentSpec {
        name: "Range",
        module: "range",
        features: &[
            f("<input type=\"range\">", b("4", "23", "3.1")),
            f("<datalist>", b("20", "110", "12.1")),
            f("pointer-events", ALWAYS),
        ],
        fallback: "ticks not drawn",
        needs_js: NeedsJs::Partial("value shown after submit; live mirroring needs script"),
    },
    ComponentSpec {
        name: "Color",
        module: "color",
        features: &[
            f("<input type=\"color\">", b("20", "29", "12.1")),
            f("color-mix()", b("111", "113", "16.2")),
        ],
        fallback: "text field accepting #rrggbb",
        needs_js: NeedsJs::No,
    },
    ComponentSpec {
        name: "Streaming",
        module: "stream",
        features: &[
            f(
                "<template shadowrootmode=\"open\">",
                b("111", "123", "16.4"),
            ),
            f("<slot name", b("53", "63", "10")),
            f("Chunked transfer", ALWAYS),
        ],
        fallback: "in-order streaming with in-place splicing",
        needs_js: NeedsJs::No,
    },
    ComponentSpec {
        name: "Upload",
        module: "upload",
        features: &[
            f("<input type=\"file\" accept multiple>", ALWAYS),
            f("<progress>", b("6", "6", "6")),
            f("loading=\"lazy\"", b("77", "75", "15.4")),
        ],
        fallback: "none needed: a plain multipart post; the progress bar needs the enhancement script",
        needs_js: NeedsJs::Partial("upload progress and a preview before sending need script"),
    },
    ComponentSpec {
        name: "Kanban",
        module: "kanban",
        features: &[
            f("<form method=\"post\">", ALWAYS),
            f("scroll-snap-type", b("69", "68", "11")),
            f("view-transition-name", b("111", "144", "18")),
        ],
        fallback: "without view transitions a moved card is simply in its new column",
        needs_js: NeedsJs::Partial("drag and drop and reordering within a column need script"),
    },
    ComponentSpec {
        name: "Alert",
        module: "alert",
        features: &[f("role=\"alert\"", ALWAYS), f("role=\"status\"", ALWAYS)],
        fallback: "none needed",
        needs_js: NeedsJs::No,
    },
    ComponentSpec {
        name: "Progress",
        module: "progress",
        features: &[
            f("<progress>", b("6", "6", "6")),
            f("appearance: none", b("84", "80", "15.4")),
        ],
        fallback: "without the pseudo-elements a browser draws its own bar",
        needs_js: NeedsJs::Partial("moving on its own needs a streamed page or the script"),
    },
    ComponentSpec {
        name: "Meter",
        module: "meter",
        features: &[f("<meter>", b("6", "16", "6"))],
        fallback: "without the pseudo-elements a browser draws its own meter",
        needs_js: NeedsJs::No,
    },
    ComponentSpec {
        name: "Tooltip",
        module: "tooltip",
        features: &[
            f(":focus-within", b("60", "52", "10.1")),
            f("@media (hover: none)", b("41", "64", "9")),
        ],
        fallback: "none needed",
        needs_js: NeedsJs::Partial("a delay before opening and Escape to close need script"),
    },
    ComponentSpec {
        name: "Separator",
        module: "separator",
        features: &[f("<hr>", ALWAYS), f("aria-orientation", ALWAYS)],
        fallback: "none needed",
        needs_js: NeedsJs::No,
    },
    ComponentSpec {
        name: "Toast",
        module: "toast",
        features: &[
            f("position: fixed", ALWAYS),
            f("role=\"status\"", ALWAYS),
            f("role=\"alert\"", ALWAYS),
            f("@keyframes", b("43", "16", "9")),
            f("prefers-reduced-motion", b("74", "63", "10.1")),
        ],
        fallback: "without CSS animations toasts stay until the next page",
        needs_js: NeedsJs::No,
    },
    ComponentSpec {
        name: "Breadcrumbs",
        module: "breadcrumbs",
        features: &[
            f("aria-current=\"page\"", ALWAYS),
            f("::before", ALWAYS),
            f("<details>", b("12", "49", "6")),
        ],
        fallback: "none needed",
        needs_js: NeedsJs::No,
    },
    ComponentSpec {
        name: "Skeleton",
        module: "skeleton",
        features: &[
            f("aria-busy", ALWAYS),
            f("role=\"status\"", ALWAYS),
            f("@keyframes", b("43", "16", "9")),
            f("prefers-reduced-motion", b("74", "63", "10.1")),
        ],
        fallback: "without CSS animations the bars are still",
        needs_js: NeedsJs::No,
    },
    ComponentSpec {
        name: "Empty state",
        module: "empty_state",
        features: &[f("<form method=\"post\">", ALWAYS)],
        fallback: "none needed",
        needs_js: NeedsJs::No,
    },
    ComponentSpec {
        name: "Stat",
        module: "stat",
        features: &[f("repeat(auto-fit", b("57", "52", "10.1"))],
        fallback: "none needed",
        needs_js: NeedsJs::No,
    },
    ComponentSpec {
        name: "Chart",
        module: "chart",
        features: &[
            f("<svg>", b("7", "4", "5.1")),
            f("role=\"img\"", ALWAYS),
            f("<title>", ALWAYS),
            f("CSS custom properties in SVG", b("49", "31", "9.1")),
        ],
        fallback: "none needed",
        needs_js: NeedsJs::Partial(
            "zoom, pan and a crosshair that follows the pointer need script",
        ),
    },
    ComponentSpec {
        name: "Sidebar",
        module: "sidebar",
        features: &[f("aria-current=\"page\"", ALWAYS)],
        fallback: "none needed",
        needs_js: NeedsJs::Partial("collapsing to an icon rail kept between pages needs script"),
    },
    ComponentSpec {
        name: "Navigation menu",
        module: "nav_menu",
        features: &[
            f("popover", b("114", "125", "17")),
            f("aria-current=\"page\"", ALWAYS),
        ],
        fallback: "a <details> dropdown without popover; a centred panel without anchor positioning",
        needs_js: NeedsJs::Partial("opening a panel on hover needs script"),
    },
    ComponentSpec {
        name: "Description list",
        module: "description_list",
        features: &[f("<dl>", ALWAYS), f("grid", b("57", "52", "10.1"))],
        fallback: "without grid the terms stack above their details",
        needs_js: NeedsJs::No,
    },
    ComponentSpec {
        name: "Toggle group",
        module: "toggle_group",
        features: &[
            f("<fieldset>", ALWAYS),
            f(":checked", ALWAYS),
            f(":focus-visible", b("86", "85", "15.4")),
        ],
        fallback: "none needed",
        needs_js: NeedsJs::Partial(
            "applying a choice the moment it is pressed needs script (or the form's submit)",
        ),
    },
    ComponentSpec {
        name: "Context menu",
        module: "context_menu",
        features: &[
            f("popover", b("114", "125", "17")),
            f("popovertarget", b("114", "125", "17")),
        ],
        fallback: "that of the popover menu: a <details> dropdown",
        needs_js: NeedsJs::Partial("opening on right-click or a long press needs script"),
    },
    ComponentSpec {
        name: "One-time code",
        module: "input_otp",
        features: &[
            f("autocomplete=\"one-time-code\"", b("84", "no", "12")),
            f("inputmode=\"numeric\"", b("66", "95", "12.1")),
            f("pattern", b("4", "4", "5")),
        ],
        fallback: "a plain spaced-out field; without autocomplete the code is typed or pasted",
        needs_js: NeedsJs::No,
    },
    ComponentSpec {
        name: "Drawer",
        module: "drawer",
        features: &[
            f("<dialog>", b("37", "98", "15.4")),
            f("command=\"show-modal\"", b("135", "144", "26.2")),
            f("closedby", b("134", "141", "26")),
            f("@starting-style", b("117", "129", "17.5")),
            f("@media", ALWAYS),
        ],
        fallback: "link to #id and a :target rule; open from the server",
        needs_js: NeedsJs::No,
    },
    ComponentSpec {
        name: "Command palette",
        module: "palette",
        features: &[
            f("popover", b("114", "125", "17")),
            f("<datalist>", b("20", "4", "12.1")),
            f("<search>", b("118", "118", "17")),
            f("accesskey", ALWAYS),
        ],
        fallback: "a <details> disclosure with the same form",
        needs_js: NeedsJs::Partial(
            "arrow keys through live results and a global Ctrl+K need script",
        ),
    },
];

fn json_str(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

/// The spec as pretty-printed JSON, the content of `spec/components.json`.
pub fn to_json() -> String {
    let mut out = String::from("{\n  \"components\": [\n");
    for (i, c) in SPECS.iter().enumerate() {
        out.push_str("    {\n");
        out.push_str(&format!("      \"name\": {},\n", json_str(c.name)));
        out.push_str(&format!("      \"module\": {},\n", json_str(c.module)));
        out.push_str("      \"features\": [\n");
        for (j, ft) in c.features.iter().enumerate() {
            out.push_str(&format!(
                "        {{ \"name\": {}, \"baseline\": {{ \"chrome\": {}, \"firefox\": {}, \"safari\": {} }} }}{}\n",
                json_str(ft.name),
                json_str(ft.baseline.chrome),
                json_str(ft.baseline.firefox),
                json_str(ft.baseline.safari),
                if j + 1 < c.features.len() { "," } else { "" }
            ));
        }
        out.push_str("      ],\n");
        out.push_str(&format!("      \"fallback\": {},\n", json_str(c.fallback)));
        let (needs, note) = match c.needs_js {
            NeedsJs::No => ("no", None),
            NeedsJs::Partial(n) => ("partial", Some(n)),
        };
        out.push_str(&format!("      \"needs_js\": {}", json_str(needs)));
        if let Some(n) = note {
            out.push_str(&format!(",\n      \"needs_js_note\": {}", json_str(n)));
        }
        let builders: Vec<_> = crate::props()
            .iter()
            .filter(|b| b.module == c.module)
            .collect();
        if !builders.is_empty() {
            out.push_str(",\n      \"builders\": [\n");
            for (j, b) in builders.iter().enumerate() {
                let calls: Vec<String> = b.calls.iter().map(|c| json_str(c)).collect();
                out.push_str(&format!(
                    "        {{\n          \"builder\": {},\n          \"lui\": {},\n          \"calls\": [{}],\n          \"props\": [",
                    json_str(b.builder),
                    json_str(&b.lui()),
                    calls.join(", ")
                ));
                for (k, p) in b.props.iter().enumerate() {
                    out.push_str(&format!(
                        "\n            {{ \"name\": {}, \"kind\": {}, \"args\": {}, \"default\": {}, \"attr\": {}, \"doc\": {} }}{}",
                        json_str(p.name),
                        json_str(p.kind.as_str()),
                        json_str(p.args),
                        json_str(p.default),
                        json_str(p.attr),
                        json_str(p.doc),
                        if k + 1 < b.props.len() { "," } else { "" }
                    ));
                }
                if !b.props.is_empty() {
                    out.push_str("\n          ");
                }
                out.push_str(&format!(
                    "]\n        }}{}\n",
                    if j + 1 < builders.len() { "," } else { "" }
                ));
            }
            out.push_str("      ]");
        }
        out.push_str(&format!(
            "\n    }}{}\n",
            if i + 1 < SPECS.len() { "," } else { "" }
        ));
    }
    out.push_str("  ]\n}\n");
    out
}

/// The README feature matrix, one row per component.
pub fn markdown_table() -> String {
    let mut out = String::from(
        "| Component | Platform features | Chrome / Firefox / Safari | Fallback | Needs JS? |\n|---|---|---|---|---|\n",
    );
    for c in SPECS {
        let features: Vec<String> = c
            .features
            .iter()
            .map(|ft| format!("`{}`", ft.name))
            .collect();
        let versions: Vec<String> = c
            .features
            .iter()
            .map(|ft| {
                format!(
                    "{} / {} / {}",
                    ft.baseline.chrome, ft.baseline.firefox, ft.baseline.safari
                )
            })
            .collect();
        let needs = match c.needs_js {
            NeedsJs::No => "No".to_string(),
            NeedsJs::Partial(n) => format!("Partly: {n}"),
        };
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            c.name,
            features.join(", "),
            versions.join("; "),
            c.fallback,
            needs
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `(module, source)` for every component file, so the header check needs no I/O.
    const SOURCES: &[(&str, &str)] = &[
        ("enhance", include_str!("enhance.rs")),
        ("layout", include_str!("layout.rs")),
        ("caps", include_str!("../../loco-ui-caps/src/lib.rs")),
        ("button", include_str!("button.rs")),
        ("input", include_str!("input.rs")),
        ("badge", include_str!("badge.rs")),
        ("card", include_str!("card.rs")),
        ("icon", include_str!("icon.rs")),
        ("avatar", include_str!("avatar.rs")),
        ("stack", include_str!("stack.rs")),
        ("cluster", include_str!("cluster.rs")),
        ("grid", include_str!("grid.rs")),
        ("split", include_str!("split.rs")),
        ("calendar", include_str!("calendar.rs")),
        ("date_picker", include_str!("date_picker.rs")),
        ("dialog", include_str!("dialog.rs")),
        ("popover", include_str!("popover.rs")),
        ("tabs", include_str!("tabs.rs")),
        ("accordion", include_str!("accordion.rs")),
        ("combobox", include_str!("combobox.rs")),
        ("pager", include_str!("pager.rs")),
        ("table", include_str!("table.rs")),
        ("paged_table", include_str!("paged_table.rs")),
        ("wizard", include_str!("wizard.rs")),
        ("form", include_str!("form.rs")),
        ("error_summary", include_str!("error_summary.rs")),
        ("counter", include_str!("counter.rs")),
        ("theme", include_str!("theme.rs")),
        ("flash", include_str!("flash.rs")),
        ("state", include_str!("state.rs")),
        ("select", include_str!("select.rs")),
        ("range", include_str!("range.rs")),
        ("color", include_str!("color.rs")),
        ("stream", include_str!("stream.rs")),
        ("upload", include_str!("upload.rs")),
        ("kanban", include_str!("kanban.rs")),
        ("alert", include_str!("alert.rs")),
        ("progress", include_str!("progress.rs")),
        ("meter", include_str!("meter.rs")),
        ("tooltip", include_str!("tooltip.rs")),
        ("separator", include_str!("separator.rs")),
        ("toast", include_str!("toast.rs")),
        ("breadcrumbs", include_str!("breadcrumbs.rs")),
        ("skeleton", include_str!("skeleton.rs")),
        ("empty_state", include_str!("empty_state.rs")),
        ("stat", include_str!("stat.rs")),
        ("chart", include_str!("chart.rs")),
        ("sidebar", include_str!("sidebar.rs")),
        ("nav_menu", include_str!("nav_menu.rs")),
        ("description_list", include_str!("description_list.rs")),
        ("toggle_group", include_str!("toggle_group.rs")),
        ("context_menu", include_str!("context_menu.rs")),
        ("input_otp", include_str!("input_otp.rs")),
        ("drawer", include_str!("drawer.rs")),
        ("palette", include_str!("palette.rs")),
    ];

    fn header(source: &str) -> String {
        source
            .lines()
            .take_while(|l| l.starts_with("//!"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn every_component_has_a_spec_and_every_spec_a_file() {
        let modules: Vec<&str> = SPECS.iter().map(|c| c.module).collect();
        let files: Vec<&str> = SOURCES.iter().map(|(m, _)| *m).collect();
        assert_eq!(modules, files, "spec order and file list must match");
    }

    #[test]
    fn doc_headers_match_the_spec() {
        for c in SPECS {
            let (_, source) = SOURCES.iter().find(|(m, _)| *m == c.module).unwrap();
            let head = header(source);
            assert!(
                head.contains("**Platform features:**"),
                "{}: no Platform features line",
                c.module
            );
            assert!(
                head.contains("**Fallback:**"),
                "{}: no Fallback line",
                c.module
            );
            if !["enhance", "layout", "caps", "state"].contains(&c.module) {
                assert!(
                    head.contains("**What it does not do without script:**"),
                    "{}: no \"What it does not do without script\" paragraph",
                    c.module
                );
            }
            assert!(head.contains("```rust"), "{}: no usage example", c.module);
            for ft in c.features {
                assert!(
                    head.contains(ft.name),
                    "{}: header does not mention `{}`",
                    c.module,
                    ft.name
                );
            }
            for browser in c
                .features
                .iter()
                .flat_map(|ft| [ft.baseline.chrome, ft.baseline.firefox, ft.baseline.safari])
            {
                assert!(
                    browser == "no" || browser.parse::<f32>().is_ok(),
                    "{}: bad version {browser}",
                    c.module
                );
            }
        }
    }

    #[test]
    fn json_file_is_generated_from_this_spec() {
        let on_disk = include_str!("../../spec/components.json");
        assert_eq!(on_disk, to_json(), "run `cargo run -p demo -- spec write`");
    }

    #[test]
    fn readme_matrix_is_generated_from_this_spec() {
        let readme = include_str!("../../README.md");
        let start = readme
            .find("<!-- matrix:start -->")
            .expect("README matrix start marker");
        let end = readme
            .find("<!-- matrix:end -->")
            .expect("README matrix end marker");
        let section = &readme[start + "<!-- matrix:start -->".len()..end];
        assert_eq!(
            section.trim(),
            markdown_table().trim(),
            "run `cargo run -p demo -- spec write`"
        );
    }
}
