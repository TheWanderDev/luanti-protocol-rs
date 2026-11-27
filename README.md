# luanti-protocol-rs

A rust implementation of the Luanti network protocol, providing the protocol foundation for rust based server and client applications.

<p align="center">
  <a href="https://thewanderdev.github.io/luanti-protocol-rs/">
    <img src="https://img.shields.io/badge/Documentation-User_Guide-2563eb?style=for-the-badge&logo=readthedocs&logoColor=white" alt="Documentation">
  </a>
  <a href="https://thewanderdev.github.io/luanti-protocol-rs/api/luanti_protocol_rs/">
    <img src="https://img.shields.io/badge/API_Reference-Rustdoc-orange?style=for-the-badge&logo=rust&logoColor=white" alt="API Reference">
  </a>
  <a href="https://crates.io/crates/luanti-protocol-rs">
  <img src="https://img.shields.io/crates/v/luanti-protocol-rs?style=for-the-badge&logo=rust&cacheSeconds=300" alt="Latest crates.io version">
</a>
</p>

## Overview

**luanti-protocol-rs** implements the Luanti network protocol in rust

It provides the protocol layer used as the foundation for:

- **luanti-runtime-rs** — luanti client and server runtime (server is paused)
- **luanti-client-headless-rs** — headless client implementation
- **luanti-client-gui-rs** — gui client implementation

the goal is to provide a shared, reusable protocol implementation across these projects

## Installation

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
luanti-protocol-rs = "0.5.182"
```

## Documentation

Explore the documentation:

| Resource | Description |
|---|---|
| [**User Guide**](https://thewanderdev.github.io/luanti-protocol-rs/) | Guides and technical documentation |
| [**API Reference**](https://thewanderdev.github.io/luanti-protocol-rs/api/luanti_protocol_rs/) | Rust API documentation |
| [**Crates.io**](https://crates.io/crates/luanti-protocol-rs) | Published Rust package |
| [**GitHub Repository**](https://github.com/TheWanderDev/luanti-protocol-rs) | Source code and development |

## License

Licensed under the [**LGPL-3.0-only**](LICENSE) license.
