# Rwf ruby

`rwf-ruby` contains Rust bindings for running Ruby applications built on top of [Rack](https://github.com/rack/rack). While there exists other projects that bind Ruby to Rust in a generic way,
running arbitrary Ruby code inside Rust requires wrapping the `ruby_exec_node` directly.

This project is experimental, and needs additional testing to ensure production stability. The bindings are written in C, see [src/bindings.c](src/bindings.c).
