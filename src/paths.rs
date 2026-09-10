//! Filesystem locations that depend on whether we're running as root.

use std::path::PathBuf;

pub fn is_root() -> bool {
    unsafe { libc::getuid() == 0 }
}

pub fn home() -> Result<PathBuf, String> {
    std::env::var("HOME")
        .map(PathBuf::from)
        .map_err(|_| "$HOME is not set".to_string())
}

pub fn install_prefix() -> Result<PathBuf, String> {
    if is_root() { Ok(PathBuf::from("/usr/local")) }
    else         { Ok(home()?.join(".local")) }
}
