//! First-party browser chrome only; web semantics remain in the Cherry engine.

pub mod assistant;
pub mod commands;
pub mod design_system;
pub mod home;
pub mod model;
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
    Bookmarks,
    History,
    Settings,
}

impl ShellPage {
    pub const ALL: [Self; 5] = [
        Self::NewTab,
        Self::Research,
        Self::Bookmarks,
        Self::History,
        Self::Settings,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::NewTab => "New Tab",
            Self::Research => "Research",
            Self::Bookmarks => "Bookmarks",
            Self::History => "History",
            Self::Settings => "Settings",
        }
    }
}

#[derive(Debug, Clone)]
pub enum Action {
    Home,
    Back,
    Forward,
    Reload,
    Stop,
    Go,
    Open(String),
    Section(ShellPage),
    Bookmark,
    SaveWorkspace,
    ClearHistory,
    Resume,
}
