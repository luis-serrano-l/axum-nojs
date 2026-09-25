//! # Languages
//!
//! Every word a component writes by itself ("Next", "Close", "Load more", "Page 2 of 9",
//! month names) comes from one table, [`Strings`], chosen per request: `ui.lang()` is the
//! visitor's language and `<html lang>` says it. English is built in; an app adds its own
//! tables and registers them once with [`languages`]. The language is the `lui-lang` cookie
//! when it names a registered one, else the first `Accept-Language` entry that does (by
//! exact tag, then by primary subtag: `de-AT` finds `de`), else the first registered table.
//!
//! ```rust
//! use loco_ui::i18n::{self, Strings, Text};
//! use loco_ui::prelude::*;
//!
//! static SPANISH: Strings = Strings::new("es")
//!     .with(Text::Next, "Siguiente")
//!     .with(Text::Previous, "Anterior")
//!     .with(Text::LoadMore, "Cargar más");
//! static LANGUAGES: [&Strings; 2] = [&Strings::ENGLISH, &SPANISH];
//! i18n::languages(&LANGUAGES); // once, at start-up
//!
//! let ui = Ui::from_request("/", "", "").accept_language("es-MX,es;q=0.9,en;q=0.5");
//! assert_eq!(ui.lang(), "es");
//! assert_eq!(ui.text(Text::Next), "Siguiente");
//! assert_eq!(ui.text(Text::Close), "Close"); // not translated: English
//! assert!(ui.page("Hola", html! {}).into_string().contains(r#"<html lang="es""#));
//! // The cookie wins over the header.
//! let ui = Ui::from_request("/", "", "lui-lang=en").accept_language("es");
//! assert_eq!(ui.lang(), "en");
//! ```
//!
//! A table can also be filled from any lookup, such as Loco's `fluent-templates` loader
//! (`LOCALES.try_lookup(&lang, key)`): [`Strings::from_lookup`] asks for each [`Text::key`]
//! (`lui-next`, `lui-load-more`, …) and keeps English for what the lookup does not have.

use std::fmt::Display;
use std::sync::{Mutex, OnceLock};

