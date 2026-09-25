//! # nojs!
//!
//! Maud's `html!` with components written like elements. Everything that is plain Maud passes
//! through untouched; a capitalized name is a component, and it expands to the same builder
//! chain a route would write by hand, so `nojs!` and the dot form are one code path.
//!
//! ```rust
//! use axum_nojs::prelude::*;
//!
//! let ui = Ui::from_request("/", "", "");
//! let narrow = true;
//! let projects = [("Site", 3), ("App", 0)];
//! let page = nojs! {
//!     h1 { "Projects" }
//!     Tabs("projects") vertical[narrow] {
//!         @for (name, open) in projects {
//!             tab (name) badge=(open) { p { "Open issues: " (open) } }
//!         }
//!         lazy "Archive" || { p { "Rendered only when opened." } }
//!     }
//! };
//! let by_hand = html! {
//!     h1 { "Projects" }
//!     (ui.tabs("projects").vertical()
//!         .tab("Site", html! { p { "Open issues: " (3) } }).badge(3)
//!         .tab("App", html! { p { "Open issues: " (0) } }).badge(0)
//!         .lazy("Archive", || html! { p { "Rendered only when opened." } }))
//! };
//! assert_eq!(page.into_string(), by_hand.into_string());
//! ```
//!
//! ## The rules
//!
//! - **A component** is a capitalized name: `DatePicker(..)` is `ui.date_picker(..)` (the name
//!   in snake case), and `(..)` holds the required arguments, as in the method call. A
//!   component with none leaves the parentheses out (`Card title="Plan" { .. }`). It ends with
//!   `;` or with a block.
//! - **Attributes** are setters: `x="v"` or `x=(expr)` is `.x(v)`, `x=(a, b)` is `.x(a, b)`, a
//!   bare `x` is `.x()`, `x[cond]` calls `.x()` only when `cond` holds, and `x=[option]`
//!   calls `.x(v)` only when the option is `Some(v)` (Maud's toggle and optional syntax).
//!   `x=|i| { markup }` passes a closure returning markup (`rows=|i| { .. }` on a pager).
//! - **A component's block** is either items or a body. It holds items when it starts with
//!   one: a lowercase name followed by its arguments (`tab "Use"`, `column "name" "Name"`,
//!   `tab (p.title)`, `separator()`). An item's attributes are the modifiers that apply to it
//!   (`tab "Use" badge=3`), its block is its last argument as markup, `|| { .. }` passes that
//!   markup as a closure instead (`lazy "Why" || { .. }`), and a block that itself holds items
//!   continues the chain (a kanban `column` with its `card`s). Otherwise the block is plain
//!   markup, passed to `.body(..)`.
//! - **`@for`, `@if` / `@else`, `@match` and `@let`** work among items as they do in Maud, so
//!   items built from data stay inline.
//! - **`ui`** is taken from the scope by that name; `nojs!(ctx => ..)` names another.
//!
//! A misspelled attribute is rustc's own error at that attribute ("no method named `vertcal`
//! … a method with a similar name exists: `vertical`"), since every expanded call keeps the
//! span it came from.

use proc_macro2::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree};
use quote::{quote, quote_spanned};

