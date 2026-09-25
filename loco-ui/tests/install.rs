//! `cargo lui install` on a fresh Loco app. `tests/fresh-loco-app` is `loco new -n fresh_app
//! --db sqlite --bg blocking --assets none` (loco 1.2.0) trimmed to what `cargo check` reads:
//! `Cargo.toml`, `Cargo.lock`, `src/` and `migration/`.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

const FIXTURE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fresh-loco-app");

/// A copy of the fixture in `target/`, so every run starts from `loco new`'s files.
fn fresh_copy(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../target/lui-install")
        .join(name);
    let _ = fs::remove_dir_all(&dir);
    copy_dir(Path::new(FIXTURE), &dir);
    dir
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

/// Run the installer on `app` against this checkout; its stdout.
fn install(app: &Path) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_cargo-lui"))
        .args(["lui", "install"])
        .arg(app)
        .args(["--dep-path", env!("CARGO_MANIFEST_DIR")])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

fn read(app: &Path, file: &str) -> String {
    fs::read_to_string(app.join(file)).unwrap()
}

#[test]
fn install_writes_every_piece_once() {
    let app = fresh_copy("once");
    let first = install(&app);
    assert_eq!(first.matches("wrote").count(), 3, "{first}");
    assert_eq!(first.matches("edited").count(), 3, "{first}");
    assert_eq!(
        read(&app, ".loco-templates/scaffold/api/controller.t"),
        include_str!("../loco-templates/scaffold/api/controller.t")
    );
    assert!(read(&app, "src/app.rs").contains("Ok(vec![Box::new(loco_ui::loco::Initializer)])"));
    assert!(read(&app, "src/views/mod.rs").contains("pub mod auth;\npub mod layout;\n"));
    assert!(read(&app, "src/views/layout.rs").contains("pub fn page(ui: &Ui"));
    let toml = read(&app, "Cargo.toml");
    assert!(toml.contains("\nloco-ui = { path = "), "{toml}");
    assert!(toml.contains("maud = \"0.27\""), "{toml}");

    let second = install(&app);
    assert!(
        !second.contains("wrote") && !second.contains("edited"),
        "{second}"
    );
    assert_eq!(second.matches("unchanged").count(), 6, "{second}");
}

#[test]
fn an_edited_file_is_kept() {
    let app = fresh_copy("edited");
    install(&app);
    fs::write(app.join("src/views/layout.rs"), "// mine\n").unwrap();
    assert!(install(&app).contains("kept"));
    assert_eq!(read(&app, "src/views/layout.rs"), "// mine\n");
}

#[test]
fn a_directory_that_is_not_a_loco_app_is_refused() {
    let out = Command::new(env!("CARGO_BIN_EXE_cargo-lui"))
        .args(["install", env!("CARGO_MANIFEST_DIR")])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("not a Loco app"));
}

/// Builds Loco and SeaORM for the fixture (about a minute the first time), so it is run by
/// `scripts/verify.sh` and CI with `--ignored` rather than by every `cargo test`.
#[test]
#[ignore]
fn the_installed_app_compiles() {
    let app = fresh_copy("check");
    install(&app);
    let target = Path::new(env!("CARGO_MANIFEST_DIR")).join("../target/lui-install-target");
    let out = Command::new(env!("CARGO"))
        .arg("check")
        .current_dir(&app)
        .env("CARGO_TARGET_DIR", target)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
