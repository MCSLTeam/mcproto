//! Minecraft player profiles and resolvable profile references.
//!
//! The concrete implementations live in `profile/`; this file is the module
//! root so no `profile/mod.rs` is used.

#[path = "profile/game_profile.rs"]
pub mod game_profile;
#[path = "profile/resolvable_profile.rs"]
pub mod resolvable_profile;

pub use game_profile::*;
pub use resolvable_profile::*;
