use libc::{c_char, c_int, dlsym, fcntl, FD_CLOEXEC, F_SETFD, RTLD_NEXT};
use std::env;
use std::ffi::CStr;
use std::fmt;
use std::io::{self, Write};
use std::os::unix::io::AsRawFd;
use std::os::unix::net::UnixStream;

type ExecFn = fn(path: *const c_char, argv: *const *mut c_char, envp: *const *mut c_char) -> c_int;

pub mod minibear {
    pub mod schema {
        include!(concat!(env!("OUT_DIR"), "/minibear.schema.rs"));
    }
}

use minibear::schema;
use prost::Message;

#[no_mangle]
unsafe extern "C" fn execve(
    path: *const c_char,
    argv: *const *mut c_char,
    envp: *const *mut c_char,
) -> c_int {
    let val = dlsym(RTLD_NEXT, b"execve\0".as_ptr() as *const c_char);
    let function: ExecFn = std::mem::transmute(val);

    let com = Command::new(argv);
    let mut connecton = match get_uds_connection() {
        Ok(con) => con,
        Err(_) => {
            return function(path, argv, envp);
        }
    };
    let current_path = match env::current_dir() {
        Ok(path) => path,
        Err(_) => return function(path, argv, envp),
    };

    let val: schema::SearchRequest = schema::SearchRequest {
        args: com.argv,
        path: current_path.to_string_lossy().to_string(),
    };

    let _ = connecton.write_all(&val.encode_to_vec()[..]);
    let _ = connecton.shutdown(std::net::Shutdown::Both);
    function(path, argv, envp)
}

fn get_uds_connection() -> io::Result<UnixStream> {
    let path =
        env::var("_MINIBEAR_SOCKET").map_err(|e| io::Error::new(io::ErrorKind::NotFound, e))?;

    UnixStream::connect(path)
}

struct Command {
    argv: Vec<String>,
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.argv.join(" "))
    }
}

impl Command {
    unsafe fn new(commands: *const *mut c_char) -> Command {
        let mut data: Vec<String> = Vec::new();
        let mut pointer = commands;
        while !pointer.read().is_null() {
            let cstring = CStr::from_ptr(pointer.read());
            data.push(cstring.to_string_lossy().into_owned());
            pointer = pointer.add(1);
        }

        Command { argv: data }
    }
}
