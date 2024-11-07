use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src/bindings.c");
    println!("cargo:rerun-if-changed=src/bindings.h"); // Bindings are generated manually because bindgen goes overboard with ruby.h

    let output = Command::new("ruby")
        .arg("headers.rb")
        .output()
        .expect("Is ruby installed on your system?")
        .stdout;
    let flags = String::from_utf8_lossy(&output).to_string();

    let mut build = cc::Build::new();

    for flag in flags.split(" ") {
        build.flag(flag);
    }

    build.file("src/bindings.c").compile("rwf_ruby");
}
