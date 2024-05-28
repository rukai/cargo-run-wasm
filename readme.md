# cargo run-wasm

[![Crates.io](https://img.shields.io/crates/v/cargo-run-wasm.svg)](https://crates.io/crates/cargo-run-wasm)
[![Docs](https://docs.rs/cargo-run-wasm/badge.svg)](https://docs.rs/cargo-run-wasm)
[![dependency status](https://deps.rs/repo/github/rukai/cargo-run-wasm/status.svg)](https://deps.rs/repo/github/rukai/cargo-run-wasm)

Allows running wasm applications and examples as simply as:

```bash
cargo run-wasm --example example_name
```

or

```bash
cargo run-wasm --package crate_name
```

or

```bash
cargo run-wasm --bin bin_name
```

In the background it:

1. Compiles the rust project to wasm
2. Runs wasm-bindgen
3. Generates an index.html that runs the wasm.
4. Launches a tiny webserver to serve index.html + your wasm

## Setup

1. Setup your wasm runnable project as a crate within a [cargo workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
2. Create a crate in the workspace named run-wasm with:

`Cargo.toml`:

```toml
[package]
name = "run-wasm"
version = "0.1.0"
edition = "2021"

[dependencies]
cargo-run-wasm = "0.3.0"
```

`main.rs`:

```rust
fn main() {
    cargo_run_wasm::run_wasm_cli_with_css("body { margin: 0px; }");
}
```

3. Create a `.cargo/config` file containing:

```toml
[alias]
run-wasm = "run --release --package run-wasm --"
```

4. Thats it, you can now run the commands described earlier. You can also run `cargo run-wasm --help` to view all the possible flags.

Note: If you want to avoid restructuring your project into a proper workspace you can do so by combining your workspace and crate `Cargo.toml` into a single file like [winit does](https://github.com/rust-windowing/winit/blob/337d50779c299240f6e0a67ef3e852f1c971cf16/Cargo.toml#L315).

## Advantages over an equivalent bash/powershell/bat script

* cross platform
* 0 external dependencies
* better UX + more robust than anything hacked together with bash/powershell/bat
* wasm-bindgen-cli version is always in sync with wasm-bindgen version because `cargo update` updates both of them at the same time thanks to being in the same workspace

## cargo custom command

cargo-run-wasm is not available as a [cargo custom command](https://doc.rust-lang.org/book/ch14-05-extending-cargo.html) as that would cause:

* issues with mismatches between wasm-bindgen versions
* issues with keeping a stable interface with the wasm app
* gives the idea that the command is compatible with every project that uses wasm which is not the case.

## Configuration

If you wish to set custom css, do so in the string argument to `run_wasm_cli_with_css`.

However it is not possible to set custom html from cargo-run-wasm, instead any DOM elements you require should be created from within your crate or example using [web-sys](https://docs.rs/web-sys/latest/web_sys/struct.Document.html#method.create_element) or another crate.
The reasoning is that in the case an example requires custom HTML it will probably:

* require HTML that is unique to that example while cargo-run-wasm is only capable of global settings
* require web-sys or similar to interact with the DOM at runtime anyway

## MSRV

Since this tool has a trivial implementation the MSRV is at 1.70 and will only be increased if dependencies require it.
If it is ever increased it must be below the MSRV of maintained branches of important users such as [wgpu](https://github.com/gfx-rs/wgpu#msrv-policy) and [winit](https://github.com/rust-windowing/winit).

The MSRV is enforced in CI and locally via the `rust-toolchain.toml` file.

## Alternatives

* [wasm-server-runner](https://github.com/jakobhellermann/wasm-server-runner)
  * Advantages
    * Integrates better with cargo
    * Doesnt require any setup to live within the project
  * Disadvantages
    * Requires the user to run `cargo install wasm-server-runner`
    * wasm-bindgen versions can go out of sync causing issues [like this](https://github.com/jakobhellermann/wasm-server-runner/issues/2)

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
