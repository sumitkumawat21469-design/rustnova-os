//! In-Memory Micro File System (MicroFS).
//!
//! Provides a read-only hierarchical filesystem for system directories,
//! process stats (/proc), system binaries (/bin), and configuration (/etc).

#[derive(Copy, Clone, Debug)]
pub struct FileEntry {
    pub path: &'static str,
    pub size_bytes: usize,
    pub is_dir: bool,
    pub permissions: &'static str,
}

pub static ROOT_FILESYSTEM: [FileEntry; 10] = [
    FileEntry { path: "/", size_bytes: 4096, is_dir: true, permissions: "drwxr-xr-x" },
    FileEntry { path: "/bin", size_bytes: 4096, is_dir: true, permissions: "drwxr-xr-x" },
    FileEntry { path: "/bin/sh", size_bytes: 48200, is_dir: false, permissions: "-rwxr-xr-x" },
    FileEntry { path: "/bin/sysmon", size_bytes: 62400, is_dir: false, permissions: "-rwxr-xr-x" },
    FileEntry { path: "/etc", size_bytes: 4096, is_dir: true, permissions: "drwxr-xr-x" },
    FileEntry { path: "/etc/os-release", size_bytes: 142, is_dir: false, permissions: "-rw-r--r--" },
    FileEntry { path: "/etc/network.conf", size_bytes: 98, is_dir: false, permissions: "-rw-r--r--" },
    FileEntry { path: "/proc", size_bytes: 0, is_dir: true, permissions: "dr-xr-xr-x" },
    FileEntry { path: "/proc/tasks", size_bytes: 512, is_dir: false, permissions: "-r--r--r--" },
    FileEntry { path: "/proc/meminfo", size_bytes: 256, is_dir: false, permissions: "-r--r--r--" },
];
