use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let src = manifest.join("source");

    // Recompile on source changes
    println!("cargo:rerun-if-changed=src/bindings.cpp");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/bridge.h");
    println!("cargo:rerun-if-changed=source");

    // Collect all .h files for rerun triggers
    let walk_dir = src.join("**/*.h");
    for entry in glob::glob(walk_dir.to_str().unwrap()).unwrap() {
        if let Ok(p) = entry {
            println!("cargo:rerun-if-changed={}", p.display());
        }
    }

    // Find glm include path
    let brew_glm = "/opt/homebrew/include/glm";
    let local_glm = "/usr/local/include/glm";
    let mut glm_include: Option<&'static str> = None;
    if std::path::Path::new(brew_glm).exists() {
        glm_include = Some("/opt/homebrew/include/glm");
    } else if std::path::Path::new(local_glm).exists() {
        glm_include = Some("/usr/local/include/glm");
    }

    // Compile bindings.cpp as a static library
    let mut cfg = cc::Build::new();
    cfg.file("src/bindings.cpp")
       .include(".")
       .include(src.to_str().unwrap())
       .include(src.join("include").to_str().unwrap())
       .std("c++20")
       .flag_if_supported("-Wno-unused-variable")
       .flag_if_supported("-Wno-sign-compare")
       .flag_if_supported("-Wno-unused-parameter")
       .flag_if_supported("-Wno-unused-function")
       .flag_if_supported("-Wno-undefined-inline")
       .flag_if_supported("-Wno-instantiation-after-selected");

    if let Some(p) = glm_include {
        cfg.include(p);
    }

    cfg.compile("soilmachine_bridge");
    // Link C++ standard library
    println!("cargo:rustc-link-lib=dylib=c++");
    println!("cargo:rustc-link-lib=static=soilmachine_bridge");
}
