#![allow(clippy::cargo_common_metadata)]

use std::process::Command;

fn main() {
    println!("cargo::rustc-check-cfg=cfg(unstable)");

    let rustc_version = Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .unwrap_or_else(|| "unknown".to_string());

    if rustc_version.contains("nightly") || rustc_version.contains("dev") {
        println!("cargo:rustc-cfg=unstable");
    }
}
