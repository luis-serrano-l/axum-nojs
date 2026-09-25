//! The playground under a component's props table: a GET form of the props that take a
//! switch, a condition, a text or a number, which re-renders the component with the chosen
//! ones and writes the matching `lui!` line. Rust cannot call a setter by its name at run
//! time, so each builder here has a small function mapping its props to its setters; props
//! that take a list, markup or several values (and builders made mostly of those) stay in the
//! table without a control. Query keys are `pg.<Builder>.<prop>`, so a playground's state is
//! in the URL like any other; the form is a swap root, so the script re-renders it in place.

use loco_ui::prelude::*;
use loco_ui::props::{Component, Prop, PropKind};

/// A builder the playground can build: its `lui!` call as written, the props it offers, what
/// follows the attributes (a body block or `;`), and the function that builds it.
pub(crate) struct Entry {
    pub builder: &'static str,
    call: &'static str,
    props: &'static [&'static str],
    rest: &'static str,
    build: fn(&Try) -> Markup,
}

/// The query as the playground reads it, for one builder.
pub(crate) struct Try<'a> {
    pub ui: &'a Ui,
    builder: &'a str,
}

impl<'a> Try<'a> {
    fn key(&self, prop: &str) -> String {
        format!("pg.{}.{prop}", self.builder)
    }
    /// A ticked switch or condition.
    fn on(&self, prop: &str) -> bool {
        matches!(self.ui.param(&self.key(prop)), Some("on" | "true"))
    }
    /// A text, when not empty.
    fn text(&self, prop: &str) -> Option<&'a str> {
        let key = self.key(prop);
        self.ui.param(&key).filter(|v| !v.trim().is_empty())
    }
    /// A number, when it parses.
    fn num<T: std::str::FromStr>(&self, prop: &str) -> Option<T> {
        self.text(prop).and_then(|v| v.trim().parse().ok())
    }
}

/// Apply `$setter` when the switch is on.
fn switch<B>(b: B, on: bool, f: impl FnOnce(B) -> B) -> B {
    if on { f(b) } else { b }
}

/// Apply `f` with the value when there is one.
fn with<B, V>(b: B, v: Option<V>, f: impl FnOnce(B, V) -> B) -> B {
    match v {
        Some(v) => f(b, v),
        None => b,
    }
}

