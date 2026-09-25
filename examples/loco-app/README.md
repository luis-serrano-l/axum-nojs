# loco-app

A [Loco](https://loco.rs) app on loco-ui: accounts (sign up, sign in, forgot and reset
password, email verification, magic link) and notes with list, show, new, edit and delete,
every page working with script off.

```sh
cd examples/loco-app
cargo loco start          # http://localhost:5150 (sqlite file created and migrated on start)
cargo test -p loco-app    # every page through Loco's router and Blitz
```

`src/controllers/notes.rs` and `src/views/notes.rs` are generator output, not hand-written:

```sh
cp -r ../../loco-ui/loco-templates .loco-templates   # already here
cargo loco generate scaffold note title:string! body:text done:bool! due:date
```

`db entities` in that step needs `sea-orm-cli` 2 on `PATH`.

`src/controllers/account.rs` and `src/views/account.rs` are `cargo lui auth` output on the
starter's `users` model and `AuthMailer` (`src/mailers/`); a test in `loco-ui` fails if they
drift from the templates. Mails go to SMTP on 1025 in development (Mailpit shows them) and
stay in memory in tests.
