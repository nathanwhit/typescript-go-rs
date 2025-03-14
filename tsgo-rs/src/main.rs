use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};
use tsgo_rs::CompilerHost;

static LOG: LazyLock<bool> = LazyLock::new(|| std::env::var("RUST_LOG").is_ok());

struct MyCompilerHost;

impl CompilerHost for MyCompilerHost {
    fn default_library_path(&self) -> String {
        "".into()
    }

    fn get_current_directory(&self) -> String {
        std::env::current_dir()
            .unwrap()
            .to_string_lossy()
            .to_string()
    }

    fn new_line(&self) -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("\n")
    }

    fn get_source_file_text(&self, file_name: &str, path: &str, language_version: i32) -> String {
        if *LOG {
            eprintln!("get_source_file_text(\"{file_name}\", \"{path}\", {language_version})");
        }
        let path = Path::new(path);
        if path.exists() {
            std::fs::read_to_string(path).unwrap()
        } else {
            "".into()
        }
    }
}

fn main() {
    let diagnostics = tsgo_rs::get_diagnostics(
        MyCompilerHost,
        Path::new("./tsconfig.json"),
        vec![PathBuf::from("./foo.ts")],
    );

    eprintln!("diagnostics: {diagnostics:#?}");
}
