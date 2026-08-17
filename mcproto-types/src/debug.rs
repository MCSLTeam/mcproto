//! Debug subscription events, updates, and their typed payloads.
//!
//! Implementations live in `debug/`; this file is the module root so no
//! `debug/mod.rs` is used.

#[path = "debug/data.rs"]
pub mod data;
#[path = "debug/event.rs"]
pub mod event;
#[path = "debug/update.rs"]
pub mod update;

pub use data::*;
pub use event::*;
pub use update::*;
