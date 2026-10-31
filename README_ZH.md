# ![Header Image](https://socialify.git.ci/MCSLTeam/mcproto/image?description=1&descriptionEditable=Minecraft%20protocol%20implementation%20in%20Rust.&font=Jost&logo=https%3A%2F%2Fimages.mcsl.com.cn%2Fnew%2FMCServerLauncherFuture.png&name=1&pattern=Circuit%20Board&theme=Auto)

中文 | [English](README.md)

[![MIT](https://img.shields.io/badge/License-MIT-blue?color=#4ec820)](LICENSE)
![Platform Windows | macOS | Linux](https://img.shields.io/badge/Platform-Windows%20|%20Linux%20|%20macOS-blue?color=#4ec820)

## 概览

mcproto 是一个模块化的 Minecraft: Java Edition 协议 Rust 实现，用于为机器人以及其他需要直接与 Minecraft
服务器通信的工具提供类型安全。目前仅支持 Java 版。

它将编解码、协议类型、网络层、派生宏以及 Microsoft/Minecraft 身份验证拆分到职责明确、可复用的
crate 中。

## 组件

[mcproto](https://github.com/MCSLTeam/mcproto/tree/main/mcproto)：公共 crate，用于控制 feature。

[mcproto-codec](https://github.com/MCSLTeam/mcproto/tree/main/mcproto-codec)：提供 VarInt、VarLong、带字节计数
的 I/O 和结构化编解码错误等 codec。

[mcproto-types](https://github.com/MCSLTeam/mcproto/tree/main/mcproto-types)：强类型的 Minecraft 数据结构，
包括协议基础类型、Component、Slots、Profile和Payload类型。

[mcproto-derive](https://github.com/MCSLTeam/mcproto/tree/main/mcproto-derive)：用于包 codec、上下文结构、
类型结构和固定协议枚举的过程宏。

[mcproto-network](https://github.com/MCSLTeam/mcproto/tree/main/mcproto-network)：协议状态包、包元数据、帧处理、
压缩、加密以及可复用的编解码。

[mcproto-oauth](https://github.com/MCSLTeam/mcproto/tree/main/mcproto-oauth)：提供同步和异步 API 的 Microsoft、
Xbox Live、XSTS 和 Minecraft Services 身份验证流程。

## 贡献

我们欢迎广大开发者的贡献。

如需报告问题或提出改进建议，请[创建 Issue](https://github.com/MCSLTeam/mcproto/issues/new)
或[提交 Pull Request](https://github.com/MCSLTeam/mcproto/compare)。

## 许可证

本项目基于 [MIT License](LICENSE) 发布。
