//! Phase 9: Custom Application Store Subsystem.
//!
//! Features:
//! - App Categories (System Tools, Productivity, Development, Utilities)
//! - Search & Filter
//! - Detailed App Descriptions, Versions, Licenses
//! - Install and Uninstall state management
//! - Installed Applications registry
//! - System Package Update interface
//!
//! NOTE: This is an educational Micro-OS application repository manager
//! and is not affiliated with or claiming official Snap Store compatibility.

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AppCategory {
    SystemTools,
    Productivity,
    Development,
    Utilities,
}

impl AppCategory {
    pub const fn as_str(&self) -> &'static str {
        match self {
            AppCategory::SystemTools => "System Tools",
            AppCategory::Productivity => "Productivity",
            AppCategory::Development => "Development",
            AppCategory::Utilities => "Utilities",
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct AppPackage {
    pub id: u32,
    pub name: &'static str,
    pub version: &'static str,
    pub category: AppCategory,
    pub description: &'static str,
    pub size_kb: u32,
    pub is_installed: bool,
    pub has_update: bool,
}

pub struct AppStore {
    pub search_query: &'static str,
    pub selected_category: Option<AppCategory>,
    pub packages: [AppPackage; 6],
}

impl AppStore {
    pub const fn new() -> Self {
        Self {
            search_query: "",
            selected_category: None,
            packages: [
                AppPackage {
                    id: 1,
                    name: "Nova System Monitor",
                    version: "v1.2.0",
                    category: AppCategory::SystemTools,
                    description: "Real-time task monitor and memory telemetry dashboard",
                    size_kb: 420,
                    is_installed: true,
                    has_update: false,
                },
                AppPackage {
                    id: 2,
                    name: "Nova Terminal Shell",
                    version: "v2.1.0",
                    category: AppCategory::Development,
                    description: "Interactive kernel shell with ANSI color support",
                    size_kb: 280,
                    is_installed: true,
                    has_update: true,
                },
                AppPackage {
                    id: 3,
                    name: "MicroFS Explorer",
                    version: "v1.0.4",
                    category: AppCategory::Productivity,
                    description: "Graphical file hierarchy and procfs inspector",
                    size_kb: 310,
                    is_installed: true,
                    has_update: false,
                },
                AppPackage {
                    id: 4,
                    name: "Wi-Fi Network Center",
                    version: "v1.1.2",
                    category: AppCategory::Utilities,
                    description: "Access Point scanner and 802.11 connection manager",
                    size_kb: 190,
                    is_installed: true,
                    has_update: false,
                },
                AppPackage {
                    id: 5,
                    name: "Quantum Profiler Pro",
                    version: "v3.0.0",
                    category: AppCategory::Development,
                    description: "Dynamic Time Quantum visualization and fairness benchmarking",
                    size_kb: 550,
                    is_installed: false,
                    has_update: false,
                },
                AppPackage {
                    id: 6,
                    name: "Crypto Key Vault",
                    version: "v1.4.1",
                    category: AppCategory::Utilities,
                    description: "Hardware spinlock protected encryption key repository",
                    size_kb: 160,
                    is_installed: false,
                    has_update: false,
                },
            ],
        }
    }

    pub fn install_package(&mut self, id: u32) -> bool {
        for pkg in self.packages.iter_mut() {
            if pkg.id == id {
                pkg.is_installed = true;
                return true;
            }
        }
        false
    }

    pub fn uninstall_package(&mut self, id: u32) -> bool {
        for pkg in self.packages.iter_mut() {
            if pkg.id == id {
                pkg.is_installed = false;
                return true;
            }
        }
        false
    }
}

pub static mut GLOBAL_APP_STORE: AppStore = AppStore::new();
