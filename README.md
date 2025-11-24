# <div align="center">MCRE</div>

<div align="center">

[![MIT licensed][license-badge]][license-url]
[![Build Status][ci-badge]][ci-url]
[![Discord chat][discord-badge]][discord-url]

</div>

**Minecraft Rust Edition**: an ambitious attempt to create a faithful, modern 1:1 clone of Minecraft in Rust.

<sub>*“1:1 clone” refers to behaviour, rendering, and functionality. Replicating every legacy bug is not a goal.</sub>

## 🎯 Goals

The following are the core architectural and product goals MCRE is committed to achieving:

- **True Cross-Platform Support**: The project is being built with portability as a core tenet.
  - **Initial Targets**: Windows, Linux, macOS, and Web (WASM).
  - **Future Targets**: Android and iOS.
- **Universal Crate Portability**: Most individual crates will be able to compile to any target platform, allowing them to be used in various projects outside of the main MCRE game.
- **Vanilla Parity**: MCRE will achieve 1:1 functional equivalence with Minecraft Java Edition regarding mechanics, generation, and rendering.
- **Protocol Compatibility**: The client will be able to connect to standard vanilla servers.

## 🏗️ Design Principles

These principles guide the day-to-day development of the MCRE codebase:

### Architecture & Portability

- **`no_std` First**: To ensure maximum portability, every crate that does not strictly require OS primitives must be `no_std`.
- **Platform Agnosticism**: Logic must be decoupled from the OS. Crates requiring runtime-specific features (like file systems) must utilize generic wrappers/traits.
  - *Example*: A storage trait that maps to `std::fs` on Desktop and `LocalStorage`/`IndexedDB` on Web.
- **Modular Composability**: The project is structured as a collection of independent components. You will be able to use the networking crate without the rendering crate.

### Experience & Performance

- **Rigorous Performance**: MCRE will leverage Rust's zero-cost abstractions to achieve high frame rates and low latency.
- **Developer Experience**: The architecture will be designed with clear APIs, comprehensive documentation, and sensible configuration.
- **Extensibility First**: The architecture must prioritize hooking and modding capabilities from the ground up, avoiding the rigidity of the original codebase.

## ✍️ Contributing

MCRE is in **very early development**, and we’d love for you to be part of its growth.
Join the community on [Discord][discord-url] to follow progress, ask questions, or contribute.

If you are unable to contribute by code, you can still participate by:

* ⭐ Starring the repository on GitHub
* 💬 Joining the discussion on [Discord][discord-url]
* 📝 Improving this README
* 🎨 Helping with branding, visuals, and design ideas

## 📖 License

MCRE is open-source software released under the [MIT License](./LICENSE).

[discord-badge]: https://img.shields.io/discord/1441732591576420397?logo=discord&label=Discord
[discord-url]: https://discord.gg/Xd7E2F5M8m
[license-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[license-url]: https://github.com/mcre-project/mcre/blob/main/LICENSE
[ci-badge]: https://github.com/mcre-project/mcre/actions/workflows/ci.yml/badge.svg?event=push&branch=main
[ci-url]: https://github.com/mcre-project/mcre/actions/workflows/ci.yml?query=event%3Apush+branch%3Amain