pub(crate) const ENTRIES: &[Entry] = &[
    Entry {
        builder: "Button",
        call: "Button(\"Save\")",
        props: &[
            "primary", "danger", "ghost", "small", "disabled", "loading", "shimmer",
        ],
        rest: ";",
        build: |t| {
            let b = t.ui.button("Save");
            let b = switch(b, t.on("shimmer"), |b| b.shimmer());
            let b = switch(b, t.on("primary"), |b| b.primary());
            let b = switch(b, t.on("danger"), |b| b.danger());
            let b = switch(b, t.on("ghost"), |b| b.ghost());
            let b = switch(b, t.on("small"), |b| b.small());
            let b = switch(b, t.on("disabled"), |b| b.disabled());
            b.loading(t.on("loading")).render()
        },
    },
    Entry {
        builder: "Badge",
        call: "Badge(\"New\")",
        props: &["secondary", "danger", "outline", "ok", "warn", "shimmer"],
        rest: ";",
        build: |t| {
            let b = t.ui.badge("New");
            let b = switch(b, t.on("shimmer"), |b| b.shimmer());
            let b = switch(b, t.on("secondary"), |b| b.secondary());
            let b = switch(b, t.on("danger"), |b| b.danger());
            let b = switch(b, t.on("outline"), |b| b.outline());
            let b = switch(b, t.on("ok"), |b| b.ok());
            switch(b, t.on("warn"), |b| b.warn()).render()
        },
    },
    Entry {
        builder: "Alert",
        call: "Alert(\"Heads up\")",
        props: &["description", "danger", "warn", "ok"],
        rest: ";",
        build: |t| {
            let b = with(t.ui.alert("Heads up"), t.text("description"), |b, v| {
                b.description(v)
            });
            let b = switch(b, t.on("danger"), |b| b.danger());
            let b = switch(b, t.on("warn"), |b| b.warn());
            switch(b, t.on("ok"), |b| b.ok()).render()
        },
    },
    Entry {
        builder: "Card",
        call: "Card",
        props: &[
            "title",
            "description",
            "beam",
            "glow",
            "gradient_border",
            "reveal",
        ],
        rest: " { p { \"The card's body.\" } }",
        build: |t| {
            let b = with(t.ui.card(), t.text("title"), |b, v| b.title(v));
            let b = with(b, t.text("description"), |b, v| b.description(v));
            let b = switch(b, t.on("beam"), |b| b.beam());
            let b = switch(b, t.on("glow"), |b| b.glow());
            let b = switch(b, t.on("gradient_border"), |b| b.gradient_border());
            let b = switch(b, t.on("reveal"), |b| b.reveal());
            b.body(html! { p { "The card's body." } }).render()
        },
    },
    Entry {
        builder: "Avatar",
        call: "Avatar(\"Ada Lovelace\")",
        props: &["small", "large"],
        rest: ";",
        build: |t| {
            let b = switch(t.ui.avatar("Ada Lovelace"), t.on("small"), |b| b.small());
            switch(b, t.on("large"), |b| b.large()).render()
        },
    },
    Entry {
        builder: "Progress",
        call: "Progress(40, 100)",
        props: &["label"],
        rest: ";",
        build: |t| with(t.ui.progress(40, 100), t.text("label"), |b, v| b.label(v)).render(),
    },
    Entry {
        builder: "Meter",
        call: "Meter(83, 0, 100)",
        props: &["label", "low", "high", "optimum"],
        rest: ";",
        build: |t| {
            let b = with(t.ui.meter(83, 0, 100), t.text("label"), |b, v| b.label(v));
            let b = with(b, t.num("low"), |b, v| b.low(v));
            let b = with(b, t.num("high"), |b, v| b.high(v));
            with(b, t.num("optimum"), |b, v| b.optimum(v)).render()
        },
    },
    Entry {
        builder: "Separator",
        call: "Separator",
        props: &["label", "vertical"],
        rest: ";",
        build: |t| {
            let b = with(t.ui.separator(), t.text("label"), |b, v| b.label(v));
            switch(b, t.on("vertical"), |b| b.vertical()).render()
        },
    },
    Entry {
        builder: "Skeleton",
        call: "Skeleton(3)",
        props: &["heading", "label"],
        rest: ";",
        build: |t| {
            let b = switch(t.ui.skeleton(3), t.on("heading"), |b| b.heading());
            with(b, t.text("label"), |b, v| b.label(v)).render()
        },
    },
    Entry {
        builder: "EmptyState",
        call: "EmptyState(\"No projects yet\")",
        props: &["icon"],
        rest: ";",
        build: |t| {
            with(
                t.ui.empty_state("No projects yet"),
                t.text("icon"),
                |b, v| b.icon(v),
            )
            .render()
        },
    },
    Entry {
        builder: "Stat",
        call: "Stat(\"Revenue\", \"$48,210\")",
        props: &["delta", "description", "down_is_good", "reveal"],
        rest: ";",
        build: |t| {
            let b = with(t.ui.stat("Revenue", "$48,210"), t.text("delta"), |b, v| {
                b.delta(v)
            });
            let b = switch(b, t.on("reveal"), |b| b.reveal());
            let b = with(b, t.text("description"), |b, v| b.description(v));
            switch(b, t.on("down_is_good"), |b| b.down_is_good()).render()
        },
    },
    Entry {
        builder: "Chart",
        call: "Chart(\"Signups\")",
        props: &["description", "unit", "bar", "line", "sparkline"],
        rest: " { point \"Mon\" 12.0; point \"Tue\" 18.0; point \"Wed\" 9.0; }",
        build: |t| {
            let b =
                t.ui.chart("Signups")
                    .point("Mon", 12.0)
                    .point("Tue", 18.0)
                    .point("Wed", 9.0);
            let b = with(b, t.text("description"), |b, v| b.description(v));
            let b = with(b, t.text("unit"), |b, v| b.unit(v));
            let b = switch(b, t.on("bar"), |b| b.bar());
            let b = switch(b, t.on("line"), |b| b.line());
            switch(b, t.on("sparkline"), |b| b.sparkline()).render()
        },
    },
    Entry {
        builder: "Table",
        call: "Table(\"try\", \"\")",
        props: &["hide_search", "choose_columns", "loading"],
        rest: " { column \"name\" \"Name\"; column \"size\" \"Size\" numeric; rows ([(\"a.txt\", \"1 KB\"), (\"b.txt\", \"2 KB\")]); }",
        build: |t| {
            let b = t.ui.table("try", "").column("name", "Name");
            let b = b.column("size", "Size").numeric();
            let b = b.rows([("a.txt", "1 KB"), ("b.txt", "2 KB")]);
            let b = switch(b, t.on("hide_search"), |b| b.hide_search());
            let b = switch(b, t.on("choose_columns"), |b| b.choose_columns());
            b.loading(t.on("loading")).render()
        },
    },
    Entry {
        builder: "DescriptionList",
        call: "DescriptionList",
        props: &["stacked"],
        rest: " { item \"Plan\" \"Team\"; item \"Seats\" \"12\"; }",
        build: |t| {
            let b =
                t.ui.description_list()
                    .item("Plan", "Team")
                    .item("Seats", "12");
            switch(b, t.on("stacked"), |b| b.stacked()).render()
        },
    },
    Entry {
        builder: "InputOtp",
        call: "InputOtp(\"code\", \"Code\")",
        props: &["length"],
        rest: ";",
        build: |t| {
            with(t.ui.input_otp("code", "Code"), t.num("length"), |b, v| {
                b.length(v)
            })
            .render()
        },
    },
    Entry {
        builder: "Input",
        call: "Input(\"name\", \"Name\")",
        props: &[
            "email",
            "password",
            "required",
            "placeholder",
            "help",
            "error",
            "maxlength",
            "hide_label",
            "gradient_border",
        ],
        rest: ";",
        build: |t| {
            let b = t.ui.input("name", "Name");
            let b = switch(b, t.on("gradient_border"), |b| b.gradient_border());
            let b = switch(b, t.on("email"), |b| b.email());
            let b = switch(b, t.on("password"), |b| b.password());
            let b = switch(b, t.on("required"), |b| b.required());
            let b = with(b, t.text("placeholder"), |b, v| b.placeholder(v));
            let b = with(b, t.text("help"), |b, v| b.help(v));
            let b = with(b, t.text("error"), |b, v| b.error(v));
            let b = with(b, t.num("maxlength"), |b, v| b.maxlength(v));
            switch(b, t.on("hide_label"), |b| b.hide_label()).render()
        },
    },
    Entry {
        builder: "Marquee",
        call: "Marquee(\"Customers\")",
        props: &["reverse", "duration"],
        rest: " { text \"Acme\"; text \"Globex\"; text \"Initech\"; text \"Umbrella\"; }",
        build: |t| {
            let b = t.ui.marquee("Customers").text("Acme").text("Globex");
            let b = b.text("Initech").text("Umbrella");
            let b = switch(b, t.on("reverse"), |b| b.reverse());
            with(b, t.num("duration"), |b, v| b.duration(v)).render()
        },
    },
    Entry {
        builder: "Range",
        call: "Range(\"volume\", \"Volume\")",
        props: &["value", "min", "max", "step"],
        rest: ";",
        build: |t| {
            let b = with(t.ui.range("volume", "Volume"), t.num("value"), |b, v| {
                b.value(v)
            });
            let b = with(b, t.num("min"), |b, v| b.min(v));
            let b = with(b, t.num("max"), |b, v| b.max(v));
            with(b, t.num("step"), |b, v| b.step(v)).render()
        },
    },
    Entry {
        builder: "Form",
        call: "Form(\"/form\")",
        props: &["submit", "inline", "get"],
        rest: " { text \"nick\" \"Nickname\"; switch \"news\" \"Newsletter\"; }",
        build: |t| {
            let b = with(t.ui.form("/form"), t.text("submit"), |b, v| b.submit(v));
            let b = switch(b, t.on("inline"), |b| b.inline());
            let b = switch(b, t.on("get"), |b| b.get());
            let b = b.id("playground").text("nick", "Nickname");
            b.switch("news", "Newsletter").render()
        },
    },
    Entry {
        builder: "ErrorPage",
        call: "ErrorPage(404)",
        props: &["title", "description", "home"],
        rest: ";",
        build: |t| {
            let b = with(t.ui.error_page(404), t.text("title"), |b, v| b.title(v));
            let b = with(b, t.text("description"), |b, v| b.description(v));
            with(b, t.text("home"), |b, v| b.home(v)).render()
        },
    },
];

