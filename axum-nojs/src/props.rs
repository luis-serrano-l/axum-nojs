//! # Props
//!
//! What every builder accepts, as data. Each builder carries a `PROPS` constant listing its
//! setters: the name, what kind of setter it is, its arguments as written in Rust, the default
//! when it is not called, the HTML attribute it sets (if it maps to one) and the first sentence
//! of its documentation. A test reads the source and fails when a setter is missing from its
//! builder's `PROPS`, or listed with arguments that no longer match.
//!
//! ```rust
//! use axum_nojs::props::PropKind;
//! use axum_nojs::tabs::Tabs;
//! let vertical = Tabs::PROPS.iter().find(|p| p.name == "vertical").unwrap();
//! assert_eq!(vertical.kind, PropKind::Switch);
//! assert_eq!(vertical.default, "off");
//! let badge = Tabs::PROPS.iter().find(|p| p.name == "badge").unwrap();
//! assert_eq!(badge.kind, PropKind::Modifier);
//! ```

/// What a setter does to its builder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropKind {
    /// Sets a value: text, markup, a list given at once, or several arguments together.
    Value,
    /// Sets a number.
    Number,
    /// No argument: switches something on (`.required()`).
    Switch,
    /// Takes a `bool`, for a route that decides from a condition (`.open(..)`).
    Condition,
    /// Adds one item to the builder's list (`.tab(..)`, `.column(..)`).
    Item,
    /// Changes the item added last (`.badge(..)`, `.sortable()`).
    Modifier,
}

impl PropKind {
    /// The kind as one lowercase word, as in the spec JSON.
    pub const fn as_str(self) -> &'static str {
        match self {
            PropKind::Value => "value",
            PropKind::Number => "number",
            PropKind::Switch => "switch",
            PropKind::Condition => "condition",
            PropKind::Item => "item",
            PropKind::Modifier => "modifier",
        }
    }
}

/// One setter of a builder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Prop {
    /// The setter's name (`vertical` for `.vertical()`).
    pub name: &'static str,
    /// What it does to the builder.
    pub kind: PropKind,
    /// Its arguments after `self`, as written in the source; empty for a switch.
    pub args: &'static str,
    /// The value when the setter is not called: `off` for switches and conditions, empty when
    /// there is nothing to say (an unset value, an item).
    pub default: &'static str,
    /// The HTML attribute it sets, empty when it sets a class, an element or several things.
    pub attr: &'static str,
    /// The first sentence of the setter's documentation.
    pub doc: &'static str,
}

impl Prop {
    /// A setter `name` of `kind` taking `args`; switches and conditions default to `off`.
    pub const fn new(name: &'static str, kind: PropKind, args: &'static str) -> Self {
        let default = match kind {
            PropKind::Switch | PropKind::Condition => "off",
            _ => "",
        };
        Prop {
            name,
            kind,
            args,
            default,
            attr: "",
            doc: "",
        }
    }

    /// The value when the setter is not called.
    pub const fn default(mut self, default: &'static str) -> Self {
        self.default = default;
        self
    }

    /// The HTML attribute it sets.
    pub const fn attr(mut self, attr: &'static str) -> Self {
        self.attr = attr;
        self
    }

    /// One sentence on what it does.
    pub const fn doc(mut self, doc: &'static str) -> Self {
        self.doc = doc;
        self
    }
}
