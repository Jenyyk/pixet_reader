use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());

    let target = std::env::var("TARGET").unwrap();
    let target_first = target.split("-").next().unwrap();

    println!("cargo:warning=using lib {}", target_first);

    println!(
        "cargo:rustc-link-search={}",
        manifest_dir
            .join(format!("lib/{target_first}/"))
            .to_str()
            .unwrap()
    );
    println!("cargo:rustc-link-lib=dylib=pxcore");

    // aarch64 target requires libzest.so during compile time
    if target_first == "aarch64" {
        println!("cargo:rustc-link-lib=dylib=zest")
    }
}
