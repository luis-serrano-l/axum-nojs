//! The code behind each component page: the lines between `// code: <href>` and `// end code`
//! in the route files, highlighted on the server.

use crate::site::COMPONENTS;
use maud::{Markup, html};
use std::sync::LazyLock;

/// Every file with `// code:` markers, as `(path, source)`, so every component page can show
/// the code that draws it. A new route file goes here.
pub(crate) const SOURCES: [(&str, &str); 11] = [
    (
        "demo/src/routes/primitives.rs",
        include_str!("routes/primitives.rs"),
    ),
    (
        "demo/src/routes/overlays.rs",
        include_str!("routes/overlays.rs"),
    ),
    (
        "demo/src/routes/disclosure.rs",
        include_str!("routes/disclosure.rs"),
    ),
    (
        "demo/src/routes/navigation.rs",
        include_str!("routes/navigation.rs"),
    ),
    ("demo/src/routes/input.rs", include_str!("routes/input.rs")),
    (
        "demo/src/routes/feedback.rs",
        include_str!("routes/feedback.rs"),
    ),
    ("demo/src/routes/table.rs", include_str!("routes/table.rs")),
    (
        "demo/src/routes/server_state.rs",
        include_str!("routes/server_state.rs"),
    ),
    (
        "demo/src/routes/widgets.rs",
        include_str!("routes/widgets.rs"),
    ),
    ("demo/src/routes/flows.rs", include_str!("routes/flows.rs")),
    ("demo/src/routes/own.rs", include_str!("routes/own.rs")),
];

/// The lines between `// code: <href>` and `// end code`, dedented, one block per pair, and the
/// file they are in: a page shows the component call, and the handler or helper next to it if
/// it has one. A page's markers all sit in one file.
pub(crate) fn snippet(href: &str) -> (&'static str, String) {
    let open = format!("// code: {href}");
    let Some((path, source)) = SOURCES
        .iter()
        .find(|(_, s)| s.lines().any(|l| l.trim() == open))
    else {
        return ("", String::new());
    };
    let mut lines = source.lines();
    let mut blocks = Vec::new();
    while lines.any(|l| l.trim() == open) {
        let block: Vec<&str> = lines
            .by_ref()
            .take_while(|l| l.trim() != "// end code")
            .collect();
        let indent = block
            .iter()
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.len() - l.trim_start().len())
            .min()
            .unwrap_or(0);
        blocks.push(
            block
                .iter()
                .map(|l| l.get(indent..).unwrap_or(""))
                .collect::<Vec<_>>()
                .join("\n"),
        );
    }
    (path, blocks.join("\n\n"))
}

/// Every component's snippet, highlighted once at startup: `(href, file, html)`.
pub(crate) static CODE: LazyLock<Vec<(&str, &str, String)>> = LazyLock::new(|| {
    COMPONENTS
        .iter()
        .map(|c| {
            let (path, code) = snippet(c.0);
            (c.0, path, highlight(&code).into_string())
        })
        .collect()
});

/// Rust source as spans the stylesheet colours, parsed by syntect's Rust grammar on the
/// server (no script). Only seven classes, `nojs-hl-{k,s,n,c,m,f,t}`, coloured with tokens
/// in `layout.rs`, instead of syntect's own HTML with a class per scope and a bundled theme.
pub(crate) fn highlight(code: &str) -> Markup {
    use syntect::{
        easy::ScopeRangeIterator,
        parsing::{ParseState, ScopeStack, SyntaxSet},
        util::LinesWithEndings,
    };
    static SYNTAXES: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);
    /// Scope prefix to class: keyword, string, number, comment, macro, function, type.
    const CLASSES: [(&str, &str); 11] = [
        ("comment", "c"),
        ("string", "s"),
        ("constant.numeric", "n"),
        ("support.macro", "m"),
        ("support.function", "f"),
        ("entity.name.function", "f"),
        ("keyword.control", "k"),
        ("storage", "k"),
        ("support.type", "t"),
        ("entity.name", "t"),
        ("constant.other", "t"),
    ];
    let class = |stack: &ScopeStack| {
        stack
            .as_slice()
            .iter()
            .rev()
            .find_map(|scope| {
                let name = scope.build_string();
                CLASSES
                    .iter()
                    .find(|(prefix, _)| name.starts_with(prefix))
                    .map(|c| c.1)
            })
            .unwrap_or("")
    };
    let rust = SYNTAXES
        .find_syntax_by_extension("rs")
        .expect("syntect ships Rust");
    let (mut state, mut stack) = (ParseState::new(rust), ScopeStack::new());
    let mut parts: Vec<(&str, String)> = Vec::new();
    for line in LinesWithEndings::from(code) {
        let ops = state.parse_line(line, &SYNTAXES).unwrap_or_default();
        for (range, op) in ScopeRangeIterator::new(&ops, line) {
            stack.apply(op).ok();
            let kind = class(&stack);
            match parts.last_mut() {
                Some((last, text)) if *last == kind => text.push_str(&line[range]),
                _ if range.is_empty() => {}
                _ => parts.push((kind, line[range].to_string())),
            }
        }
    }
    html! { @for (kind, text) in parts { @if kind.is_empty() { (text) } @else { span class={ "nojs-hl-" (kind) } { (text) } } } }
}