/// Maud's `html!` with components written like elements; see the crate documentation.
#[proc_macro]
pub fn nojs(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut tokens: Vec<TokenTree> = TokenStream::from(input).into_iter().collect();
    let mut ui = Ident::new("ui", Span::call_site());
    if let [
        TokenTree::Ident(name),
        TokenTree::Punct(eq),
        TokenTree::Punct(gt),
        ..,
    ] = &tokens[..]
        && eq.as_char() == '='
        && eq.spacing() == Spacing::Joint
        && gt.as_char() == '>'
    {
        ui = name.clone();
        tokens.drain(..3);
    }
    let cx = Cx { ui };
    match cx.markup(&tokens) {
        Ok(markup) => quote!(::maud::html! { #markup }).into(),
        Err((span, message)) => quote_spanned!(span=> ::core::compile_error!(#message)).into(),
    }
}

type Result<T> = std::result::Result<T, (Span, String)>;

struct Cx {
    ui: Ident,
}

/// The builder variable of an expansion; hygienic, so it never meets a caller's name.
fn b() -> Ident {
    Ident::new("__nojs_builder", Span::mixed_site())
}

fn punct(t: Option<&TokenTree>, c: char) -> bool {
    matches!(t, Some(TokenTree::Punct(p)) if p.as_char() == c)
}

fn brace(t: Option<&TokenTree>) -> Option<&Group> {
    match t {
        Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => Some(g),
        _ => None,
    }
}

fn paren(t: Option<&TokenTree>) -> Option<&Group> {
    match t {
        Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Parenthesis => Some(g),
        _ => None,
    }
}

fn keyword(t: Option<&TokenTree>) -> Option<String> {
    match t {
        Some(TokenTree::Ident(i)) => Some(i.to_string()),
        _ => None,
    }
}

fn starts_upper(i: &Ident) -> bool {
    i.to_string().starts_with(|c: char| c.is_ascii_uppercase())
}

/// `DatePicker` → `date_picker`.
fn snake(name: &str) -> String {
    let mut out = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

fn stream(tokens: &[TokenTree]) -> TokenStream {
    tokens.iter().cloned().collect()
}

/// `tokens` up to (not including) the first top-level brace group, and its index.
fn until_brace(tokens: &[TokenTree], from: usize, what: &str) -> Result<usize> {
    (from..tokens.len())
        .find(|&i| brace(tokens.get(i)).is_some())
        .ok_or_else(|| {
            (
                tokens[from.min(tokens.len() - 1)].span(),
                format!("`{what}` needs a `{{ .. }}` block"),
            )
        })
}

/// Whether `tokens[i]` starts an item: a lowercase name followed by an argument.
fn item_at(tokens: &[TokenTree], i: usize) -> bool {
    matches!(tokens.get(i), Some(TokenTree::Ident(name)) if !starts_upper(name))
        && matches!(
            tokens.get(i + 1),
            Some(TokenTree::Literal(_)) | Some(TokenTree::Group(_))
        )
        && brace(tokens.get(i + 1)).is_none()
}

/// Whether a component's block holds items: its first entry (looking inside control flow)
/// is an item, or it is empty.
fn holds_items(tokens: &[TokenTree]) -> bool {
    let mut i = 0;
    while i < tokens.len() {
        if punct(tokens.get(i), '@') {
            if keyword(tokens.get(i + 1)).as_deref() == Some("let") {
                i = (i..tokens.len())
                    .find(|&j| punct(tokens.get(j), ';'))
                    .map_or(tokens.len(), |j| j + 1);
                continue;
            }
            return match (i..tokens.len()).find_map(|j| brace(tokens.get(j))) {
                Some(g) if keyword(tokens.get(i + 1)).as_deref() == Some("match") => {
                    let arms: Vec<TokenTree> = g.stream().into_iter().collect();
                    let at = arms
                        .windows(2)
                        .position(|w| punct(w.first(), '=') && punct(w.get(1), '>'));
                    at.is_some_and(|at| {
                        brace(arms.get(at + 2)).map_or_else(
                            || item_at(&arms, at + 2),
                            |g| holds_items(&g.stream().into_iter().collect::<Vec<_>>()),
                        )
                    })
                }
                Some(g) => holds_items(&g.stream().into_iter().collect::<Vec<_>>()),
                None => false,
            };
        }
        return item_at(tokens, i);
    }
    true
}

impl Cx {
    /// Plain Maud markup, with every component in it expanded to a splice.
    fn markup(&self, tokens: &[TokenTree]) -> Result<TokenStream> {
        let mut out = TokenStream::new();
        let mut i = 0;
        while i < tokens.len() {
            match &tokens[i] {
                TokenTree::Punct(p) if p.as_char() == '@' => {
                    let kw = keyword(tokens.get(i + 1)).unwrap_or_default();
                    if kw == "let" {
                        let end = (i..tokens.len())
                            .find(|&j| punct(tokens.get(j), ';'))
                            .ok_or_else(|| (p.span(), "`@let` needs a `;`".to_string()))?;
                        out.extend(stream(&tokens[i..=end]));
                        i = end + 1;
                        continue;
                    }
                    let at = until_brace(tokens, i, &format!("@{kw}"))?;
                    out.extend(stream(&tokens[i..at]));
                    let g = brace(tokens.get(at)).unwrap();
                    let inner: Vec<TokenTree> = g.stream().into_iter().collect();
                    let body = if kw == "match" {
                        self.arms(&inner)?
                    } else {
                        self.markup(&inner)?
                    };
                    out.extend([regroup(g, body)]);
                    i = at + 1;
                }
                TokenTree::Group(g) if g.delimiter() == Delimiter::Brace => {
                    let inner: Vec<TokenTree> = g.stream().into_iter().collect();
                    out.extend([regroup(g, self.markup(&inner)?)]);
                    i += 1;
                }
                TokenTree::Ident(name) if starts_upper(name) => {
                    let (expr, next) = self.component(tokens, i)?;
                    out.extend([TokenTree::Group(Group::new(Delimiter::Parenthesis, expr))]);
                    i = next;
                }
                TokenTree::Ident(_) | TokenTree::Punct(_) => {
                    // An element (`p`, `a href=..`, `.class`, `#id`, `my-el`) up to its block or `;`.
                    let mut j = i;
                    loop {
                        match tokens.get(j) {
                            None => {
                                out.extend(stream(&tokens[i..]));
                                return Ok(out);
                            }
                            Some(TokenTree::Punct(p)) if p.as_char() == ';' && j > i => {
                                out.extend(stream(&tokens[i..=j]));
                                break;
                            }
                            Some(TokenTree::Group(g))
                                if g.delimiter() == Delimiter::Brace
                                    && !punct(tokens.get(j - 1), '=') =>
                            {
                                out.extend(stream(&tokens[i..j]));
                                let inner: Vec<TokenTree> = g.stream().into_iter().collect();
                                out.extend([regroup(g, self.markup(&inner)?)]);
                                break;
                            }
                            _ => j += 1,
                        }
                    }
                    i = j + 1;
                }
                other => {
                    out.extend([other.clone()]);
                    i += 1;
                }
            }
        }
        Ok(out)
    }

    /// The arms of a markup `@match`: patterns copied, each arm's markup expanded.
    fn arms(&self, tokens: &[TokenTree]) -> Result<TokenStream> {
        let mut out = TokenStream::new();
        let mut i = 0;
        while i < tokens.len() {
            let Some(arrow) = (i..tokens.len())
                .find(|&j| punct(tokens.get(j), '=') && punct(tokens.get(j + 1), '>'))
            else {
                out.extend(stream(&tokens[i..]));
                break;
            };
            out.extend(stream(&tokens[i..arrow + 2]));
            let end = (arrow + 2..tokens.len())
                .find(|&j| punct(tokens.get(j), ','))
                .unwrap_or(tokens.len());
            out.extend(self.markup(&tokens[arrow + 2..end])?);
            out.extend(tokens.get(end).cloned());
            i = end + 1;
        }
        Ok(out)
    }

    /// A component at `tokens[at]`: the builder expression and the index after it.
    fn component(&self, tokens: &[TokenTree], at: usize) -> Result<(TokenStream, usize)> {
        let TokenTree::Ident(name) = &tokens[at] else {
            unreachable!()
        };
        let method = Ident::new(&snake(&name.to_string()), name.span());
        let ui = &self.ui;
        let b = b();
        let mut i = at + 1;
        let args = match paren(tokens.get(i)) {
            Some(g) => {
                i += 1;
                g.clone()
            }
            None => {
                let mut g = Group::new(Delimiter::Parenthesis, TokenStream::new());
                g.set_span(name.span());
                g
            }
        };
        let mut steps = vec![quote!(let #b = #ui.#method #args;)];
        i = self.attributes(tokens, i, &mut steps)?;
        match tokens.get(i) {
            Some(TokenTree::Punct(p)) if p.as_char() == ';' => i += 1,
            Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => {
                let inner: Vec<TokenTree> = g.stream().into_iter().collect();
                if holds_items(&inner) {
                    self.items(&inner, &mut steps)?;
                } else {
                    let body = self.markup(&inner)?;
                    let call = Ident::new("body", g.span_open());
                    steps.push(quote!(let #b = #b.#call(::maud::html! { #body });));
                }
                i += 1;
            }
            None => {}
            Some(other) => {
                return Err((
                    other.span(),
                    format!("`{name}` ends with `;` or a `{{ .. }}` block"),
                ));
            }
        }
        Ok((quote!({ #(#steps)* #b }), i))
    }

    /// Attributes from `tokens[i]` up to a `;`, a block or a closure body; each becomes a step.
    fn attributes(
        &self,
        tokens: &[TokenTree],
        mut i: usize,
        steps: &mut Vec<TokenStream>,
    ) -> Result<usize> {
        let b = b();
        while let Some(TokenTree::Ident(name)) = tokens.get(i) {
            if name == "move" && punct(tokens.get(i + 1), '|') {
                break;
            }
            i += 1;
            match tokens.get(i) {
                Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Bracket => {
                    let cond = g.stream();
                    steps.push(quote!(let #b = if #cond { #b.#name() } else { #b };));
                    i += 1;
                }
                Some(TokenTree::Punct(p)) if p.as_char() == '=' => {
                    i += 1;
                    let (value, next) = self.value(tokens, i, name)?;
                    i = next;
                    match value {
                        Value::Optional(option) => {
                            let v = Ident::new("__nojs_value", Span::mixed_site());
                            steps.push(quote!(let #b = match #option {
                                ::core::option::Option::Some(#v) => #b.#name(#v),
                                ::core::option::Option::None => #b,
                            };));
                        }
                        Value::Args(args) => steps.push(quote!(let #b = #b.#name(#args);)),
                    }
                }
                _ => steps.push(quote!(let #b = #b.#name();)),
            }
        }
        Ok(i)
    }

    /// An attribute's value at `tokens[i]` and the index after it.
    fn value(&self, tokens: &[TokenTree], i: usize, name: &Ident) -> Result<(Value, usize)> {
        match tokens.get(i) {
            Some(TokenTree::Literal(l)) => Ok((Value::Args(quote!(#l)), i + 1)),
            Some(TokenTree::Punct(p)) if p.as_char() == '-' => {
                let lit = tokens.get(i + 1).cloned();
                Ok((Value::Args(quote!(-#lit)), i + 2))
            }
            Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Parenthesis => {
                Ok((Value::Args(g.stream()), i + 1))
            }
            Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Bracket => {
                Ok((Value::Optional(g.stream()), i + 1))
            }
            Some(TokenTree::Punct(p)) if p.as_char() == '|' => {
                let (closure, next) = self.closure(tokens, i)?;
                Ok((Value::Args(closure), next))
            }
            Some(TokenTree::Ident(m)) if m == "move" => {
                let (closure, next) = self.closure(tokens, i)?;
                Ok((Value::Args(closure), next))
            }
            other => Err((
                other.map_or(name.span(), TokenTree::span),
                format!(
                    "`{name}=` takes a literal, `(expr)`, `[option]` or a closure `|..| {{ .. }}`"
                ),
            )),
        }
    }

    /// `move? |params| { markup }` at `tokens[i]` as a closure returning markup.
    fn closure(&self, tokens: &[TokenTree], mut i: usize) -> Result<(TokenStream, usize)> {
        let mut head = TokenStream::new();
        if keyword(tokens.get(i)).as_deref() == Some("move") {
            head.extend([tokens[i].clone()]);
            i += 1;
        }
        let start = i;
        let open = tokens[i].clone();
        // `||` or `|params|`
        let close = if matches!(&tokens[i], TokenTree::Punct(p) if p.spacing() == Spacing::Joint)
            && punct(tokens.get(i + 1), '|')
        {
            i + 1
        } else {
            (i + 1..tokens.len())
                .find(|&j| punct(tokens.get(j), '|'))
                .ok_or_else(|| {
                    (
                        open.span(),
                        "a closure's parameters end with `|`".to_string(),
                    )
                })?
        };
        head.extend(stream(&tokens[start..=close]));
        let Some(g) = brace(tokens.get(close + 1)) else {
            return Err((
                open.span(),
                "a closure here is `|..| { markup }`".to_string(),
            ));
        };
        let inner: Vec<TokenTree> = g.stream().into_iter().collect();
        let body = self.markup(&inner)?;
        Ok((quote!(#head ::maud::html! { #body }), close + 2))
    }

    /// The items of a component's block, each a step on the builder.
    fn items(&self, tokens: &[TokenTree], steps: &mut Vec<TokenStream>) -> Result<()> {
        let b = b();
        let mut i = 0;
        while i < tokens.len() {
            if punct(tokens.get(i), '@') {
                i = self.control(tokens, i, steps)?;
                continue;
            }
            let Some(TokenTree::Ident(name)) = tokens
                .get(i)
                .filter(|_| !matches!(&tokens[i], TokenTree::Ident(n) if starts_upper(n)))
            else {
                return Err((
                    tokens[i].span(),
                    "a component's block holds items (`tab \"Title\" { .. }`) or markup, not both; \
                     put markup inside an item's block"
                        .to_string(),
                ));
            };
            i += 1;
            // Arguments: literals, negative literals and `(..)` groups, before any attribute.
            let mut args: Vec<TokenStream> = Vec::new();
            let first = i;
            loop {
                match tokens.get(i) {
                    Some(TokenTree::Literal(l)) => args.push(quote!(#l)),
                    Some(TokenTree::Punct(p)) if p.as_char() == '-' => {
                        let lit = tokens.get(i + 1).cloned();
                        args.push(quote!(-#lit));
                        i += 1;
                    }
                    Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Parenthesis => {
                        if !g.stream().is_empty() {
                            args.push(g.stream());
                        }
                    }
                    _ => break,
                }
                i += 1;
            }
            if i == first && brace(tokens.get(i)).is_some() {
                return Err((
                    name.span(),
                    format!(
                        "`{name} {{ .. }}` is markup, but this block holds items; put markup inside \
                         an item's block (an item with no arguments is `{name}()`)"
                    ),
                ));
            }
            let mut modifiers = Vec::new();
            i = self.attributes(tokens, i, &mut modifiers)?;
            let mut nested = Vec::new();
            match tokens.get(i) {
                Some(TokenTree::Punct(p)) if p.as_char() == ';' => i += 1,
                Some(TokenTree::Punct(p)) if p.as_char() == '|' => {
                    let (closure, next) = self.closure(tokens, i)?;
                    args.push(closure);
                    i = next;
                }
                Some(TokenTree::Ident(m)) if m == "move" => {
                    let (closure, next) = self.closure(tokens, i)?;
                    args.push(closure);
                    i = next;
                }
                Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => {
                    let inner: Vec<TokenTree> = g.stream().into_iter().collect();
                    if !inner.is_empty() && holds_items(&inner) {
                        self.items(&inner, &mut nested)?;
                    } else {
                        let body = self.markup(&inner)?;
                        args.push(quote!(::maud::html! { #body }));
                    }
                    i += 1;
                }
                None => {}
                Some(other) => {
                    return Err((
                        other.span(),
                        format!("item `{name}` ends with `;` or a `{{ .. }}` block"),
                    ));
                }
            }
            let mut call = Group::new(Delimiter::Parenthesis, quote!(#(#args),*));
            call.set_span(name.span());
            steps.push(quote!(let #b = #b.#name #call;));
            steps.extend(modifiers);
            steps.extend(nested);
        }
        Ok(())
    }

    /// `@for`, `@if`/`@else`, `@match` or `@let` among items, at `tokens[at]` (the `@`).
    fn control(
        &self,
        tokens: &[TokenTree],
        at: usize,
        steps: &mut Vec<TokenStream>,
    ) -> Result<usize> {
        let b = b();
        let kw = keyword(tokens.get(at + 1)).unwrap_or_default();
        let scoped = |cx: &Cx, g: &Group| -> Result<TokenStream> {
            let inner: Vec<TokenTree> = g.stream().into_iter().collect();
            let mut inner_steps = Vec::new();
            cx.items(&inner, &mut inner_steps)?;
            Ok(quote!({ #(#inner_steps)* #b }))
        };
        match kw.as_str() {
            "let" => {
                let end = (at..tokens.len())
                    .find(|&j| punct(tokens.get(j), ';'))
                    .ok_or_else(|| (tokens[at].span(), "`@let` needs a `;`".to_string()))?;
                let binding = stream(&tokens[at + 1..=end]);
                steps.push(binding);
                Ok(end + 1)
            }
            "for" => {
                let open = until_brace(tokens, at, "@for")?;
                let head = stream(&tokens[at + 2..open]);
                let body = scoped(self, brace(tokens.get(open)).unwrap())?;
                steps.push(quote!(let mut #b = #b; for #head { #b = #body; } let #b = #b;));
                Ok(open + 1)
            }
            "if" => {
                let mut chain = TokenStream::new();
                let mut i = at + 1;
                let mut closed = false;
                loop {
                    // `if cond { .. }` at `tokens[i]` (`i` on `if`), or a final `{ .. }`.
                    let open = until_brace(tokens, i, "@if")?;
                    let head = stream(&tokens[i..open]);
                    let body = scoped(self, brace(tokens.get(open)).unwrap())?;
                    chain.extend(quote!(#head #body));
                    i = open + 1;
                    if keyword(tokens.get(i)).is_none()
                        && punct(tokens.get(i), '@')
                        && keyword(tokens.get(i + 1)).as_deref() == Some("else")
                    {
                        chain.extend(quote!(else));
                        i += 2;
                        if keyword(tokens.get(i)).as_deref() == Some("if") {
                            continue;
                        }
                        let open = until_brace(tokens, i, "@else")?;
                        chain.extend(scoped(self, brace(tokens.get(open)).unwrap())?);
                        i = open + 1;
                        closed = true;
                    }
                    break;
                }
                if !closed {
                    chain.extend(quote!(else { #b }));
                }
                steps.push(quote!(let #b = #chain;));
                Ok(i)
            }
            "match" => {
                let open = until_brace(tokens, at, "@match")?;
                let scrutinee = stream(&tokens[at + 2..open]);
                let arms: Vec<TokenTree> = brace(tokens.get(open))
                    .unwrap()
                    .stream()
                    .into_iter()
                    .collect();
                let mut out = TokenStream::new();
                let mut i = 0;
                while i < arms.len() {
                    let Some(arrow) = (i..arms.len())
                        .find(|&j| punct(arms.get(j), '=') && punct(arms.get(j + 1), '>'))
                    else {
                        break;
                    };
                    out.extend(stream(&arms[i..arrow + 2]));
                    let (body, next) = match brace(arms.get(arrow + 2)) {
                        Some(g) => (scoped(self, g)?, arrow + 3),
                        None => {
                            let end = (arrow + 2..arms.len())
                                .find(|&j| punct(arms.get(j), ','))
                                .unwrap_or(arms.len());
                            let mut inner_steps = Vec::new();
                            self.items(&arms[arrow + 2..end], &mut inner_steps)?;
                            (quote!({ #(#inner_steps)* #b }), end)
                        }
                    };
                    out.extend(body);
                    out.extend([TokenTree::Punct(Punct::new(',', Spacing::Alone))]);
                    i = if punct(arms.get(next), ',') {
                        next + 1
                    } else {
                        next
                    };
                }
                steps.push(quote!(let #b = match #scrutinee { #out };));
                Ok(open + 1)
            }
            other => Err((
                tokens[at].span(),
                format!(
                    "`@{other}` is not supported among items; use `@for`, `@if`, `@match` or `@let`"
                ),
            )),
        }
    }
}

enum Value {
    /// Arguments for the setter call.
    Args(TokenStream),
    /// `[option]`: the setter is called with the value when there is one.
    Optional(TokenStream),
}

/// `g` with new contents, keeping its delimiter and span.
fn regroup(g: &Group, inner: TokenStream) -> TokenTree {
    let mut out = Group::new(g.delimiter(), inner);
    out.set_span(g.span());
    TokenTree::Group(out)
}
