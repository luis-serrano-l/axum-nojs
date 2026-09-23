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

## Reply to Blitz issue #923 (approved: post without the last paragraph)

Verified on 2026-09-23 against the sources: html5ever 0.39.0 (pinned by blitz-dom, both
0.3.0-beta.2 and main) has the full declarative-shadow arm; the published beta.2 has no shadow
DOM at all (`stylo.rs` stubs return `None`, `get_template_contents` returns the template itself);
main already implements `get_template_contents` via `DocumentMutator::template_contents`; open
PR #892 adds `DocumentMutator::attach_shadow(host, mode)` and lists declarative shadow DOM as
out of scope. The owner approved posting it on 2026-09-23 without the PR-offer paragraph; the
`gh issue comment` call was denied by the sandbox, so it still needs to be run by hand.

---

Thanks, that matches what I see in the sources. For anyone picking this up, the current state:

- html5ever 0.39.0 (what `blitz-dom` pins on both beta.2 and main) already does the parser side:
  `should_attach_declarative_shadow` checks `shadowrootmode=open|closed` and
  `TreeSink::allow_declarative_shadow_roots`, inserts the template on the open-element stack
  only (never into the tree), calls `TreeSink::attach_declarative_shadow(host, template, attrs)`,
  and on `false` falls back to an inert template (spec step 8.1.1). `blitz-html` does not
  override the default, so it always gets the fallback.
- After a successful attach, the tree builder keeps routing the template's children through
  `get_template_contents(template)` (`tree_builder/mod.rs` ~line 428). So the sink does not need
  to move nodes afterwards; it only has to make that call return the shadow root.
- On `main`, `get_template_contents` is already real (`DocumentMutator::template_contents`), so
  the prerequisite from the report is done. What is missing is the shadow root itself, which
  #892 adds (`DocumentMutator::attach_shadow(host_id, mode)`) while explicitly leaving
  `<template shadowrootmode>` out.

So once #892 lands, the whole fix should be one sink method in `html_sink.rs`, roughly:

```rust
fn attach_declarative_shadow(&self, host: &NodeId, template: &NodeId, attrs: &[Attribute]) -> bool {
    let mode = match attrs.iter().find(|a| a.name.local == local_name!("shadowrootmode")).map(|a| &*a.value) {
        Some("open") => ShadowRootMode::Open,
        Some("closed") => ShadowRootMode::Closed,
        _ => return false,
    };
    let mut mutr = self.mutr();
    let root = mutr.attach_shadow(*host, mode);          // returns the existing root if already shadowed
    mutr.set_template_contents(*template, root);           // so the parser's children land in the shadow root
    true
}
```

plus a small `set_template_contents` on the mutator (the field is private) and, if
`attach_shadow` returning an existing root should count as "already a shadow host", a `false`
return in that case so the template stays inert as the spec requires. The post-parse walk is a
good fallback for older html5ever, but on 0.39 the tree builder already does that work.

I'm happy to open a PR with that on top of #892 once it is merged, if that helps.
