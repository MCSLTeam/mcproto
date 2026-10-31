![Header Image](https://socialify.git.ci/MCSLTeam/mcproto/image?description=1&descriptionEditable=Minecraft%20protocol%20implementation%20in%20Rust.&font=Jost&logo=https%3A%2F%2Fimages.mcsl.com.cn%2Fnew%2FMCServerLauncherFuture.png&name=1&pattern=Circuit%20Board&theme=Auto)  

English | [中文](https://github.com/MCSLTeam/mcproto/blob/master/README_ZH.md)

[![MIT](https://img.shields.io/badge/License-MIT-blue?color=#4ec820)](LICENSE)
![Platform Windows | macOS | Linux](https://img.shields.io/badge/Platform-Windows%20|%20Linux%20|%20macOS-blue?color=#4ec820)

## Overview

mcproto is a modular Rust implementation of the Minecraft: (just Java Edition now) protocol, providing a typed foundation for bots, and other tools that need to communicate with Minecraft servers directly.
It separates wire-level codecs, protocol types, networking,
derive macros, and Microsoft/Minecraft authentication into focused, reusable crates.

## Components

[mcproto](https://github.com/MCSLTeam/mcproto/tree/main/mcproto): The public facade crate, exposing the protocol
libraries through a single feature-controlled entry point.

[mcproto-codec](https://github.com/MCSLTeam/mcproto/tree/main/mcproto-codec): Foundational wire codecs for VarInt,
VarLong, counted I/O, and structured codec errors.

[mcproto-types](https://github.com/MCSLTeam/mcproto/tree/main/mcproto-types): Strongly typed Minecraft data structures,
including protocol primitives, components, slots, profiles, and packet payload types.

[mcproto-derive](https://github.com/MCSLTeam/mcproto/tree/main/mcproto-derive): Procedural macros for packet codecs,
contextual structures, type structures, and fixed protocol enumerations.

[mcproto-network](https://github.com/MCSLTeam/mcproto/tree/main/mcproto-network): Protocol-state packets, packet
metadata, framing, compression, encryption, and reusable encode/decode pipelines.

[mcproto-oauth](https://github.com/MCSLTeam/mcproto/tree/main/mcproto-oauth): Microsoft, Xbox Live, XSTS, and Minecraft
Services authentication flows with synchronous and asynchronous APIs.

## Contribute

We're welcome to your contribution.  
To report issues or suggest improvements,
please [open an issue](https://github.com/MCSLTeam/mcproto/issues/new)
or [submit a pull request](https://github.com/MCSLTeam/mcproto/compare).

## License

This project is distributed under
the [MIT License](LICENSE).
