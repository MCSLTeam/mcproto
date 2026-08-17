//! Typed recipe protocol structures.
//!
//! Implementations live in `recipe/`; this file is the module root so no
//! `recipe/mod.rs` is used.

#[path = "recipe/display.rs"]
mod display;

pub use display::*;
