//! Get the cargo metadata needed for cargo-run-wasm
use std::{
    path::Path,
    process::{Command, Stdio},
};

use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct CargoMetadata {
    pub(crate) target_directory: String,
}

impl CargoMetadata {
    pub(crate) fn new(cargo_executable: &str, manifest_dir: &Path) -> Self {
        let output = Command::new(cargo_executable)
            .current_dir(manifest_dir)
            .args(["metadata", "--no-deps", "--format-version=1"])
            .stdout(Stdio::piped())
            .output()
            .expect("Should be able to run cargo");
        let result = String::from_utf8(output.stdout).expect("Command output should be utf-8");
        serde_json::from_str::<CargoMetadata>(&result)
            .expect("Should match the CargoMetadata definition")
    }
}