/// Every string a component writes by itself. `{}` in a text is filled in order by
/// [`Strings::fill`]; `{0}`, `{1}` by position, for a language that orders them otherwise.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Text {
    /// "Next": wizard, pager.
    Next,
    /// "Previous": pager.
    Previous,
    /// "Back": wizard.
    Back,
    /// "Skip": wizard.
    Skip,
    /// "Finish": wizard.
    Finish,
    /// "Close": dialog, drawer.
    Close,
    /// "Cancel": dialog, table edit.
    Cancel,
    /// "Dismiss": flash, toast.
    Dismiss,
    /// "Dismiss: {}": the dismiss link's name, with the message.
    DismissMessage,
    /// "Submit": a form's button when none is named.
    Submit,
    /// "Search": combobox, palette.
    Search,
    /// "Type to search…": combobox.
    TypeToSearch,
    /// "No matches.": combobox.
    NoMatches,
    /// "Create": combobox.
    Create,
    /// "Create “{}”": combobox, with what was typed.
    CreateValue,
    /// "Remove {}": combobox chip, upload file.
    RemoveValue,
    /// "Selected": combobox chips.
    Selected,
    /// "Results": combobox list.
    Results,
    /// "Load more": pager.
    LoadMore,
    /// "Showing {} of {}": pager.
    ShowingOf,
    /// "Loading": skeleton.
    Loading,
    /// "Pages": the table pager's `<nav>`.
    Pages,
    /// "Page": before the page number box.
    Page,
    /// "of {}": after the page number box.
    OfTotal,
    /// "{}–{} of {}": the rows shown.
    RangeOf,
    /// "Go": the page jump and tab select buttons.
    Go,
    /// "Rows per page": table.
    RowsPerPage,
    /// "Show": the rows-per-page button.
    Show,
    /// "No rows match.": table.
    NoRows,
    /// "Columns": table column chooser.
    Columns,
    /// "Edit": table row, wizard summary.
    Edit,
    /// "Save": table row edit.
    Save,
    /// "Expand all": accordion.
    ExpandAll,
    /// "Collapse all": accordion.
    CollapseAll,
    /// "Breadcrumb": the breadcrumbs' `<nav>`.
    Breadcrumb,
    /// "Show {} more": collapsed breadcrumbs.
    ShowMore,
    /// "Previous month": calendar.
    PreviousMonth,
    /// "Next month": calendar.
    NextMonth,
    /// "{0}, {1} {2} {3}": a day in full (weekday, day, month, year).
    LongDate,
    /// "Pick a date": date picker.
    PickDate,
    /// "Opacity {}": colour picker.
    Opacity,
    /// "Presets": colour picker.
    Presets,
    /// "Use {}": a colour preset.
    UseValue,
    /// "Reset": counter.
    Reset,
    /// "Value": counter.
    Value,
    /// "Set": counter.
    Set,
    /// "{} to {}": counter bounds.
    FromTo,
    /// "at least {}": counter bound.
    AtLeast,
    /// "at most {}": counter bound.
    AtMost,
    /// ", in steps of {}": counter step.
    InStepsOf,
    /// "No cards": kanban column.
    NoCards,
    /// ", over the limit": kanban column count.
    OverLimit,
    /// "Move {} to {}": kanban card.
    MoveTo,
    /// "Nothing by that name. Try one word, or pick from the list.": palette.
    NothingByThatName,
    /// "Type a command or a page": palette.
    TypeCommand,
    /// "Minimum": range.
    Minimum,
    /// "Maximum": range.
    Maximum,
    /// "Filter options": select.
    FilterOptions,
    /// "Filter": select.
    Filter,
    /// "Tab": tabs select.
    Tab,
    /// "Choose files or drop them on the button": upload.
    ChooseFiles,
    /// "Choose a file or drop it on the button": upload.
    ChooseFile,
    /// "Upload": upload.
    Upload,
    /// "Uploaded files": upload.
    UploadedFiles,
    /// "Picked up where you left off, at step {}.": wizard.
    Resumed,
    /// "Start over": wizard.
    StartOver,
    /// " (has errors)": wizard step.
    HasErrors,
    /// "Progress": wizard.
    Progress,
    /// "{} of {} steps done": wizard.
    StepsDone,
    /// "Step {}": wizard.
    Step,
    /// "(optional)": wizard step.
    Optional,
    /// "There is a problem": error summary.
    Problem,
    /// "Delete": record page.
    Delete,
    /// "Sign out": app shell.
    SignOut,
    /// "Page not found": 404 page.
    NotFound,
    /// "The page you asked for is not here. It may have moved, or the link was mistyped.": 404 page.
    NotFoundMessage,
    /// "Something went wrong": 500 page.
    ServerError,
    /// "The server could not answer this time. Try again in a moment.": 500 page.
    ServerErrorMessage,
    /// "Go home": error pages.
    GoHome,
    /// "(skipped)": wizard review.
    Skipped,
    /// "Edit {}": wizard review: the edit link's name.
    EditValue,
    /// "Step {} of {}": wizard.
    StepOf,
    /// "{0} {1} {2}": a day on a button (day, month, year).
    DayMonthYear,
    /// "{0} {1}": the calendar's title (month, year).
    MonthYear,
    /// "First": table pager.
    First,
    /// "Last": table pager.
    Last,
    /// "Filter rows": table filter label.
    FilterRows,
    /// "Filter rows…": table filter placeholder.
    FilterRowsHint,
    /// "Clear": table filter.
    Clear,
    /// "Download CSV": table.
    DownloadCsv,
    /// "Select": table: the select-all box.
    Select,
    /// "Select {}": table: a row's box.
    SelectRow,
    /// "Actions": table: the actions column.
    Actions,
    /// "Row actions": table: a row's menu.
    RowActions,
    /// "With the selected rows:": table bulk bar.
    WithSelected,
    /// "1 match": combobox, palette.
    OneMatch,
    /// "{} matches": combobox, palette.
    Matches,
    /// "{} for “{}”": palette: the count and the query.
    ForQuery,
    /// " selected": combobox: after a picked result.
    IsSelected,
    /// "decrement": counter.
    Decrement,
    /// "increment": counter.
    Increment,
    /// "January": calendar.
    January,
    /// "February": calendar.
    February,
    /// "March": calendar.
    March,
    /// "April": calendar.
    April,
    /// "May": calendar.
    May,
    /// "June": calendar.
    June,
    /// "July": calendar.
    July,
    /// "August": calendar.
    August,
    /// "September": calendar.
    September,
    /// "October": calendar.
    October,
    /// "November": calendar.
    November,
    /// "December": calendar.
    December,
    /// "Monday": calendar.
    Monday,
    /// "Tuesday": calendar.
    Tuesday,
    /// "Wednesday": calendar.
    Wednesday,
    /// "Thursday": calendar.
    Thursday,
    /// "Friday": calendar.
    Friday,
    /// "Saturday": calendar.
    Saturday,
    /// "Sunday": calendar.
    Sunday,
}

