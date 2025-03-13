use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

fn var(s: &str) -> String {
    env::var(s).unwrap()
}

fn rerun_if_changed(path: impl AsRef<Path>) {
    println!("cargo:rerun-if-changed={}", path.as_ref().display());
}

fn main() {
    let out_path = PathBuf::from(var("OUT_DIR"));

    let tsgo_dir = PathBuf::from(var("CARGO_MANIFEST_DIR")).join("..");

    let mut go = Command::new("go");
    go.current_dir(&tsgo_dir)
        .args(["build", "-buildmode=c-archive", "-o"])
        .arg(out_path.join("libtsgo.a"))
        .arg("./cmd/libr");

    go.status().unwrap();
    std::fs::copy(tsgo_dir.join("./cmd/libr/prog.h"), out_path.join("prog.h")).unwrap();

    let bindings = bindgen::Builder::default()
        .header(out_path.join("libtsgo.h").to_string_lossy())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // .blocklist_item("__.*")
        .allowlist_type("[a-z]+")
        .allowlist_function("[A-Z].+")
        .allowlist_function("free")
        .generate()
        .unwrap();

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .unwrap();

    [
        tsgo_dir.join("./cmd/libr/main.go"),
        tsgo_dir.join("./cmd/libr/prog.h"),
    ]
    .into_iter()
    .for_each(rerun_if_changed);

    println!(
        "cargo:rustc-link-search=native={}",
        out_path.to_string_lossy()
    );
    println!("cargo:rustc-link-lib=static=tsgo")
}
