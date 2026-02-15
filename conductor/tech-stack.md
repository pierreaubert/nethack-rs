# Technology Stack

## Core Language & Architecture
- **Rust (2024 Edition):** The primary language for all components, targeting version 1.93+.
- **Workspace-Based Monorepo:** A modular architecture separating the core engine from frontends and adapters.
- **Core Logic (`nh-core`):** A strictly decoupled game engine designed for high performance and portability.

## Target Environments & Blockchain
- **Native Desktop:** Support for macOS, Linux, and Windows using the Rust toolchain.
- **PolkaVM:** The core engine targets PolkaVM bytecode directly for execution on Polkadot.
- **Pallet Revive:** Integration with Polkadot's Pallet Revive for smart contract deployment, ensuring compatibility with the next-generation contract execution environment.

## Frontends & UI
- **TUI (Terminal User Interface):** Built with `ratatui` and `crossterm` for classic roguelike play.
- **Graphical Engine:** Built with `bevy` for modern graphical tiles and animations.
- **Event-Driven API:** A shared event/command system to bridge the core engine with any frontend.

## Key Libraries & Utilities
- **Serialization:** `serde` for save-game and configuration management.
- **CLI Framework:** `clap` for managing simulation and player parameters.
- **Randomness:** `rand` and `rand_chacha` for deterministic game seeds.
- **Error Handling:** `thiserror` and `strum` for robust and idiomatic Rust patterns.