/// How many texts there are.
const N: usize = Text::ALL.len();

impl Text {
    /// Every text, in table order.
    pub const ALL: [Text; 120] = [
        Text::Next,
        Text::Previous,
        Text::Back,
        Text::Skip,
        Text::Finish,
        Text::Close,
        Text::Cancel,
        Text::Dismiss,
        Text::DismissMessage,
        Text::Submit,
        Text::Search,
        Text::TypeToSearch,
        Text::NoMatches,
        Text::Create,
        Text::CreateValue,
        Text::RemoveValue,
        Text::Selected,
        Text::Results,
        Text::LoadMore,
        Text::ShowingOf,
        Text::Loading,
        Text::Pages,
        Text::Page,
        Text::OfTotal,
        Text::RangeOf,
        Text::Go,
        Text::RowsPerPage,
        Text::Show,
        Text::NoRows,
        Text::Columns,
        Text::Edit,
        Text::Save,
        Text::ExpandAll,
        Text::CollapseAll,
        Text::Breadcrumb,
        Text::ShowMore,
        Text::PreviousMonth,
        Text::NextMonth,
        Text::LongDate,
        Text::PickDate,
        Text::Opacity,
        Text::Presets,
        Text::UseValue,
        Text::Reset,
        Text::Value,
        Text::Set,
        Text::FromTo,
        Text::AtLeast,
        Text::AtMost,
        Text::InStepsOf,
        Text::NoCards,
        Text::OverLimit,
        Text::MoveTo,
        Text::NothingByThatName,
        Text::TypeCommand,
        Text::Minimum,
        Text::Maximum,
        Text::FilterOptions,
        Text::Filter,
        Text::Tab,
        Text::ChooseFiles,
        Text::ChooseFile,
        Text::Upload,
        Text::UploadedFiles,
        Text::Resumed,
        Text::StartOver,
        Text::HasErrors,
        Text::Progress,
        Text::StepsDone,
        Text::Step,
        Text::Optional,
        Text::Problem,
        Text::Delete,
        Text::SignOut,
        Text::NotFound,
        Text::NotFoundMessage,
        Text::ServerError,
        Text::ServerErrorMessage,
        Text::GoHome,
        Text::Skipped,
        Text::EditValue,
        Text::StepOf,
        Text::DayMonthYear,
        Text::MonthYear,
        Text::First,
        Text::Last,
        Text::FilterRows,
        Text::FilterRowsHint,
        Text::Clear,
        Text::DownloadCsv,
        Text::Select,
        Text::SelectRow,
        Text::Actions,
        Text::RowActions,
        Text::WithSelected,
        Text::OneMatch,
        Text::Matches,
        Text::ForQuery,
        Text::IsSelected,
        Text::Decrement,
        Text::Increment,
        Text::January,
        Text::February,
        Text::March,
        Text::April,
        Text::May,
        Text::June,
        Text::July,
        Text::August,
        Text::September,
        Text::October,
        Text::November,
        Text::December,
        Text::Monday,
        Text::Tuesday,
        Text::Wednesday,
        Text::Thursday,
        Text::Friday,
        Text::Saturday,
        Text::Sunday,
    ];

