//! `cargo lui install`: set up a Loco app for loco-ui in one command.
//!
//! ```sh
//! cargo install --git https://github.com/luis-serrano-l/loco-ui loco-ui --bin cargo-lui
//! cargo lui install                  # in the app's directory
//! cargo lui install path/to/app      # or name it
//! # from a loco-ui checkout, without installing:
//! cargo run -p loco-ui -- install path/to/app --dep-path "$PWD/loco-ui"
//! ```
//!
//! It writes, and prints one line per step:
//! - `.loco-templates/scaffold/api/{controller,dto}.t`, the scaffold that makes
//!   `cargo loco generate scaffold` write Maud views and HTML controllers;
//! - the initializer line in `src/app.rs` (`Box::new(loco_ui::loco::Initializer)`);
//! - `src/views/layout.rs`, a page with a header, and `pub mod layout;` in `src/views/mod.rs`;
//! - `loco-ui` (git, or `--dep-path`) and `maud` under `[dependencies]` in `Cargo.toml`.
//!
//! Idempotent: a second run changes nothing. A file that exists with other content is left
//! alone and reported (`--force` overwrites the templates and the layout). Standard library
//! only, so it builds without the crate's features.

use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

const CONTROLLER: &str = include_str!("../../loco-templates/scaffold/api/controller.t");
const DTO: &str = include_str!("../../loco-templates/scaffold/api/dto.t");
const INITIALIZER: &str = "Box::new(loco_ui::loco::Initializer)";
const GIT: &str = "https://github.com/luis-serrano-l/loco-ui";

const LAYOUT: &str = r#"//! The page every view renders into: a header with a home link (add yours), then the view.
//! Written by `cargo lui install`; edit freely.

use loco_ui::prelude::*;

/// `body` inside the app's header, as a full page titled `title`.
pub fn page(ui: &Ui, title: &str, body: Markup) -> Page {
    ui.page(
        title,
        html! {
            header {
                (ui.cluster(html! {
                    a href="/" { strong { "Home" } }
                }))
            }
            main { (body) }
        },
    )
}
"#;

const USAGE: &str = "usage: cargo lui install [APP_DIR] [--dep-path PATH] [--force]";

fn main() -> ExitCode {
    // `cargo lui ..` runs this binary as `cargo-lui lui ..`.
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("lui") {
        args.remove(0);
    }
    if args.first().map(String::as_str) != Some("install") {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    }
    let mut opts = Options {
        app: PathBuf::from("."),
        dep_path: None,
        force: false,
    };
    let mut rest = args.into_iter().skip(1);
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--force" => opts.force = true,
            "--dep-path" => match rest.next() {
                Some(p) => opts.dep_path = Some(p),
                None => {
                    eprintln!("{USAGE}");
                    return ExitCode::from(2);
                }
            },
            s if s.starts_with('-') => {
                eprintln!("unknown option {s}\n{USAGE}");
                return ExitCode::from(2);
            }
            _ => opts.app = PathBuf::from(arg),
        }
    }
    match install(&opts) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

struct Options {
    app: PathBuf,
    dep_path: Option<String>,
    force: bool,
}

