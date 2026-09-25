# loco-app

A [Loco](https://loco.rs) app on axum-nojs: sign up, sign in, and notes with list, show, new,
edit and delete, every page working with script off.

```sh
cd examples/loco-app
cargo loco start          # http://localhost:5150 (sqlite file created and migrated on start)
cargo test -p loco-app    # every page through Loco's router and Blitz
```

`src/controllers/notes.rs` and `src/views/notes.rs` are generator output, not hand-written:

```sh
cp -r ../../axum-nojs/loco-templates .loco-templates   # already here
cargo loco generate scaffold note title:string! body:text done:bool! due:date
```

`db entities` in that step needs `sea-orm-cli` 2 on `PATH`.
