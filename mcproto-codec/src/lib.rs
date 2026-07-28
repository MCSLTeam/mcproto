//! mcproto-codec
//!
//! This crate provides a codec for the Minecraft protocol, including support for reading and writing various data types used in the protocol, such as VarInt, VarLong, and other primitive types.
//! It also includes error handling and context management for encoding and decoding operations.
pub mod error;
pub mod io;
pub mod varint;
pub mod varlong;
