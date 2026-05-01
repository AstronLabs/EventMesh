# Soroban Contracts

This folder contains the Soroban smart contract crate for EventMesh.

## Install

The contract crate uses the Soroban SDK from Cargo. From this directory run:

```bash
cargo test
```

If you want the Soroban CLI locally, install it with:

```bash
cargo install soroban-cli --locked
```

## Build

```bash
cargo build --target wasm32-unknown-unknown --release
```

## Structure

- `src/lib.rs` contains the starter contract.
- `Cargo.toml` pins the Soroban SDK and release settings.
