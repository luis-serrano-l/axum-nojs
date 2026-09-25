//! A minimal Loco app on loco-ui: accounts and one scaffolded model, every page working with
//! script off. The notes controller and views were written by `cargo loco generate scaffold`
//! with the templates in `.loco-templates/` (copied from `loco-ui/loco-templates/`); the
//! account pages by `cargo lui auth` on the starter's `users` model and `AuthMailer`.

pub mod app;
pub mod controllers;
pub mod mailers;
pub mod models;
pub mod views;
