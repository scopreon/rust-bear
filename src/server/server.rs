use std::io::{Read, Write};
use std::os::unix::net::UnixListener;
use std::{env, fs};

use std::os::unix::net::UnixStream;
use std::thread;

pub mod minibear {
    pub mod schema {
        include!(concat!(env!("OUT_DIR"), "/minibear.schema.rs"));
    }
}

use minibear::schema;
use prost::Message;

fn handle_client(mut stream: UnixStream) -> schema::SearchRequest {
    let mut buf = Vec::new();
    let _ = stream.read_to_end(&mut buf);

    let req: schema::SearchRequest = schema::SearchRequest::decode(&buf[..]).unwrap();

    println!("got: {:?}", req.args);

    req
}

fn main() -> std::io::Result<()> {
    // let path = "/Users/saulcoops/Programming/rust-bear/mysoc.sock";
    let path = env::var("_MINIBEAR_SOCKET").expect("expecteed");

    let _ = fs::remove_file(&path);
    let listener = UnixListener::bind(&path)?;

    let mut threads: Vec<thread::JoinHandle<schema::SearchRequest>> = Vec::new();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                threads.push(thread::spawn(|| handle_client(stream)));
            }
            Err(err) => {
                break;
            }
        }
    }
    let _results: Vec<schema::SearchRequest> = threads
        .into_iter()
        .map(|t| t.join().expect("Thread panicked"))
        .collect();

    Ok(())
}
