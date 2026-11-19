//! Build script. Exports the effective version (T-21, O-36) as `OXYSOL_VERSION` for the menu label: the
//! metadata version with the patch derived from git, or `<version>+src` without git history. The same
//! file computes it for the application build script and for `cargo xtask version`.

use std::env;
use std::path::PathBuf;

#[path = "../../xtask/src/version_scheme.rs"]
#[allow(
    dead_code,
    reason = "the file is shared with xtask and the application build script; each uses a part"
)]
mod version_scheme;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let base = env::var("CARGO_PKG_VERSION").unwrap_or_default();
    let version = match env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => {
            let root = PathBuf::from(dir).join("..").join("..");
            for file in version_scheme::watched_files(&root) {
                println!("cargo:rerun-if-changed={}", file.display());
            }
            version_scheme::build_version(&root, &base)
        }
        Err(_) => version_scheme::source_fallback(&base),
    };
    println!("cargo:rustc-env=OXYSOL_VERSION={version}");
}
