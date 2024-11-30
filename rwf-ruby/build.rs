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

    // Github actions workaround. I don't remember if this works or not.
    match Command::new("find")
        .arg("/opt/hostedtoolcache/Ruby")
        .arg("-name")
        .arg("libruby.so")
        .output()
    {
        Ok(output) => {
            let lib = String::from_utf8_lossy(&output.stdout)
                .to_string()
                .trim()
                .to_string();
            let lib = lib.split("\n").next().unwrap_or("").trim();
            if !lib.is_empty() {
                build.flag(format!("-L{}", lib));
            }
        }

        Err(_) => (),
    };

    build.file("src/bindings.c").compile("rwf_ruby");
}
