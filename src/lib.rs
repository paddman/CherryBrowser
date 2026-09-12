//! CherryBrowser core.
//!
//! The browser engine in this crate is independent. It does not embed Chromium,
//! WebKit, Gecko, or another browser engine.

pub mod app;
pub mod cancel;
pub mod css;
mod css_syntax;
pub mod dom;
mod font_support;
pub mod html;
pub mod image_data;
pub mod layout;
pub mod loader;
pub mod net;
pub mod renderer;
pub mod text;
mod text_layout;
pub mod ui;
