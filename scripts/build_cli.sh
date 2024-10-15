#!/bin/bash
DIR="$( cd "$( dirname "$0" )" && pwd )"
pushd "$DIR/.."
cargo build --bin rwf-cli
cp target/debug/rwf-cli ~/.cargo/bin/rwf-cli
