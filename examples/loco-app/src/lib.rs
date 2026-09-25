//! A minimal Loco app on axum-nojs: sign-in and one scaffolded model, every page working with
//! script off. The notes controller and views were written by `cargo loco generate scaffold`
//! with the templates in `.loco-templates/` (copied from `axum-nojs/loco-templates/`).

pub mod app;
pub mod controllers;
pub mod models;
pub mod views;
