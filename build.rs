use std::{
    collections::HashMap, 
    fs, 
    path::Path, 
    process::Command
};

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use serde::Deserialize;

pub struct Commit {
    pub author: String,
    pub email: String,
    pub date: String,
    pub date_small: String,
    pub sha: String,
    pub sha_small: String,
    pub message: String,
    pub files: usize,
    pub added: usize,
    pub deleted: usize,
    pub total: usize,
}

impl Commit {
    /// # Panics
    /// Panicked when git failed
    #[must_use]
    pub fn new() -> Self {
        let log = Command::new("git")
            .env("LC_ALL", "C")
            .args([
                "log",
                "--format=%an%n%ae%n%ad%n%H%n%h%n%s",
                "--date=format:%b %d %Y, %H:%M:%S",
                "-1",
            ])
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .expect("Failed to call Git log");

        let log_parts: Vec<&str> = log.split('\n').collect();
        assert!(
            log_parts.len() >= 6,
            "Failed to call Git log: {log_parts:?}"
        );

        let author = log_parts[0].to_owned();
        let email = log_parts[1].to_owned();
        let date = log_parts[2].to_owned();
        let date_small = date[..date.find(',').unwrap()].to_owned();
        let sha = log_parts[3].to_owned();
        let sha_small = log_parts[4].to_owned();
        let message = log_parts[5].to_owned();

        let numstat_raw = Command::new("git")
            .env("LC_ALL", "C")
            .args([
                "log", "--shortstat", "-1",
                "--first-parent", "-m", "--format=",
            ])
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .expect("Failed to call Git shortstat");

        let mut nums = numstat_raw
            .split(|c: char| !c.is_ascii_digit())
            .filter(|s| !s.is_empty())
            .filter_map(|s| s.parse::<usize>().ok());

        let files = nums.next().unwrap_or(0);
        let added = nums.next().unwrap_or(0);
        let deleted = nums.next().unwrap_or(0);
        let total = added + deleted;

        Self {
            author,
            email,
            date,
            date_small,
            sha,
            sha_small,
            message,
            files,
            added,
            deleted,
            total,
        }
    }
}

impl Default for Commit {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize)]
struct LogoEntry {
    names: Vec<String>,
    lines: String,
    colors: Vec<String>,
    color_keys: String,
    color_title: String,
}

fn generate(input_json: &Path, out_dir: &Path) -> std::io::Result<()> {
    let text = fs::read_to_string(input_json)?;
    let map: HashMap<String, Vec<LogoEntry>> =
        serde_json::from_str(&text).expect("invalid logo JSON");

    for (letter, entries) in &map {
        let code = generate_module(letter, entries);
        fs::write(out_dir.join(format!("{letter}.rs")), code.to_string())?;
    }
    Ok(())
}

fn generate_module(letter: &str, entries: &[LogoEntry]) -> TokenStream {
    let static_name = Ident::new(&letter.to_ascii_uppercase(), Span::call_site());
    let logo_infos = entries.iter().map(generate_logo_info);

    quote! {
        pub static #static_name: &[crate::logo::LogoInfo] = &[
            #(#logo_infos),*
        ];
    }
}

