//! Type-safe entity metadata and particle protocol structures.
//!
//! The implementation follows the current Java Edition [Entity Metadata
//! Format]. Submodules live in `entity_metadata/`; this file is the module
//! root so no `entity_metadata/mod.rs` is used.
//!
//! [Entity Metadata Format]: https://minecraft.wiki/w/Java_Edition_protocol/Entity_metadata#Entity_Metadata_Format

#[path = "entity_metadata/metadata.rs"]
mod metadata;
#[path = "entity_metadata/particle.rs"]
mod particle;
#[path = "entity_metadata/types.rs"]
mod types;

pub use metadata::*;
pub use particle::*;
pub use types::*;
