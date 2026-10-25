//! What the desktop app's Tauri commands do, kept apart from Tauri itself.
//!
//! `main.rs` only adapts these functions to `#[tauri::command]`, so every rule
//! here is tested directly, on temporary folders, without a window or a device.

pub mod args;
pub mod cli;
pub mod files;
pub mod wipe;

pub use args::{expand_paths_in_args, sanitize_args};
