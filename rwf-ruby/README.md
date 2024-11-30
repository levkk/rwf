# Rwf ruby

[![Documentation](https://img.shields.io/badge/documentation-blue?style=flat)](https://levkk.github.io/rwf/)
[![Latest crate](https://img.shields.io/crates/v/rwf-ruby.svg)](https://crates.io/crates/rwf-ruby)
[![Reference docs](https://img.shields.io/docsrs/rwf-ruby)](https://docs.rs/rwf-ruby)

`rwf-ruby` contains Rust bindings for running Ruby applications built on top of [Rack](https://github.com/rack/rack). While there exists other projects that bind Ruby to Rust in a generic way,
running arbitrary Ruby code inside Rust requires wrapping `ruby_exec_node` directly.

This project is experimental, and needs additional testing to ensure production stability. The bindings are written in C, see [src/bindings.c](src/bindings.c).
