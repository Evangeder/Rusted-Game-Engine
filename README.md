# Rusted - Game Engine made in Rust

A modular game engine written in Rust — just for fun and learning. This guide will help you install everything, build, and run the project.

## Prerequisites

1.  **Rust** (latest stable) via rustup:
    ```
    curl https://sh.rustup.rs -sSf | sh
    rustup update stable
    ```
    Or via [windows installer](https://www.rust-lang.org/tools/install)
    
2.  **Build tools**:
    -   **Windows:** Visual Studio Build Tools (with C++ and Windows SDK)
    -   **Linux:**  `build-essential`, `pkg-config`, `cmake`
    -   **macOS:** Xcode Command Line Tools
3.  **Graphics drivers** up to date (Vulkan/Metal/DX12).
4. [Vulkan SDK](https://vulkan.lunarg.com/sdk/home#windows)
5. [Visual Studio 2022 or higher](https://visualstudio.microsoft.com/pl/downloads)
	- MSVC C++ Packages for all architectures, prefferably latest ones
6. IDE of your liking, i use VS Code
    - Install Rust and WGSL addons


## Setting up

Before building, run those commands once:
```
rustup component add rust-analyzer clippy rustfmt
cargo clippy --all-targets -- -D warnings
cargo fmt
rustup target add x86_64-pc-windows-msvc
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-apple-darwin
```

Essentially, this installs optional Rust components:
* `rust-analyzer` - smart code analysis & completion for editors like VS Code,
* `clippy` - linter that catches common mistakes and suggests idiomatic Rust.
    * Clippy runs on all targets (bins, tests, examples) and treats warnings as errors (forces clean code).
* `rustfmt` - automatic code formatter.
And adds targets for cross-compilation for Windows, Linux and Mac.

## Building

Debug build:

```
cargo build
```

Release build:

```
cargo build --release
```

## Running

Run the triangle demo:
```
cargo run --package triangle
```
Or run from the root (if set as default in Cargo.toml):
```
cargo run
```

## Notes

-   This is a hobby project, expect experiments and occasional breakage.
    
-   Code is split into crates for graphics backends, platform handling, shaders, and core engine logic.
    
-   Demos are in `demos/` — start with `triangle`.
    
-   Have fun tinkering!