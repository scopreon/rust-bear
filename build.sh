#!/usr/bin/env bash
ln -sf Cargo1.toml Cargo.toml
cargo build
ld -r -b binary -o thisshoudlbelinked.o ./target/debug/librust_bear_lib.so
ar rcs thisshoudlbelinked.a thisshoudlbelinked.o
ln -sf Cargo2.toml Cargo.toml
cargo build