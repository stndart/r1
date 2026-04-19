fn main() {
    let manifest_dir =
        std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("example_host crate should sit under the workspace root");
    let profile = std::env::var("PROFILE").expect("PROFILE");
    let target_dir = std::env::var_os("CARGO_TARGET_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| workspace_root.join("target"));
    let out_dir = target_dir.join(&profile);
    // MSVC import library for a Rust cdylib can land under `deps/` before (or
    // instead of) the copy in the profile root; link both so `example.dll.lib`
    // resolves on the first workspace build.
    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-search=native={}", out_dir.join("deps").display());
}
