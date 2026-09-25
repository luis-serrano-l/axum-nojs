//! # Props
//!
//! What every builder accepts, as data. Each builder carries a `PROPS` constant listing its
//! setters: the name, what kind of setter it is, its arguments as written in Rust, the default
//! when it is not called, the HTML attribute it sets (if it maps to one) and the first sentence
//! of its documentation. A test reads the source and fails when a setter is missing from its
//! builder's `PROPS`, or listed with arguments that no longer match.
//!
//! [`crate::props()`] lists every builder with its constructors and `PROPS`, which the spec JSON
//! and the demo's props tables are made from.
//!
//! ```rust
//! use loco_ui::props::PropKind;
//! use loco_ui::tabs::Tabs;
//! let vertical = Tabs::PROPS.iter().find(|p| p.name == "vertical").unwrap();
//! assert_eq!(vertical.kind, PropKind::Switch);
//! assert_eq!(vertical.default, "off");
//! let badge = Tabs::PROPS.iter().find(|p| p.name == "badge").unwrap();
//! assert_eq!(badge.kind, PropKind::Modifier);
//!
//! let tabs = loco_ui::props().iter().find(|c| c.builder == "Tabs").unwrap();
//! assert_eq!(tabs.calls, ["ui.tabs(name: &str)"]);
//! assert_eq!(tabs.lui(), "Tabs");
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

/// What a builder's API promises (README "Component status").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// Its constructor and setters change only in a breaking release, with a note.
    Stable,
    /// New: its setters may be renamed or reshaped in any release while it is tried.
    Beta,
}

impl Status {
    /// The status as one lowercase word, as in the spec JSON and on the demo.
    pub const fn as_str(self) -> &'static str {
        match self {
            Status::Stable => "stable",
            Status::Beta => "beta",
        }
    }
}

/// A builder and how to get one: its module, its constructors as written in Rust, its setters
/// and how settled its API is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Component {
    /// The file under `loco-ui/src/` without `.rs`, as in the spec.
    pub module: &'static str,
    /// The builder type (`Tabs`).
    pub builder: &'static str,
    /// Its constructors with their required arguments (`ui.tabs(name: &str)`).
    pub calls: &'static [&'static str],
    /// Its setters.
    pub props: &'static [Prop],
    /// Stable, or beta while it is new.
    pub status: Status,
}

impl Component {
    /// The name `lui!` knows it by: the first constructor's method in `UpperCamelCase`
    /// (`ui.date_picker(..)` is `DatePicker`); a type's own constructor keeps the type name.
    pub fn lui(&self) -> String {
        let call = self.calls.first().copied().unwrap_or(self.builder);
        let Some(method) = call.strip_prefix("ui.") else {
            return self.builder.to_string();
        };
        let method = &method[..method.find('(').unwrap_or(method.len())];
        method
            .split('_')
            .map(|w| {
                let mut c = w.chars();
                c.next().map_or(String::new(), |f| {
                    f.to_ascii_uppercase().to_string() + c.as_str()
                })
            })
            .collect()
    }
}