fn install(o: &Options) -> Result<(), String> {
    let app = &o.app;
    if !app.join("src/app.rs").is_file() || !app.join("Cargo.toml").is_file() {
        return Err(format!(
            "{} is not a Loco app (no Cargo.toml and src/app.rs)",
            app.display()
        ));
    }
    let templates = app.join(".loco-templates/scaffold/api");
    write_file(&templates.join("controller.t"), CONTROLLER, o.force)?;
    write_file(&templates.join("dto.t"), DTO, o.force)?;
    write_file(&app.join("src/views/layout.rs"), LAYOUT, o.force)?;
    edit(&app.join("src/views/mod.rs"), |s| {
        add_line(s, "pub mod layout;")
    })?;
    edit(&app.join("src/app.rs"), add_initializer)?;
    let dep = match &o.dep_path {
        Some(p) => format!(r#"loco-ui = {{ path = "{p}", features = ["loco"] }}"#),
        None => format!(r#"loco-ui = {{ git = "{GIT}", features = ["loco"] }}"#),
    };
    edit(&app.join("Cargo.toml"), |s| {
        let s = add_dependency(s, "loco-ui", &dep)?;
        add_dependency(&s, "maud", r#"maud = "0.27""#)
    })?;
    println!("done: `cargo loco generate scaffold <model> <fields..>` now writes Maud views");
    Ok(())
}

/// Write `content` unless the file already holds it; another content is kept unless `force`.
fn write_file(path: &Path, content: &str, force: bool) -> Result<(), String> {
    match fs::read_to_string(path) {
        Ok(old) if old == content => println!("unchanged {}", path.display()),
        Ok(_) if !force => println!("kept      {} (edited; --force overwrites)", path.display()),
        _ => {
            if let Some(dir) = path.parent() {
                fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
            }
            fs::write(path, content).map_err(|e| format!("{}: {e}", path.display()))?;
            println!("wrote     {}", path.display());
        }
    }
    Ok(())
}

/// Apply `f` to the file (created empty when missing) and write it back only if it changed.
fn edit(path: &Path, f: impl Fn(&str) -> Result<String, String>) -> Result<(), String> {
    let old = fs::read_to_string(path).unwrap_or_default();
    let new = f(&old).map_err(|e| format!("{}: {e}", path.display()))?;
    if new == old {
        println!("unchanged {}", path.display());
    } else {
        fs::write(path, new).map_err(|e| format!("{}: {e}", path.display()))?;
        println!("edited    {}", path.display());
    }
    Ok(())
}

/// Append `line` unless a line reads exactly that.
fn add_line(s: &str, line: &str) -> Result<String, String> {
    if s.lines().any(|l| l.trim() == line) {
        return Ok(s.to_string());
    }
    let mut out = s.to_string();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(line);
    out.push('\n');
    Ok(out)
}

/// Put the initializer first in the `vec![..]` that `fn initializers` returns.
fn add_initializer(s: &str) -> Result<String, String> {
    if s.contains(INITIALIZER) {
        return Ok(s.to_string());
    }
    let missing = || format!("no `vec![` in `fn initializers`; add `{INITIALIZER}` to it by hand");
    let at = s.find("fn initializers").ok_or_else(missing)?;
    let open = at + s[at..].find("vec![").ok_or_else(missing)? + "vec![".len();
    let empty = s[open..].trim_start().starts_with(']');
    let insert = if empty {
        INITIALIZER.to_string()
    } else {
        format!("{INITIALIZER}, ")
    };
    Ok(format!("{}{insert}{}", &s[..open], &s[open..]))
}

/// Add `line` as the first entry of `[dependencies]` unless `name` is already a dependency.
fn add_dependency(s: &str, name: &str, line: &str) -> Result<String, String> {
    let mut section = "";
    for l in s.lines() {
        let t = l.trim();
        if t.starts_with('[') {
            section = t;
        } else if section == "[dependencies]" && t.split(['=', ' ', '.']).next() == Some(name) {
            return Ok(s.to_string());
        }
    }
    let at = s
        .find("[dependencies]\n")
        .ok_or("no [dependencies] section")?;
    let at = at + "[dependencies]\n".len();
    Ok(format!("{}{line}\n{}", &s[..at], &s[at..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_initializer_goes_first_once() {
        let fresh = "async fn initializers(_ctx: &AppContext) -> Result<..> {\n    Ok(vec![])\n}";
        let once = add_initializer(fresh).unwrap();
        assert!(once.contains("Ok(vec![Box::new(loco_ui::loco::Initializer)])"));
        assert_eq!(add_initializer(&once).unwrap(), once);
        let other = "fn initializers() { Ok(vec![Box::new(Mine)]) }";
        let added = add_initializer(other).unwrap();
        assert!(added.contains("vec![Box::new(loco_ui::loco::Initializer), Box::new(Mine)]"));
        assert!(add_initializer("fn routes() {}").is_err());
    }

    #[test]
    fn a_dependency_is_added_once_and_only_under_dependencies() {
        let toml = "[workspace.dependencies]\nmaud = \"1\"\n\n[dependencies]\nserde = \"1\"\n";
        let added = add_dependency(toml, "maud", "maud = \"0.27\"").unwrap();
        assert!(added.contains("[dependencies]\nmaud = \"0.27\"\nserde"));
        assert_eq!(
            add_dependency(&added, "maud", "maud = \"0.27\"").unwrap(),
            added
        );
        let dotted = "[dependencies]\nloco-ui.path = \"x\"\n";
        assert_eq!(add_dependency(dotted, "loco-ui", "..").unwrap(), dotted);
    }

    #[test]
    fn a_module_line_is_appended_once() {
        let once = add_line("pub mod auth;", "pub mod layout;").unwrap();
        assert_eq!(once, "pub mod auth;\npub mod layout;\n");
        assert_eq!(add_line(&once, "pub mod layout;").unwrap(), once);
    }
}
