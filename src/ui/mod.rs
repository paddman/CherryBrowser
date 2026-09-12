//! CherryBrowser native UX/UI shell.
//!
//! These modules own only the browser chrome and first-party Cherry surfaces.
//! Page semantics and rendering remain in the independent Cherry engine.

pub mod assistant;
pub mod design_system;
pub mod home;
pub mod research;
pub mod settings;
pub mod sidebar;
pub mod theme;
pub mod workspace;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShellPage {
    #[default]
    NewTab,
    Research,
    Settings,
}