    /// The month `n`, 1 to 12.
    pub const fn month(n: u32) -> Text {
        Text::ALL[Text::January as usize + (n as usize - 1) % 12]
    }

    /// The weekday `n`, 0 for Monday to 6 for Sunday.
    pub const fn weekday(n: u32) -> Text {
        Text::ALL[Text::Monday as usize + n as usize % 7]
    }

    /// The key a lookup is asked for: `lui-` and the name in kebab case (`lui-load-more`).
    pub fn key(self) -> String {
        let name = format!("{self:?}");
        let mut key = String::from("lui");
        for c in name.chars() {
            if c.is_ascii_uppercase() {
                key.push('-');
            }
            key.push(c.to_ascii_lowercase());
        }
        key
    }
}

/// The English table, in [`Text::ALL`] order.
const ENGLISH: [&str; N] = [
    "Next",
    "Previous",
    "Back",
    "Skip",
    "Finish",
    "Close",
    "Cancel",
    "Dismiss",
    "Dismiss: {}",
    "Submit",
    "Search",
    "Type to search\u{2026}",
    "No matches.",
    "Create",
    "Create \u{201c}{}\u{201d}",
    "Remove {}",
    "Selected",
    "Results",
    "Load more",
    "Showing {} of {}",
    "Loading",
    "Pages",
    "Page",
    "of {}",
    "{}\u{2013}{} of {}",
    "Go",
    "Rows per page",
    "Show",
    "No rows match.",
    "Columns",
    "Edit",
    "Save",
    "Expand all",
    "Collapse all",
    "Breadcrumb",
    "Show {} more",
    "Previous month",
    "Next month",
    "{0}, {1} {2} {3}",
    "Pick a date",
    "Opacity {}",
    "Presets",
    "Use {}",
    "Reset",
    "Value",
    "Set",
    "{} to {}",
    "at least {}",
    "at most {}",
    ", in steps of {}",
    "No cards",
    ", over the limit",
    "Move {} to {}",
    "Nothing by that name. Try one word, or pick from the list.",
    "Type a command or a page",
    "Minimum",
    "Maximum",
    "Filter options",
    "Filter",
    "Tab",
    "Choose files or drop them on the button",
    "Choose a file or drop it on the button",
    "Upload",
    "Uploaded files",
    "Picked up where you left off, at step {}.",
    "Start over",
    " (has errors)",
    "Progress",
    "{} of {} steps done",
    "Step {}",
    "(optional)",
    "There is a problem",
    "Delete",
    "Sign out",
    "Page not found",
    "The page you asked for is not here. It may have moved, or the link was mistyped.",
    "Something went wrong",
    "The server could not answer this time. Try again in a moment.",
    "Go home",
    "(skipped)",
    "Edit {}",
    "Step {} of {}",
    "{0} {1} {2}",
    "{0} {1}",
    "First",
    "Last",
    "Filter rows",
    "Filter rows\u{2026}",
    "Clear",
    "Download CSV",
    "Select",
    "Select {}",
    "Actions",
    "Row actions",
    "With the selected rows:",
    "1 match",
    "{} matches",
    "{} for \u{201c}{}\u{201d}",
    " selected",
    "decrement",
    "increment",
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];

/// One language's texts. Start from [`Strings::new`] (English with another tag) and set
/// what you translate with [`Strings::with`], in a `static`; or fill one at run time with
/// [`Strings::from_lookup`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Strings {
    lang: &'static str,
    texts: [&'static str; N],
}

impl Default for &'static Strings {
    fn default() -> Self {
        &Strings::ENGLISH
    }
}

/// Where a text sits in the table: `ALL` is in declaration order.
const fn slot(text: Text) -> usize {
    text as usize
}

impl Strings {
    /// The built-in table, `lang="en"`.
    pub const ENGLISH: Strings = Strings {
        lang: "en",
        texts: ENGLISH,
    };

    /// English texts under the language tag `lang` (`"es"`, `"pt-BR"`), to translate with
    /// [`Strings::with`].
    pub const fn new(lang: &'static str) -> Strings {
        Strings {
            lang,
            texts: ENGLISH,
        }
    }

