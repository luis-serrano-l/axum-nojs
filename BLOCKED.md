# Blocked

Questions only the owner can answer. Everything else in ROADMAP.md is done.

## License and repository for `cargo publish`

`cargo publish --dry-run -p webonsive` succeeds, so the M6 box is ticked. A real publish to
crates.io is rejected without a `license` (or `license-file`) in `webonsive/Cargo.toml`, and
`repository` / `readme` are expected too. Nothing was chosen on your behalf.

Suggested, if you agree:

```toml
[package]
license = "MIT OR Apache-2.0"
repository = "https://github.com/<you>/webonsive"
readme = "../README.md"
```
