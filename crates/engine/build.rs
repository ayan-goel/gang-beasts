use sha2::{Digest, Sha256};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn rust_sources(directory: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).expect("engine sources must be readable") {
        let path = entry.expect("source entry must be readable").path();
        if path.is_dir() {
            rust_sources(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let root = manifest.parent().unwrap().parent().unwrap();
    let mut files = vec![
        root.join("Cargo.toml"),
        root.join("Cargo.lock"),
        root.join("rust-toolchain.toml"),
        root.join("config/character.json"),
        root.join("config/motors.json"),
        root.join("config/balance.json"),
        root.join("config/locomotion.json"),
        root.join("config/hands.json"),
        root.join("config/grip.json"),
        root.join("assets/maps/skyline/map.json"),
        root.join("config/impact.json"),
        root.join("config/consciousness.json"),
        root.join("config/locomotion-motors.json"),
        root.join("config/standing-motors.json"),
        manifest.join("Cargo.toml"),
        manifest.join("build.rs"),
    ];
    println!("cargo:rerun-if-changed=src");
    rust_sources(&manifest.join("src"), &mut files);
    files.sort();
    let mut hash = Sha256::new();
    let compiler = std::process::Command::new(env::var("RUSTC").unwrap())
        .arg("--version")
        .output()
        .expect("compiler identity must be available");
    assert!(compiler.status.success(), "compiler version query failed");
    hash.update((compiler.stdout.len() as u64).to_le_bytes());
    hash.update(&compiler.stdout);
    for file in files {
        println!("cargo:rerun-if-changed={}", file.display());
        let name = file
            .strip_prefix(root)
            .unwrap()
            .to_str()
            .unwrap()
            .replace('\\', "/");
        let contents = fs::read(&file).expect("build fingerprint source must exist");
        // Length-prefix paths and contents so concatenation cannot make two
        // distinct source trees share the same fingerprint input.
        hash.update((name.len() as u64).to_le_bytes());
        hash.update(name.as_bytes());
        hash.update((contents.len() as u64).to_le_bytes());
        hash.update(contents);
    }
    println!("cargo:rustc-env=BRAWLER_ENGINE_HASH={:x}", hash.finalize());
    println!(
        "cargo:rustc-env=BRAWLER_BUILD_TARGET={}",
        env::var("TARGET").unwrap()
    );
    println!(
        "cargo:rustc-env=BRAWLER_BUILD_PROFILE={}",
        env::var("PROFILE").unwrap()
    );
}