/// Every builder, by module then name.
pub(crate) const COMPONENTS: &[Component] = &[
    Component {
        module: "accordion",
        builder: "Accordion",
        calls: &["ui.accordion(group: &str)"],
        props: crate::accordion::Accordion::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "alert",
        builder: "Alert",
        calls: &["ui.alert(title: &str)"],
        props: crate::alert::Alert::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "avatar",
        builder: "Avatar",
        calls: &["ui.avatar(name: &str)"],
        props: crate::avatar::Avatar::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "badge",
        builder: "Badge",
        calls: &["ui.badge(text: &str)"],
        props: crate::badge::Badge::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "breadcrumbs",
        builder: "Breadcrumbs",
        calls: &["ui.breadcrumbs()"],
        props: crate::breadcrumbs::Breadcrumbs::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "button",
        builder: "Button",
        calls: &[
            "ui.button(text: &str)",
            "ui.link_button(text: &str, href: &str)",
        ],
        props: crate::button::Button::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "calendar",
        builder: "Calendar",
        calls: &["ui.calendar(name: &str)"],
        props: crate::calendar::Calendar::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "card",
        builder: "Card",
        calls: &["ui.card()"],
        props: crate::card::Card::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "chart",
        builder: "Chart",
        calls: &["ui.chart(title: &str)"],
        props: crate::chart::Chart::PROPS,
        status: Status::Beta,
    },
    Component {
        module: "cluster",
        builder: "Cluster",
        calls: &["ui.cluster(content: Markup)"],
        props: crate::cluster::Cluster::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "color",
        builder: "Color",
        calls: &["ui.color(name: &str, label: &str)"],
        props: crate::color::Color::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "combobox",
        builder: "Combobox",
        calls: &["ui.combobox(name: &str, action: &str)"],
        props: crate::combobox::Combobox::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "counter",
        builder: "Counter",
        calls: &["ui.counter(action: &str, value: i64)"],
        props: crate::counter::Counter::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "date_picker",
        builder: "DatePicker",
        calls: &["ui.date_picker(name: &str, label: &str)"],
        props: crate::date_picker::DatePicker::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "dialog",
        builder: "Dialog",
        calls: &["ui.dialog(trigger: &str)"],
        props: crate::dialog::Dialog::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "drawer",
        builder: "Drawer",
        calls: &["ui.drawer(label: &str)"],
        props: crate::drawer::Drawer::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "empty_state",
        builder: "EmptyState",
        calls: &["ui.empty_state(title: &str)"],
        props: crate::empty_state::EmptyState::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "error_summary",
        builder: "ErrorSummary",
        calls: &["ui.error_summary(errors: &[(&str, &str)])"],
        props: crate::error_summary::ErrorSummary::PROPS,
        status: Status::Beta,
    },
    Component {
        module: "flash",
        builder: "Flash",
        calls: &["ui.flash()"],
        props: crate::flash::Flash::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "form",
        builder: "Form",
        calls: &["ui.form(action: &str)", "ui.fields()"],
        props: crate::form::Form::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "grid",
        builder: "Grid",
        calls: &["ui.grid(min: &str, content: Markup)"],
        props: crate::grid::Grid::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "icon",
        builder: "IconMark",
        calls: &["ui.icon(icon: Icon)"],
        props: crate::icon::IconMark::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "input",
        builder: "Input",
        calls: &[
            "ui.input(name: &str, label: &str)",
            "ui.checkbox(name: &str, label: &str)",
            "ui.switch(name: &str, label: &str)",
        ],
        props: crate::input::Input::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "input",
        builder: "RadioGroup",
        calls: &["ui.radio_group(name: &str, legend: &str)"],
        props: crate::input::RadioGroup::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "kanban",
        builder: "Kanban",
        calls: &["ui.kanban(action: &str)"],
        props: crate::kanban::Kanban::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "marquee",
        builder: "Marquee",
        calls: &["ui.marquee(label: &str)"],
        props: crate::marquee::Marquee::PROPS,
        status: Status::Beta,
    },
    Component {
        module: "meter",
        builder: "Meter",
        calls: &["ui.meter(value: i64, min: i64, max: i64)"],
        props: crate::meter::Meter::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "pager",
        builder: "Pager",
        calls: &["ui.pager(href: &str, total: usize)"],
        props: crate::pager::Pager::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "palette",
        builder: "Palette",
        calls: &["ui.palette(action: &str)"],
        props: crate::palette::Palette::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "popover",
        builder: "Menu",
        calls: &["ui.menu(label: &str)"],
        props: crate::popover::Menu::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "progress",
        builder: "Progress",
        calls: &["ui.progress(value: u64, max: u64)"],
        props: crate::progress::Progress::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "range",
        builder: "Range",
        calls: &[
            "ui.range(name: &str, label: &str)",
            "ui.range_pair(name: &str, label: &str)",
        ],
        props: crate::range::Range::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "select",
        builder: "Select",
        calls: &["ui.select(name: &str, label: &str)"],
        props: crate::select::Select::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "select",
        builder: "SelectOption",
        calls: &["SelectOption::new(value: &str, text: &str)"],
        props: crate::select::SelectOption::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "separator",
        builder: "Separator",
        calls: &["ui.separator()"],
        props: crate::separator::Separator::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "skeleton",
        builder: "Skeleton",
        calls: &["ui.skeleton(lines: usize)"],
        props: crate::skeleton::Skeleton::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "split",
        builder: "Split",
        calls: &["ui.split(side: Markup, main: Markup)"],
        props: crate::split::Split::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "stack",
        builder: "Stack",
        calls: &["ui.stack(content: Markup)"],
        props: crate::stack::Stack::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "stat",
        builder: "Stat",
        calls: &["ui.stat(label: &str, value: &str)"],
        props: crate::stat::Stat::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "table",
        builder: "Row",
        calls: &["Row::new(cells: impl IntoIterator<Item = Markup>)"],
        props: crate::table::Row::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "table",
        builder: "Table",
        calls: &["ui.table(id: &str, href: &str)"],
        props: crate::table::Table::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "tabs",
        builder: "Tabs",
        calls: &["ui.tabs(name: &str)"],
        props: crate::tabs::Tabs::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "theme",
        builder: "ThemeToggle",
        calls: &["ui.theme_toggle(action: &str)"],
        props: &[],
        status: Status::Stable,
    },
    Component {
        module: "toast",
        builder: "Toasts",
        calls: &["ui.toasts()"],
        props: crate::toast::Toasts::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "tooltip",
        builder: "Tooltip",
        calls: &["ui.tooltip(text: &str, trigger: Markup)"],
        props: crate::tooltip::Tooltip::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "upload",
        builder: "Upload",
        calls: &["ui.upload(action: &str, name: &str)"],
        props: crate::upload::Upload::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "wizard",
        builder: "Wizard",
        calls: &["ui.wizard(id: &str, action: &str)"],
        props: crate::wizard::Wizard::PROPS,
        status: Status::Stable,
    },
    Component {
        module: "blocks/app_shell",
        builder: "AppShell",
        calls: &["ui.app_shell(name: &str)"],
        props: crate::blocks::app_shell::AppShell::PROPS,
        status: Status::Beta,
    },
    Component {
        module: "blocks/auth_page",
        builder: "AuthPage",
        calls: &["ui.auth_page(title: &str)"],
        props: crate::blocks::auth_page::AuthPage::PROPS,
        status: Status::Beta,
    },
    Component {
        module: "blocks/dashboard_page",
        builder: "DashboardPage",
        calls: &["ui.dashboard_page(title: &str)"],
        props: crate::blocks::dashboard_page::DashboardPage::PROPS,
        status: Status::Beta,
    },
    Component {
        module: "blocks/error_page",
        builder: "ErrorPage",
        calls: &["ui.error_page(status: u16)"],
        props: crate::blocks::error_page::ErrorPage::PROPS,
        status: Status::Beta,
    },
    Component {
        module: "blocks/record_page",
        builder: "RecordPage",
        calls: &["ui.record_page(title: &str)"],
        props: crate::blocks::record_page::RecordPage::PROPS,
        status: Status::Beta,
    },
    Component {
        module: "blocks/settings_page",
        builder: "SettingsPage",
        calls: &["ui.settings_page(title: &str)"],
        props: crate::blocks::settings_page::SettingsPage::PROPS,
        status: Status::Beta,
    },
    Component {
        module: "context_menu",
        builder: "ContextMenu",
        calls: &["ui.context_menu(label: &str)"],
        props: crate::context_menu::ContextMenu::PROPS,
        status: Status::Beta,
    },
    Component {
        module: "description_list",
        builder: "DescriptionList",
        calls: &["ui.description_list()"],
        props: crate::description_list::DescriptionList::PROPS,
        status: Status::Beta,
    },
    Component {
        module: "input_otp",
        builder: "InputOtp",
        calls: &["ui.input_otp(name: &str, label: &str)"],
        props: crate::input_otp::InputOtp::PROPS,
        status: Status::Beta,
    },
    Component {
        module: "nav_menu",
        builder: "NavMenu",
        calls: &["ui.nav_menu(label: &str)"],
        props: crate::nav_menu::NavMenu::PROPS,
        status: Status::Beta,
    },
    Component {
        module: "sidebar",
        builder: "Sidebar",
        calls: &["ui.sidebar(label: &str)"],
        props: crate::sidebar::Sidebar::PROPS,
        status: Status::Beta,
    },
    Component {
        module: "toggle_group",
        builder: "ToggleGroup",
        calls: &["ui.toggle_group(name: &str, label: &str)"],
        props: crate::toggle_group::ToggleGroup::PROPS,
        status: Status::Beta,
    },
];
