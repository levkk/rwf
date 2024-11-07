use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=src/bindings.c");
    println!("cargo:rerun-if-changed=src/bindings.h");

    // let bindings = bindgen::Builder::default()
    //     .header("src/bindings.h")
    //     .parse_callbacks(Box::new(bindgen::CargoCallbacks))
    //     .generate()
    //     .expect("bindgen");

    // let out_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("src/bindings.rs");
    // bindings.write_to_file(out_path).expect("write bindings");

    cc::Build::new()
        .file("src/bindings.c")
        .flag("-I/usr/include/ruby-3.3.0")
        .flag("-I/usr/include/ruby-3.3.0/x86_64-linux")
        .compile("rwf_ruby");
}
