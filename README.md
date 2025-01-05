
# Code for NW Project

## Building

You need:
- Rust toolchain
  - thumbv6m-none-eabi target
- [embassy](https://github.com/embassy-rs/embassy) sources
  - Currently the `Cargo.toml` points to `../../src/embassy`, but that can be changed to any path
- [elf2uf2-rs](https://github.com/JoNil/elf2uf2-rs.git)

`cargo build --release`

## Running

Put pico into bootloader mode (hold down button and then connect to computer)

`cargo run --release`