impl Entry {
    /// The props this entry has a control for.
    #[cfg(test)]
    pub(crate) fn offered(&self) -> &'static [&'static str] {
        self.props
    }
}

/// The playground entry for `builder`, if it has one.
pub(crate) fn entry(builder: &str) -> Option<&'static Entry> {
    ENTRIES.iter().find(|e| e.builder == builder)
}

/// Whether the query tries any of `builder`'s props.
pub(crate) fn tried(ui: &Ui, builder: &str) -> bool {
    entry(builder).is_some_and(|e| {
        e.props
            .iter()
            .any(|p| ui.param(&format!("pg.{builder}.{p}")).is_some())
    })
}

/// The `lui!` line for what the query chose.
fn snippet(t: &Try, e: &Entry, props: &[Prop]) -> String {
    let mut line = e.call.to_string();
    for p in props.iter().filter(|p| e.props.contains(&p.name)) {
        match p.kind {
            PropKind::Switch if t.on(p.name) => line += &format!(" {}", p.name),
            PropKind::Condition if t.on(p.name) => line += &format!(" {}=(true)", p.name),
            PropKind::Number => {
                if let Some(v) = t.text(p.name) {
                    line += &format!(" {}={}", p.name, v.trim());
                }
            }
            PropKind::Value => {
                if let Some(v) = t.text(p.name) {
                    line += &format!(" {}={:?}", p.name, v);
                }
            }
            _ => {}
        }
    }
    line + e.rest
}

