//! Desktop Applications Module.

pub mod app_store;
pub mod file_manager;
pub mod settings;
pub mod system_monitor;
pub mod terminal;

pub use app_store::{AppCategory, AppPackage, AppStore, GLOBAL_APP_STORE};
pub use file_manager::FileManagerApp;
pub use settings::SettingsApp;
pub use system_monitor::SystemMonitor;
pub use terminal::TerminalApp;
