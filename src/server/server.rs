use prost::Message;
use rust_bear_proto::minibear::schema;
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::io::Write;
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::sync::mpsc::{self, Receiver};
use std::thread::{self, sleep};
use std::time::Duration;
use std::{env, fs};
use tempfile;
const SOURCE_HEADERS: &[&str] = &[".cpp", ".cxx", ".cc", ".c"];

fn extract_source(command: &schema::SearchRequest) -> Vec<&str> {
    let mut files: Vec<&str> = Vec::new();
    let mut iter = command.args.iter();
    while let Some(arg) = iter.next() {
        for suffix in SOURCE_HEADERS {
            if arg.ends_with(suffix) {
                files.push(arg.as_str());
            }
        }
    }
    files
}

unsafe extern "C" {
    static _binary___target_debug_librust_bear_lib_so_start: std::ffi::c_char;
    static _binary___target_debug_librust_bear_lib_so_end: std::ffi::c_char;
}

#[cfg(test)]
mod tests {
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

fn handle_client(mut stream: UnixStream) -> Result<schema::SearchRequest, prost::DecodeError> {
    let mut buf = Vec::new();
    let _ = stream.read_to_end(&mut buf);

    let req: schema::SearchRequest = schema::SearchRequest::decode(&buf[..])?;
    println!("got: {:?}", req);

    Ok(req)
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
fn get_socket_file() -> PathBuf {
    Path::new("/tmp").join(format!("minibear_sock{}", process::id()))
}
use std::slice;
fn main() -> std::io::Result<()> {
    let mut f = tempfile::NamedTempFile::new()?;
    unsafe {
        let start = std::ptr::from_ref(&_binary___target_debug_librust_bear_lib_so_start);
        let end = std::ptr::from_ref(&_binary___target_debug_librust_bear_lib_so_end);
        let distance = end as usize - start as usize;

        let i = slice::from_raw_parts(start as *const u8, distance);
        f.write(i)?;
    }
    let (tx, rx): (mpsc::Sender<bool>, mpsc::Receiver<bool>) = mpsc::channel();
    let handle = thread::spawn(|| server(rx));
    let args: Vec<String> = env::args().collect();

    let cdylib_path = f.path();

    sleep(Duration::from_secs(1));

    process::Command::new(&args[1])
        .args(&args[2..])
        .env("LD_PRELOAD", cdylib_path)
        .env("_MINIBEAR_SOCKET", get_socket_file())
        .status()?;
    sleep(Duration::from_secs(2));
    let _ = tx.send(true);
    // let mut compile_commands: Vec<CompileCommandsEntry> = Vec::new();
    let mut compile_commands: Vec<CompileCommandsEntry> = Vec::new();

    for result in handle.join().expect("msg")? {
        for file in extract_source(&result) {
            let directory_path = Path::new(&result.path);
            let file_path = Path::new(file);

            let (final_dir, filename) = if file_path.is_absolute() {
                (
                    file_path
                        .parent()
                        .expect("Dodgy path")
                        .to_string_lossy()
                        .to_string(),
                    file_path
                        .file_name()
                        .expect("Dodgy path")
                        .to_string_lossy()
                        .to_string(),
                )
            } else {
                (
                    directory_path
                        .join(file_path.parent().expect("Dodgy path"))
                        .to_string_lossy()
                        .to_string(),
                    file_path
                        .file_name()
                        .expect("Dodgy path")
                        .to_string_lossy()
                        .to_string(),
                )
            };

            compile_commands.push(CompileCommandsEntry {
                directory: final_dir,
                command: result.args.join(" "),
                file: filename,
            });
        }
    }
    let serialized =
        serde_json::to_string_pretty(&compile_commands).expect("Failed to serialize data.");
    let mut outfile = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open("compile_commands.json")?;

    let _ = outfile.write_all(serialized.as_bytes());
    println!("serialized = {}", serialized);
    Ok(())
}

fn server(reciever: Receiver<bool>) -> std::io::Result<Vec<schema::SearchRequest>> {
    let path = get_socket_file();

    let _ = fs::remove_file(&path);
    let listener = UnixListener::bind(&path)?;

    let mut threads: Vec<thread::JoinHandle<schema::SearchRequest>> = Vec::new();
    let _ = listener.set_nonblocking(true);
    loop {
        sleep(Duration::from_millis(10));
        if reciever.try_recv().unwrap_or(false) {
            break;
        }
        match listener.accept() {
            Ok(stream) => {
                threads.push(thread::spawn(|| {
                    handle_client(stream.0).expect("Weird stuff happened")
                }));
            }
            Err(_) => continue,
        }
    }
    let _results: Vec<schema::SearchRequest> = threads
        .into_iter()
        .map(|t| t.join().expect("Thread panicked"))
        .collect();
    Ok(_results)
}
