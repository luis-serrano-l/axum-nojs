# Blocked

Questions only the owner can answer. Everything else in ROADMAP.md is done.

## License and repository for `cargo publish`

`cargo publish --dry-run` succeeds for both `wo-caps` and `webonsive` (M6 and M9 boxes
ticked). A real publish to crates.io is rejected without a `license` (or `license-file`),
and `repository` is expected too. Nothing was chosen on your behalf. Also: is `wo-caps` the
name you want on crates.io (it is short and free as of 2026-09-23), or `webonsive-caps`?

Suggested, if you agree (same block in `wo-caps/Cargo.toml` and `webonsive/Cargo.toml`;
`wo-caps` already has `readme = "README.md"`):

```toml
[package]
license = "MIT OR Apache-2.0"
repository = "https://github.com/<you>/webonsive"
readme = "../README.md"   # webonsive only
```

M13 also needs a `LICENSE-MIT` and `LICENSE-APACHE` file at the workspace root with your
name as the copyright holder; say the word and they will be written.

## Reply to Blitz issue #923

Posted on 2026-09-23 after the owner's approval:
https://github.com/DioxusLabs/blitz/issues/923#issuecomment-5800768284
