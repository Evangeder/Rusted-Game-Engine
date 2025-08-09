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