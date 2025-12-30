# Compilation database generator

Tried compiling https://github.com/rizsotto/Bear from source, was slightly painful so decided why not re-write in rust

## How it works:
- hook into various sys calls `execve`, `posix_spawn` etc using `LD_PRELOAD`
- host server listening on Unix Domain Socket
- syscall hooks send args to server with `protobuf` format
- generate compile_commands.json file at end

## Usage:

`git clone git@github.com:scopreon/rust-bear.git`

`cargo build --release`

`./target/release/server <command>` 

e.g. `./target/release/server make build-production`

Tada! `compile_commands.json` generated.

## Conclusion

Was fun
