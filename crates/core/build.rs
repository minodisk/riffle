use std::fmt::Write;
use std::path::{Path, PathBuf};

fn main() {
    let dir = Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("i18n");
    println!("cargo:rerun-if-changed=i18n");
    let mut files: Vec<(String, PathBuf)> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .map(|path| {
            let code = path.file_stem().unwrap().to_str().unwrap().to_string();
            (code, path)
        })
        .collect();
    files.sort();
    let mut out = String::from("&[\n");
    for (code, path) in &files {
        println!("cargo:rerun-if-changed={}", path.display());
        let path = path.to_str().unwrap();
        writeln!(out, "    ({code:?}, include_str!({path:?})),").unwrap();
    }
    out.push_str("]\n");
    let dest = Path::new(&std::env::var("OUT_DIR").unwrap()).join("i18n.rs");
    std::fs::write(dest, out).unwrap();
}
