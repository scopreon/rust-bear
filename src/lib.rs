use libc::STDERR_FILENO;
use libc::{c_char, c_int, c_void, dlsym, RTLD_NEXT};
use std::env;
use std::ffi::CStr;
use std::fmt;
use std::fs;
use std::io;
use std::os::unix::io::AsRawFd;

type ExecFn = fn(path: *const c_char, argv: *const *mut c_char, envp: *const *mut c_char) -> c_int;

#[no_mangle]
pub unsafe extern "C" fn execve(
    path: *const c_char,
    argv: *const *mut c_char,
    envp: *const *mut c_char,
) -> c_int {
    let file = match get_outfile() {
        Ok(file) => file,
        Err(_) => {
            return -1;
        }
    };
    let com = Command::new(argv);
    let msg = format!("Command: {}\n", com);

    // libc::write(STDERR_FILENO, msg2.as_ptr() as *const c_void, msg2.len());
    libc::write(file.as_raw_fd(), msg.as_ptr() as *const c_void, msg.len());
    let val = dlsym(RTLD_NEXT, "execve\0".as_ptr() as *const c_char);

    let function: ExecFn = std::mem::transmute(val);
    function(path, argv, envp)
}

fn get_outfile() -> Result<fs::File, io::Error> {
    let filename = env::var("_RUST_BEAR_OUT")
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "ERROR env var not set"))?;
    fs::OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(filename)
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
