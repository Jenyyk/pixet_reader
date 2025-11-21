use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());

    let target = std::env::var("TARGET").unwrap();
    let mut target_iter = target.split("-");
    let lib_target = match target_iter.next() {
        Some("aarch64") => "aarch64",
        Some("x86_64") => match target_iter.next() {
            Some("unknown") => "x86_64",
            Some("pc") => "windows",
            _ => panic!("Unsupported target"),
        },
        _ => panic!("Unsupported target"),
    };

    println!("cargo:warning=using lib {}", lib_target);

    println!(
        "cargo:rustc-link-search={}",
        manifest_dir
            .join(format!("lib/{lib_target}/"))
            .to_str()
            .unwrap()
    );
    println!("cargo:rustc-link-lib=dylib=pxcore");

    // aarch64 target requires libzest.so during compile time
    if lib_target == "aarch64" {
        println!("cargo:rustc-link-lib=dylib=zest")
    }
}
