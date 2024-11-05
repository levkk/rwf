fn main() {
    println!("cargo:rerun-if-changed=src/bindings.c");

    cc::Build::new()
        .file("src/bindings.c")
        .flag("-I/usr/include/ruby-3.3.0")
        .flag("-I/usr/include/ruby-3.3.0/x86_64-linux")
        .compile("rwf_ruby");
}