    /// The same table with `text` translated.
    pub const fn with(mut self, text: Text, translated: &'static str) -> Strings {
        self.texts[slot(text)] = translated;
        self
    }

    /// The language tag, for `<html lang>`.
    pub const fn lang(&self) -> &'static str {
        self.lang
    }

    /// The text as written in the table, placeholders and all.
    pub const fn get(&self, text: Text) -> &'static str {
        self.texts[slot(text)]
    }

    /// The text with its placeholders filled: `{}` by the next argument, `{0}`, `{1}` by
    /// position.
    ///
    /// ```rust
    /// use loco_ui::i18n::{Strings, Text};
    /// assert_eq!(Strings::ENGLISH.fill(Text::RangeOf, &[&11, &20, &42]), "11–20 of 42");
    /// assert_eq!(Strings::ENGLISH.fill(Text::LongDate, &[&"Thursday", &24, &"September", &2026]),
    ///            "Thursday, 24 September 2026");
    /// ```
    pub fn fill(&self, text: Text, args: &[&dyn Display]) -> String {
        let template = self.get(text);
        let mut out = String::with_capacity(template.len() + 16);
        let mut next = 0;
        let mut rest = template;
        while let Some(open) = rest.find('{') {
            out.push_str(&rest[..open]);
            let Some(close) = rest[open..].find('}') else {
                out.push_str(&rest[open..]);
                return out;
            };
            let inside = &rest[open + 1..open + close];
            let index = if inside.is_empty() {
                next += 1;
                Some(next - 1)
            } else {
                inside.parse().ok()
            };
            match index.and_then(|i: usize| args.get(i)) {
                Some(arg) => out.push_str(&arg.to_string()),
                None => out.push_str(&rest[open..=open + close]),
            }
            rest = &rest[open + close + 1..];
        }
        out.push_str(rest);
        out
    }

    /// A table for `lang` filled from `lookup`, asked for each [`Text::key`]; English where
    /// it answers `None`. Built once per `lang` and kept for the life of the process, so the
    /// same `lang` always gives the first table built.
    ///
    /// With Loco's `fluent-templates` (keys like `lui-load-more = Cargar más` in
    /// `assets/i18n/es/main.ftl`):
    /// `Strings::from_lookup("es", |key| LOCALES.try_lookup(&langid!("es"), key))`.
    ///
    /// ```rust
    /// use loco_ui::i18n::{Strings, Text};
    /// let ftl = [("lui-load-more", "Mehr laden"), ("lui-next", "Weiter")];
    /// let german = Strings::from_lookup("de", |key| {
    ///     ftl.iter().find(|(k, _)| *k == key).map(|(_, v)| v.to_string())
    /// });
    /// assert_eq!(german.get(Text::LoadMore), "Mehr laden");
    /// assert_eq!(german.get(Text::Close), "Close");
    /// ```
    pub fn from_lookup(lang: &str, lookup: impl Fn(&str) -> Option<String>) -> &'static Strings {
        static BUILT: OnceLock<Mutex<Vec<&'static Strings>>> = OnceLock::new();
        let built = BUILT.get_or_init(|| Mutex::new(Vec::new()));
        let mut built = built.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(s) = built.iter().find(|s| s.lang == lang) {
            return s;
        }
        let mut strings = Strings::new(Box::leak(lang.to_string().into_boxed_str()));
        for text in Text::ALL {
            if let Some(found) = lookup(&text.key()) {
                strings.texts[slot(text)] = Box::leak(found.into_boxed_str());
            }
        }
        let strings: &'static Strings = Box::leak(Box::new(strings));
        built.push(strings);
        strings
    }
}

impl crate::Redirect {
    /// Remember the language `tag` for this visitor (the `lui-lang` cookie, a year), for a
    /// language switch. A tag no table is registered for is ignored when read.
    pub fn lang(self, tag: &str) -> Self {
        let tag: String = tag
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
            .collect();
        self.cookie(format!(
            "{LANG_COOKIE}={tag}; Path=/; Max-Age=31536000; SameSite=Lax"
        ))
    }
}

