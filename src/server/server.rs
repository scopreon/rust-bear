mod helpers;

use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::io::{self, Read, Write};
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, sleep, sleep_ms};
use std::time::Duration;
use std::{env, fs};

pub mod minibear {
    pub mod schema {
        include!(concat!(env!("OUT_DIR"), "/minibear.schema.rs"));
    }
}

const header: &[&str] = &[".cpp", ".cxx", ".cc", ".c"];

fn extract_source(command: &schema::SearchRequest) -> Vec<&str> {
    let mut files: Vec<&str> = Vec::new();
    let mut iter = command.args.iter();
    while let Some(arg) = iter.next() {
        for suffix in header {
            if arg.ends_with(suffix) {
                files.push(arg.as_str());
            }
        }
    }
    files
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_run() {
        let a = vec![
            "-std=c11".into(), // joined
            "-std".into(),     // separate
            "gnu11".into(),
            "file.c".into(),
        ];

        let command = schema::SearchRequest {
            args: a,
            path: "123".to_string(),
        };
        let ret = extract_source(&command);
        println!("{:?}", ret);
    }
}

use minibear::schema;
use prost::Message;

fn handle_client(mut stream: UnixStream) -> schema::SearchRequest {
    let mut buf = Vec::new();
    let _ = stream.read_to_end(&mut buf);

    let req: schema::SearchRequest = schema::SearchRequest::decode(&buf[..]).unwrap();
    println!("got: {:?}", req);

    req
}

// [
//   {
//     "directory" : "/Users/lawrenceolson/Code/Clang-tutorial/",
//     "command" : "clang -c -o input04.o input04.c",
//     "file" : "input04.c"
//   }
// ]
#[derive(Serialize, Deserialize)]
struct CompileCommandsEntry {
    directory: String,
    command: String,
    file: String,
}

fn main() -> std::io::Result<()> {
    let (tx, rx): (mpsc::Sender<bool>, mpsc::Receiver<bool>) = mpsc::channel();
    let handle = thread::spawn(|| server(rx));
    let args: Vec<String> = env::args().collect();
    sleep(Duration::from_secs(2));

    Command::new(&args[1])
        .args(&args[2..])
        // .stdin(Stdio::null())
        // .stdout(Stdio::null())
        // .stderr(Stdio::null())
        .env("LD_PRELOAD", "/workspace/target/debug/librust_bear.so")
        .env("_MINIBEAR_SOCKET", "/workspace/mysock.sock")
        .status()?;
    sleep(Duration::from_secs(2));
    tx.send(true);
    // let mut compile_commands: Vec<CompileCommandsEntry> = Vec::new();
    let mut compile_commands: Vec<CompileCommandsEntry> = Vec::new();

    for result in handle.join().expect("msg")? {
        for file in extract_source(&result) {
            compile_commands.push(CompileCommandsEntry {
                directory: result.path.clone(),
                command: result.args.join(" "),
                file: file.to_string(),
            });
        }
    }
    let serialized = serde_json::to_string_pretty(&compile_commands).unwrap();

    // Prints serialized = {"x":1,"y":2}
    println!("serialized = {}", serialized);
    Ok(())
    // let path = "/Users/saulcoops/Programming/rust-bear/mysoc.sock";
    // let path = env::var("_MINIBEAR_SOCKET").expect("expecteed");

    // let _ = fs::remove_file(&path);
    // let listener = UnixListener::bind(&path)?;
    // // let mut i = 0;

    // let mut threads: Vec<thread::JoinHandle<schema::SearchRequest>> = Vec::new();
    // let _ = listener.set_nonblocking(true);
    // loop {
    //     sleep(Duration::from_millis(10));
    //     println!("Looking");
    //     match listener.accept() {
    //         Ok(stream) => {
    //             threads.push(thread::spawn(|| handle_client(stream.0)));
    //         }
    //         Err(_) => continue,
    //     }
    // }
    // // for stream in listener.incoming() {
    // //     i += 1;
    // //     if i == 8 {
    // //         break;
    // //     }
    // //     match stream {
    // //         Ok(stream) => {
    // //             threads.push(thread::spawn(|| handle_client(stream)));
    // //         }
    // //         Err(err) => {
    // //             break;
    // //         }
    // //     }
    // // }
    // let _results: Vec<schema::SearchRequest> = threads
    //     .into_iter()
    //     .map(|t| t.join().expect("Thread panicked"))
    //     .collect();

    // let mut compile_commands: Vec<CompileCommandsEntry> = Vec::new();
    // for result in _results {
    //     for file in extract_source(&result) {
    //         compile_commands.push(CompileCommandsEntry {
    //             directory: result.path.clone(),
    //             command: result.args.join(" "),
    //             file: file.to_string(),
    //         });
    //     }
    // }
    // let serialized = serde_json::to_string_pretty(&compile_commands).unwrap();

    // // Prints serialized = {"x":1,"y":2}
    // println!("serialized = {}", serialized);
}

fn server(reciever: Receiver<bool>) -> std::io::Result<Vec<schema::SearchRequest>> {
    // let path = "/Users/saulcoops/Programming/rust-bear/mysoc.sock";
    let path = env::var("_MINIBEAR_SOCKET").expect("expecteed");

    let _ = fs::remove_file(&path);
    let listener = UnixListener::bind(&path)?;
    // let mut i = 0;

    let mut threads: Vec<thread::JoinHandle<schema::SearchRequest>> = Vec::new();
    let _ = listener.set_nonblocking(true);
    loop {
        sleep(Duration::from_millis(10));
        // println!("here");
        if reciever.try_recv().unwrap_or(false) {
            break;
        }
        match listener.accept() {
            Ok(stream) => {
                threads.push(thread::spawn(|| handle_client(stream.0)));
            }
            Err(_) => continue,
        }
    }
    // for stream in listener.incoming() {
    //     i += 1;
    //     if i == 8 {
    //         break;
    //     }
    //     match stream {
    //         Ok(stream) => {
    //             threads.push(thread::spawn(|| handle_client(stream)));
    //         }
    //         Err(err) => {
    //             break;
    //         }
    //     }
    // }
    let _results: Vec<schema::SearchRequest> = threads
        .into_iter()
        .map(|t| t.join().expect("Thread panicked"))
        .collect();
    Ok(_results)

    // let mut compile_commands: Vec<CompileCommandsEntry> = Vec::new();
    // for result in _results {
    //     for file in extract_source(&result) {
    //         compile_commands.push(CompileCommandsEntry {
    //             directory: result.path.clone(),
    //             command: result.args.join(" "),
    //             file: file.to_string(),
    //         });
    //     }
    // }
    // let serialized = serde_json::to_string_pretty(&compile_commands).unwrap();

    // // Prints serialized = {"x":1,"y":2}
    // println!("serialized = {}", serialized);
    // Ok(())
}
