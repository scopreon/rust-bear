use std::fs;
use std::io::prelude::*;
use std::os::unix::net::UnixStream;

use libc::EOF;
pub mod snazzy {
    pub mod items {
        include!(concat!(env!("OUT_DIR"), "/snazzy.items.rs"));
    }
}
use prost::Message;
use snazzy::items;
fn main() -> std::io::Result<()> {
    let path = "/Users/saulcoops/Programming/rust-bear/mysoc.sock";
    let mut stream = UnixStream::connect(path)?;
    let mut buffer: Vec<u8> = Vec::new();

    // buffer.extend_from_slice("test 123\n123".as_bytes());
    let mut req: items::SearchRequest = items::SearchRequest { args: Vec::new() };
    req.args.push("test".to_owned());
    req.args.push("test2".to_owned());
    req.encode(&mut buffer);

    stream.write_all(&buffer[..]);
    stream.shutdown(std::net::Shutdown::Both);
    // Remove stale socket from previous run
    // let _ = fs::remove_file(path);
    Ok(())
}