static LANGUAGES: OnceLock<Mutex<&'static [&'static Strings]>> = OnceLock::new();

/// The tables an app offers, the first being the default. English only until this is
/// called; call it at start-up (a later call replaces the list).
pub fn languages(tables: &'static [&'static Strings]) {
    let list = LANGUAGES.get_or_init(|| Mutex::new(&[&Strings::ENGLISH]));
    *list.lock().unwrap_or_else(|e| e.into_inner()) = tables;
}

fn registered() -> &'static [&'static Strings] {
    LANGUAGES.get().map_or(&[&Strings::ENGLISH], |l| {
        *l.lock().unwrap_or_else(|e| e.into_inner())
    })
}

/// The cookie that picks a language over `Accept-Language`.
pub const LANG_COOKIE: &str = "lui-lang";

/// The table in `list` for `tag`: exact match, then by primary subtag.
fn find(list: &[&'static Strings], tag: &str) -> Option<&'static Strings> {
    let tag = tag.trim();
    let primary = |t: &str| t.split('-').next().unwrap_or("").to_ascii_lowercase();
    list.iter()
        .find(|s| s.lang.eq_ignore_ascii_case(tag))
        .or_else(|| list.iter().find(|s| primary(s.lang) == primary(tag)))
        .copied()
}

/// The table for a request: the `lui-lang` cookie, then `Accept-Language` (by `q`), then
/// the first registered table.
pub(crate) fn choose(cookie: Option<&str>, accept_language: &str) -> &'static Strings {
    choose_in(registered(), cookie, accept_language)
}

fn choose_in(
    list: &[&'static Strings],
    cookie: Option<&str>,
    accept_language: &str,
) -> &'static Strings {
    if let Some(s) = cookie.and_then(|c| find(list, c)) {
        return s;
    }
    let mut wanted: Vec<(&str, f32)> = accept_language
        .split(',')
        .filter_map(|part| {
            let mut bits = part.split(';');
            let tag = bits.next()?.trim();
            let q = bits
                .find_map(|b| b.trim().strip_prefix("q="))
                .and_then(|q| q.parse().ok())
                .unwrap_or(1.0);
            (!tag.is_empty() && tag != "*").then_some((tag, q))
        })
        .collect();
    wanted.sort_by(|a, b| b.1.total_cmp(&a.1));
    wanted
        .iter()
        .find_map(|(tag, _)| find(list, tag))
        .unwrap_or(list[0])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_kebab_case_and_the_table_is_complete() {
        for (i, t) in Text::ALL.iter().enumerate() {
            assert_eq!(*t as usize, i, "{t:?} is out of place in Text::ALL");
        }
        assert_eq!(Text::LoadMore.key(), "lui-load-more");
        assert_eq!(Text::Next.key(), "lui-next");
        assert_eq!(Strings::ENGLISH.get(Text::Sunday), "Sunday");
        assert_eq!(Strings::ENGLISH.get(Text::month(9)), "September");
        assert_eq!(Strings::ENGLISH.get(Text::weekday(3)), "Thursday");
        assert_eq!(Strings::ENGLISH.get(Text::Problem), "There is a problem");
    }

    #[test]
    fn accept_language_by_quality_then_primary_subtag() {
        static DE: Strings = Strings::new("de").with(Text::Next, "Weiter");
        static PT: Strings = Strings::new("pt-BR");
        let list: &[&Strings] = &[&Strings::ENGLISH, &DE, &PT];
        let lang = |cookie, header| choose_in(list, cookie, header).lang();
        assert_eq!(lang(None, "fr, de-AT;q=0.8, en;q=0.5"), "de");
        assert_eq!(lang(None, "en;q=0.4, pt-br"), "pt-BR");
        assert_eq!(lang(None, "fr"), "en");
        assert_eq!(lang(Some("de"), "pt-BR"), "de");
        assert_eq!(lang(Some("xx"), ""), "en");
    }

    #[test]
    fn fill_leaves_unknown_placeholders() {
        assert_eq!(
            Strings::ENGLISH.fill(Text::MoveTo, &[&"Card"]),
            "Move Card to {}"
        );
    }
}
