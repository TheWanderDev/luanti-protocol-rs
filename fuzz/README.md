# Protocol fuzzing

The fuzz targets exercise untrusted packet and payload decoders in `luanti-protocol-rs`.
They are isolated from the main workspace and do not add fuzz dependencies to normal builds.

## Requirements

- Rust nightly
- `cargo-fuzz`
- LLVM sanitizer support and a C++11 compiler

Install the tooling:

```sh
rustup toolchain install nightly
cargo install cargo-fuzz
```

Run from `crates/luanti-protocol-rs`:

```sh
cargo +nightly fuzz run datagram
cargo +nightly fuzz run active_objects
cargo +nightly fuzz run world_data
cargo +nightly fuzz run definitions
cargo +nightly fuzz run server_payloads
```

Limit a run or cap individual inputs with libFuzzer options:

```sh
cargo +nightly fuzz run world_data -- -max_total_time=300 -max_len=1048576
```

Crashes are saved under `fuzz/artifacts/<target>/`, reusable corpus entries belong under
`fuzz/corpus/<target>/`.
