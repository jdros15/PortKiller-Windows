pub mod app;
pub use app::run;

pub mod config;
pub mod model;
pub mod utils;

// Platform abstraction layer
pub mod platform;

// UI modules (cross-platform)
pub mod ui {
    pub mod icon;
    pub mod menu;
}

// Integrations (some platform-specific)
pub mod integrations {
    #[cfg(target_os = "macos")]
    pub mod brew;

    pub mod docker;

    #[cfg(target_os = "windows")]
    pub mod windows_services;
}

// Re-export platform-specific implementations through unified interface
pub mod process {
    pub mod kill {
        pub use crate::platform::current::kill::*;
    }
    pub mod ports {
        pub use crate::platform::current::ports::*;
    }
}

pub mod notify {
    pub use crate::platform::current::notify::*;
}

pub mod launch {
    pub use crate::platform::current::launch::*;
}

