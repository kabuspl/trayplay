use std::process::Command;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    Command::new("bash")
        .arg("lrelease.sh")
        .current_dir(manifest_dir)
        .status()
        .unwrap();

    let qt_include_path = std::env::var("DEP_QT_INCLUDE_PATH").unwrap();

    let mut config = cpp_build::Config::new();
    for f in std::env::var("DEP_QT_COMPILE_FLAGS")
        .unwrap()
        .split_terminator(";")
    {
        config.flag(f);
    }

    config.include(format!("{}/QtCore", qt_include_path));
    config.include(format!("{}/QtGui", qt_include_path));
    config.include(format!("{}/QtQuick", qt_include_path));
    config.include(format!("{}/QtQml", qt_include_path));
    config.include(format!("{}/QtQuickControls2", qt_include_path));

    println!("cargo:rerun-if-changed=src/settings.rs");
    println!("cargo:rerun-if-changed=ui/lang");

    config.include(&qt_include_path).build("src/main.rs");
}
