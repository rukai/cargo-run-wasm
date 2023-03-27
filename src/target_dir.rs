//! Get the target directory for cargo-run-wasm
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct CargoDirectories {
    pub workspace_root: PathBuf,
    pub target_directory: PathBuf,
}

impl CargoDirectories {
    fn from_cargo(cargo_executable: &str, manifest_dir: &Path) -> Self {
        let output = Command::new(cargo_executable)
            .current_dir(manifest_dir)
            .args(["metadata", "--no-deps", "--format-version=1"])
            .output()
            .unwrap();

        let result = String::from_utf8(output.stdout).expect("Command output should be utf-8");
        let value = serde_json::from_str::<serde_json::Value>(&result)
            .expect("Should match the CargoMetadata definition");
        let obj = value.as_object().expect("Metadata should be object");
        let target_directory = PathBuf::from(
            obj.get("target_directory")
                .expect("Should have target directory")
                .as_str()
                .unwrap()
                .to_owned(),
        );
        let workspace_root = PathBuf::from(
            obj.get("workspace_root")
                .expect("Should have target directory")
                .as_str()
                .unwrap()
                .to_owned(),
        );
        CargoDirectories {
            target_directory,
            workspace_root,
        }
    }

    pub fn new(cargo_executable: &str) -> CargoDirectories {
        let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());

        // First try to find the directories ourselves.
        // We can rely on Cargo.toml being correct as Cargo issues warnings when unused/incorrect Cargo.toml's are left around.
        // It is however possible for this to return false positives if the user leaves an unused directory named target next to their Cargo.toml
        // I think this is acceptable though.
        let mut workspace_root = manifest_dir.clone();
        loop {
            let target = workspace_root.join("target");
            let cargo_toml = workspace_root.join("Cargo.toml");
            if target.exists() && cargo_toml.exists() {
                return CargoDirectories {
                    target_directory: target,
                    workspace_root,
                };
            }
            if !workspace_root.pop() {
                break;
            }
        }

        // By default we dont use this code path because I've seen it take between 80-12ms on different machines.
        // The only time this gets used is if the user manually configures cargo to use a different target directory via e.g. the CARGO_TARGET_DIR env var
        // This is because:
        // 1. In order for cargo-run-wasm to be running a target directory must have been created for the cargo-run-wasm binary to live in.
        // 2. If the target directory is in its default location it can always be found by traversing parent directories because the workspace can only create its child packages in a child directory
        CargoDirectories::from_cargo(cargo_executable, &manifest_dir)
    }
}
