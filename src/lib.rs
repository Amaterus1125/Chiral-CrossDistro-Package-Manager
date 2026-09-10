pub mod ui;

mod arch;
mod checksum;
mod commands;
mod constants;
mod db;
mod debian;
mod deps;
mod download;
mod extract;
mod paths;

// Re-exported so `use chiral::{install_binary, remove_binary, ...}` in
// main.rs keeps working exactly as before the split.
pub use commands::{
    info_package, install_binary, list_installed, remove_binary, search_packages,
    self_update, show_deps, update_binary,
};

// Also part of the old public surface of this file.
pub use db::db_list;
pub use deps::resolve_deps;

// Tests live in their own file — see src/host_safety_tests.rs
#[cfg(test)]
mod host_safety_tests;
