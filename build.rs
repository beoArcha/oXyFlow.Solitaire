//! Build script: embeds the application icon and version information into the Windows executable.
//! `winresource` takes `FileVersion` and `ProductVersion` from `CARGO_PKG_VERSION`.

use std::env;
use std::process::ExitCode;

const ICON_PATH: &str = "assets/icon/generated/oxysol.ico";

fn main() -> ExitCode {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={ICON_PATH}");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return ExitCode::SUCCESS;
    }
    match embed_windows_resources() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            println!("cargo:warning=cannot embed the Windows icon: {message}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(windows)]
fn embed_windows_resources() -> Result<(), std::io::Error> {
    const PRODUCT_NAME: &str = "oXySol";
    const FILE_DESCRIPTION: &str = "oXySol - terminal solitaire";
    let mut resource = winresource::WindowsResource::new();
    resource
        .set_icon(ICON_PATH)
        .set("ProductName", PRODUCT_NAME)
        .set("FileDescription", FILE_DESCRIPTION)
        .set("OriginalFilename", "oxysol.exe")
        .set("InternalName", "oxysol");
    resource.compile()
}

#[cfg(not(windows))]
fn embed_windows_resources() -> Result<(), std::io::Error> {
    Ok(())
}