fn generate_logo_info(entry: &LogoEntry) -> TokenStream {
    let names: Vec<_> = entry.names.iter().collect();
    let path_lit = &entry.lines;

    let colors: Vec<syn::Path> = entry
        .colors
        .iter()
        .map(|c| syn::parse_str::<syn::Path>(c).expect("bad color path"))
        .collect();

    let color_keys: syn::Path = syn::parse_str(&entry.color_keys).expect("bad color_keys path");
    let color_title: syn::Path = syn::parse_str(&entry.color_title).expect("bad color_title path");

    quote! {
        crate::logo::LogoInfo {
            names: &[#(#names),*],
            lines: include_bytes!(concat!(env!("LOGO_OUT_DIR"), #path_lit)),
            colors: &[
                #(#colors),*
            ],
            color_keys: #color_keys,
            color_title: #color_title,
        }
    }
}

mod setup {
    pub fn build_bypass() {
        use std::{path::PathBuf, time::SystemTime};

        let timestamp = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .to_string();

        let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
        println!("cargo:rustc-env=LOGO_OUT_DIR={}", out_dir.display());
        let trigger_file = out_dir.join(format!("build_trigger_{timestamp}"));
        std::fs::write(&trigger_file, &timestamp).unwrap();

        println!("cargo:rerun-if-changed={}", trigger_file.display());
        println!("cargo:rerun-if-env-changed=BUILD_TIMESTAMP_{timestamp}");
    }

    pub fn lua_and_libc() {
        use std::env;

        let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
        let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();

        match (target_os.as_str(), target_arch.as_str()) {
            ("linux", "x86_64") => {
                println!("cargo:rustc-link-search=native=/usr/lib/x86_64-linux-gnu");
                println!("cargo:rustc-link-lib=static=lua5.4");
                println!("cargo:rustc-link-lib=dylib=dl");
                println!("cargo:rustc-link-lib=dylib=m");
                println!("cargo:rustc-link-arg=-pthread");
                println!("cargo:rustc-link-arg=-lc");

                println!("cargo:rustc-cfg=lua54");
            }
            ("android", "aarch64") => {
                let lua_dir =
                    env::var("LUA_ANDROID_LIB_DIR").unwrap_or("bin/android-aarch64".to_owned());
                println!("cargo:rustc-link-search=native={lua_dir}");
                println!("cargo:rustc-link-lib=static=lua5.4");
                println!("cargo:rustc-link-arg=-Wl,--no-as-needed");
                println!("cargo:rustc-link-arg=-lc");
                println!("cargo:rustc-link-lib=dylib=m");
                println!("cargo:rustc-link-lib=dylib=dl");

                println!("cargo:rustc-cfg=lua54");
            }
            ("windows", _) => {
                println!("cargo:rustc-link-search=native=bin/windows");
                println!("cargo:rustc-link-lib=static=lua55");

                // FIXME: This just suppresses the error rather than solving it,
                // in the future Lua should be built manually for the linker that Rust uses
                println!("cargo:rustc-link-arg=/NODEFAULTLIB:LIBCMT");

                println!("cargo:rustc-cfg=lua5_5");
            }
            _ => {}
        }
    }

    pub fn offset_bits_cfg() {
        let offset_bits_64 = std::env::var("CARGO_CFG_TARGET_POINTER_WIDTH")
            .is_ok_and(|w| w == "64");

        if offset_bits_64 {
            println!("cargo:rustc-cfg=file_offset_bits_64");
        } else {
            println!("cargo:rustc-cfg=file_offset_bits_32");
        }
        println!("cargo:rustc-check-cfg=cfg(file_offset_bits_64)");
        println!("cargo:rustc-check-cfg=cfg(file_offset_bits_32)");
    }

    pub fn compress_logos() -> (usize, usize) {
        use std::{
            path::PathBuf,
            sync::atomic::{AtomicUsize, Ordering},
        };

        use zlib_rs::{DeflateConfig, ReturnCode, compress_bound, compress_slice};

        use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

        static VALID_CHARS: &[char] = &[
            'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q',
            'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '_',
        ];

        let raw_bytes_len = AtomicUsize::new(0);
        let encoded_bytes_len = AtomicUsize::new(0);

        let base_path = PathBuf::from("src/logo");
        let mut all_paths = Vec::new();
        for letter in VALID_CHARS {
            let letter_path = base_path.join(letter.to_string());
            if let Ok(entries) = std::fs::read_dir(letter_path) {
                for entry in entries.flatten() {
                    all_paths.push(entry.path());
                }
            }
        }

        let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
        all_paths.par_iter().for_each(|path| {
            let content = std::fs::read(path).unwrap();
            raw_bytes_len.fetch_add(content.len(), Ordering::Relaxed);

            let mut compressed_buf = vec![0u8; compress_bound(content.len())];
            let (compressed, rc) =
                compress_slice(&mut compressed_buf, &content, DeflateConfig::default());
            encoded_bytes_len.fetch_add(compressed.len(), Ordering::Relaxed);
            assert_eq!(rc, ReturnCode::Ok);

            let letter = path
                .parent()
                .and_then(|p| p.file_name())
                .unwrap()
                .to_str()
                .unwrap();
            let dest_dir = out_dir.join("temp").join(letter);
            std::fs::create_dir_all(&dest_dir).ok();
            let dest_path = dest_dir.join(path.file_name().unwrap());
            std::fs::write(dest_path, &*compressed).ok();
        });

        let raw = raw_bytes_len.load(Ordering::Relaxed);
        let encoded = encoded_bytes_len.load(Ordering::Relaxed);

        (raw, encoded)
    }

    pub fn generate_logos() {
        use std::path::PathBuf;

        let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
        let json_path = manifest_dir.join("logo.json");
        let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("logo");

        println!("cargo:rerun-if-changed={}", json_path.display());

        std::fs::create_dir_all(&out_dir).expect("create OUT_DIR/logo");
        crate::generate(&json_path, &out_dir).expect("Generate logo modules");
    }

    pub fn generate_help() {
        use std::path::PathBuf;

        use termimad::MadSkin;

        static HELP_RAW: &str = concat!(
            "lazyfetch is a neofetch-like tool for beautiful system information display with flexible output customization\n",
            "\n",
            "**Usage: lazyfetch*** <?options>*\n",
            "\n",
            "**Commands:**\n",
            "  -h, --help <?options>     \tPrint this message\n",
            "  -v, --version <?options>  \tPrint lazyfetch version\n",
            "  -l, --logo                \tCustom logo (name or file)\n",
            "  -c, --config              \tCustom preset (http url or file)",
        );

        let skin = MadSkin::default();
        let string = skin.text(HELP_RAW, None).to_string();

        let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("help.txt");
        std::fs::write(&out_dir, string).expect("Create OUT_DIR/help.txt");
    }

    pub mod env {
        pub fn target() {
            let target = std::env::var("TARGET").unwrap();
            println!("cargo:rustc-env=TARGET={target}");
        }

        pub fn target_os() -> String {
            let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
            println!("cargo:rustc-env=TARGET_OS={target_os}");
            target_os
        }

        pub fn target_arch() {
            let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
            println!("cargo:rustc-env=TARGET_ARCH={target_arch}");
        }

        pub fn build_time() {
            let build_time = chrono::Local::now()
                .format("%b %d %Y, %H:%M:%S")
                .to_string();
            println!("cargo:rustc-env=COMPILE_TIME={build_time}");
        }

        pub fn rustc_version() -> String {
            use std::process::Command;

            let mut rustc_version = Command::new("rustc")
                .arg("--version")
                .output()
                .ok()
                .and_then(|output| String::from_utf8(output.stdout).ok())
                .unwrap_or_else(|| "unknown".to_string());

            if let Some(idx) = rustc_version.find('(') {
                rustc_version = rustc_version[..idx - 1].trim().to_string();
            }
            println!("cargo:rustc-env=RUSTC_VERSION={}", rustc_version.trim());
            rustc_version
        }

        pub fn cargo_version() {
            use std::process::Command;

            let mut cargo_version = Command::new("cargo")
                .arg("--version")
                .output()
                .ok()
                .and_then(|output| String::from_utf8(output.stdout).ok())
                .unwrap_or_else(|| "Unknown".to_string());

            if let Some(idx) = cargo_version.find('(') {
                cargo_version = cargo_version[..idx - 1].trim().to_string();
            }
            println!("cargo:rustc-env=CARGO_VERSION={}", cargo_version.trim());
        }

        pub fn commit() {
            println!("cargo:rerun-if-env-changed=GITHUB_SHA");
            println!("cargo:rerun-if-env-changed=GITHUB_REF_NAME");
            println!("cargo:rerun-if-env-changed=GITHUB_HEAD_REF");

            let commit = crate::Commit::new();

            println!("cargo:rustc-env=COMMIT_AUTHOR={}", commit.author);
            println!("cargo:rustc-env=COMMIT_EMAIL={}", commit.email);
            println!("cargo:rustc-env=COMMIT_DATE={}", commit.date);
            println!("cargo:rustc-env=COMMIT_DATE_SMALL={}", commit.date_small);
            println!("cargo:rustc-env=COMMIT_SHA={}", commit.sha);
            println!("cargo:rustc-env=COMMIT_SHA_SMALL={}", commit.sha_small);
            println!("cargo:rustc-env=COMMIT_MESSAGE={}", commit.message);
            println!("cargo:rustc-env=COMMIT_FILES={}", commit.files);
            println!("cargo:rustc-env=COMMIT_ADDED={}", commit.added);
            println!("cargo:rustc-env=COMMIT_DELETED={}", commit.deleted);
            println!("cargo:rustc-env=COMMIT_TOTAL={}", commit.total);
        }

        pub fn libc_version(target_os: &str) {
            #[cfg(target_os = "linux")]
            fn get_libc_version() -> String {
                use std::{path::PathBuf, process::Command};

                let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
                let c_file = out_dir.join("version.c");
                let exe = out_dir.join("version");

                std::fs::write(
                    &c_file,
                    r#"
                    #include <stdio.h>

                    int main() {
                        printf("%d.%d\n", __GLIBC__, __GLIBC_MINOR__);
                        return 0;
                    }
                "#,
                )
                .unwrap();

                let status = Command::new("gcc")
                    .arg(&c_file)
                    .arg("-o")
                    .arg(&exe)
                    .status()
                    .expect("failed to compile C program");

                assert!(status.success(), "Compilation failed");

                let output = Command::new(&exe).output().expect("failed to run program");

                String::from_utf8(output.stdout).unwrap().trim().to_string()
            }

            let ver = match target_os {
                #[cfg(target_os = "linux")]
                "linux" => get_libc_version(),
                "android" => "bionic".to_string(),
                _ => String::new(),
            };
            println!("cargo:rustc-env=LIBC_VERSION={ver}");
        }

        pub fn project_hash() {
            use std::fmt::Write as _;
            use std::fs;
            use std::io::Read;
            use std::path::{Path, PathBuf};

            use sha2::{Digest, Sha256};

            fn collect_files(dir: &Path) -> Vec<PathBuf> {
                let mut files = Vec::new();
                let mut stack = vec![dir.to_path_buf()];

                while let Some(current) = stack.pop() {
                    let Ok(entries) = fs::read_dir(&current) else {
                        continue;
                    };
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            stack.push(path);
                        } else {
                            files.push(path);
                        }
                    }
                }

                files.sort();
                files
            }

            fn hash_file(hasher: &mut Sha256, path: &Path) -> std::io::Result<()> {
                hasher.update(path.to_string_lossy().as_bytes());
                let mut file = fs::File::open(path)?;
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;
                hasher.update(&buffer);
                Ok(())
            }

            fn to_hex(bytes: &[u8]) -> String {
                let mut s = String::with_capacity(bytes.len() * 2);
                for b in bytes {
                    let _ = write!(s, "{b:02x}");
                }
                s
            }

            let mut hasher = Sha256::new();
            for path in collect_files(Path::new("src")) {
                if let Err(err) = hash_file(&mut hasher, &path) {
                    eprintln!("cargo:warning=failed to hash {}: {err}", path.display());
                }
            }

            let hex = to_hex(&hasher.finalize());
            println!("cargo:rustc-env=PROJECT_HASH={hex}");
        }
    }
}

fn main() {
    setup::build_bypass();
    setup::lua_and_libc();
    setup::offset_bits_cfg();

    // Setup env
    setup::env::target();
    let os = setup::env::target_os();
    setup::env::target_arch();
    setup::env::build_time();
    let ver = setup::env::rustc_version();
    setup::env::cargo_version();
    setup::env::commit();
    setup::env::libc_version(&os);
    setup::env::project_hash();

    // Check is nightly
    let is_nightly = ver.contains("nightly") || ver.contains("dev");
    assert!(
        is_nightly,
        "\x1b[31;1mTo compile and work with the lazyfetch source code, the nightly version of the compiler is required\x1b[0m"
    );

    let (raw, encoded) = setup::compress_logos();
    #[allow(clippy::cast_precision_loss)]
    {
        println!(
            "cargo:warning={}b -> {}b = {:.02}%",
            raw,
            encoded,
            (encoded as f64 / raw as f64).mul_add(-100.0, 100.0)
        );
    }

    setup::generate_logos();
    setup::generate_help();
}
