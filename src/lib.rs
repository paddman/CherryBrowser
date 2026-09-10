//! CherryBrowser core.
//!
//! The browser engine in this crate is independent. It does not embed Chromium,
//! WebKit, Gecko, or another browser engine.

pub mod app;
pub mod css;
pub mod dom;
pub mod html;
pub mod layout;
pub mod net;
pub mod renderer;
