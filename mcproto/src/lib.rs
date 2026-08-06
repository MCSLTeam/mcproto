//! # mcproto
//!
//! Minecraft protocol implementation in Rust. Work in progress.
//!
//! This crate is the public facade for the protocol crates and re-exports them
//! behind feature flags:
//!
//! - [`codec`] provides the wire-level encoding and decoding primitives, such
//!   as [VarInt] and [VarLong], together with a structured [`CodecError`] model.
//! - [`types`] provides Minecraft protocol data types such as
//!   [`PrefixedString`], [`Identifier`], and text components in both the [JSON]
//!   and [NBT] wire representations.
//!
//! # Feature flags
//!
//! - `codec`: enables the [`codec`] module (default).
//! - `types`: enables the [`types`] module and its dependency on `codec`
//!   (default).
//!
//! [VarInt]: https://minecraft.wiki/w/Java_Edition_protocol/Packets#VarInt_and_VarLong
//! [VarLong]: https://minecraft.wiki/w/Java_Edition_protocol/Packets#VarInt_and_VarLong
//! [`CodecError`]: codec::error::CodecError
//! [`PrefixedString`]: types::basic::PrefixedString
//! [`Identifier`]: types::basic::Identifier
//! [JSON]: types::json_text_component
//! [NBT]: types::text_component

#![warn(missing_docs)]

#[cfg(feature = "codec")]
pub use mcproto_codec as codec;
#[cfg(feature = "types")]
pub use mcproto_types as types;