/// The control for one prop in the table's "Try" column; empty for props it does not offer.
fn control(ui: &Ui, t: &Try, e: &Entry, p: &Prop) -> Markup {
    if !e.props.contains(&p.name) {
        return html! {};
    }
    let key = t.key(p.name);
    match p.kind {
        PropKind::Switch | PropKind::Condition => {
            html! { (ui.checkbox(&key, p.name).checked(t.on(p.name))) }
        }
        _ => html! { (ui.input(&key, p.name).hide_label().value(t.text(p.name).unwrap_or(""))) },
    }
}

/// The props table as a form, the preview and the `lui!` line, in one swap root.
pub(crate) fn playground(
    ui: &Ui,
    action: &str,
    c: &Component,
    table: impl Fn(&dyn Fn(&Prop) -> Markup) -> Markup,
) -> Markup {
    let Some(e) = entry(c.builder) else {
        return table(&|_| html! {});
    };
    let t = Try {
        ui,
        builder: c.builder,
    };
    let id = format!("pg-{}", c.builder.to_lowercase());
    html! {
        div id=(id) data-lui="swap" class="lui-playground" {
            form method="get" action=(action) {
                (table(&|p| control(ui, &t, e, p)))
                p { (ui.button("Try").primary().small()) " " a href=(action) { "Reset" } }
            }
            div class="lui-playground-preview" { ((e.build)(&t)) }
            pre tabindex="0" aria-label={ (c.builder) " in lui!" } { code { (snippet(&t, e, c.props)) } }
        }
    }
}
