## 0.3.1

* Add shortcuts for `--package` (`-p`) and `--release` (`-r`)
* Add support for `--profile` command, previously only `--release` was supported
* Improve target directory detection

## 0.3

* rust API is unchanged
* CLI API is redone to expose all of the `cargo build` CLI API
  * The only thing backwards incompatible is that `cargo run-wasm crate_name` is no longer valid. Instead you should use `cargo run-wasm --package crate_name`

## 0.2

* prehistory
