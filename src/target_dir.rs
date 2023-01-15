//! Get the target directory for cargo-run-wasm
use std::{
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

struct CargoMetadata {
    target_directory: String,
    workspace_root: String,
}

impl CargoMetadata {
    fn new(output: Output) -> Self {
        let result = String::from_utf8(output.stdout).expect("Command output should be utf-8");
        let value = serde_json::from_str::<serde_json::Value>(&result)
            .expect("Should match the CargoMetadata definition");
        let obj = value.as_object().expect("Metadata should be object");
        let target_directory = obj
            .get("target_directory")
            .expect("Should have target directory")
            .as_str()
            .unwrap()
            .to_owned();
        let workspace_root = obj
            .get("workspace_root")
            .expect("Should have target directory")
            .as_str()
            .unwrap()
            .to_owned();
        Self {
            target_directory,
            workspace_root,
        }
    }
}

pub(crate) struct CargoDirectories {
    pub workspace_root: PathBuf,
    pub target_directory: PathBuf,
}

pub(crate) fn get_target_directory(
    cargo_executable: &str,
    manifest_dir: &Path,
) -> CargoDirectories {
    // Launch 'cargo metadata' pessimistically
    let mut child = Command::new(cargo_executable)
        .current_dir(manifest_dir)
        .args(["metadata", "--no-deps", "--format-version=1"])
        .stdout(Stdio::piped())
        .spawn()
        .expect("Should be able to run cargo metadata");
    // Then see if we can find the target directory ourselves
    let direct_target = manifest_dir.join("target");
    if direct_target.exists() {
        let _ = child.kill();
        return CargoDirectories {
            target_directory: direct_target,
            workspace_root: manifest_dir.to_path_buf(),
        };
    }
    if let Some(parent) = manifest_dir.parent() {
        let parent_target = parent.join("target");
        if parent_target.exists() {
            let _ = child.kill();
            return CargoDirectories {
                workspace_root: parent.to_path_buf(),
                target_directory: parent_target,
            };
        }
    }
    // Then wait on cargo metadata to finish
    let output = child
        .wait_with_output()
        .expect("Failed to wait on cargo metadata");
    let meta_result = CargoMetadata::new(output);
    CargoDirectories {
        workspace_root: PathBuf::from(meta_result.workspace_root),
        target_directory: PathBuf::from(meta_result.target_directory),
    }
}
