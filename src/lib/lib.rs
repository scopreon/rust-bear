use libc::{c_char, c_int, dlsym, pid_t, posix_spawn_file_actions_t, posix_spawnattr_t, RTLD_NEXT};
use prost::Message;
use rust_bear_proto::minibear::schema;
use std::env;
use std::ffi::CStr;
use std::fmt;
use std::io::{self, Write};
use std::os::unix::net::UnixStream;

//   int posix_spawn(pid_t *restrict pid, const char *restrict path,
//                    const posix_spawn_file_actions_t *restrict file_actions,
//                    const posix_spawnattr_t *restrict attrp,
//                    char *const argv[restrict],
//                    char *const envp[restrict]);

type ExecveFn =
    fn(path: *const c_char, argv: *const *mut c_char, envp: *const *mut c_char) -> c_int;
type ExecveatFn = fn(
    dfd: c_int,
    path: *const c_char,
    argv: *const *mut c_char,
    envp: *const *mut c_char,
    flags: c_int,
) -> c_int;

type PosixSpawnFn = fn(
    pid: *const pid_t,
    path: *const c_char,
    file_actions: *const posix_spawn_file_actions_t,
    attrp: *const posix_spawnattr_t,
    argv: *const *mut c_char,
    envp: *const *mut c_char,
) -> c_int;

#[no_mangle]
unsafe extern "C" fn posix_spawn(
    pid: *const pid_t,
    path: *const c_char,
    file_actions: *const posix_spawn_file_actions_t,
    attrp: *const posix_spawnattr_t,
    argv: *const *mut c_char,
    envp: *const *mut c_char,
) -> c_int {
    let val = dlsym(RTLD_NEXT, b"posix_spawn\0".as_ptr() as *const c_char);
    let function: PosixSpawnFn = std::mem::transmute(val);

    let com = Command::new(argv);
    let mut connecton = match get_uds_connection() {
        Ok(con) => con,
        Err(_) => {
            return function(pid, path, file_actions, attrp, argv, envp);
        }
    };
    let current_path = match env::current_dir() {
        Ok(path) => path,
        Err(_) => return function(pid, path, file_actions, attrp, argv, envp),
    };

    let val: schema::SearchRequest = schema::SearchRequest {
        args: com.argv,
        path: current_path.to_string_lossy().to_string(),
    };

    let _ = connecton.write_all(&val.encode_to_vec()[..]);
    let _ = connecton.shutdown(std::net::Shutdown::Both);
    function(pid, path, file_actions, attrp, argv, envp)
}

#[no_mangle]
unsafe extern "C" fn execve(
    path: *const c_char,
    argv: *const *mut c_char,
    envp: *const *mut c_char,
) -> c_int {
    let val = dlsym(RTLD_NEXT, b"execve\0".as_ptr() as *const c_char);
    let function: ExecveFn = std::mem::transmute(val);

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

// int dfd 	const char *filename 	const char *const *argv 	const char *const *envp 	int flags

#[no_mangle]
unsafe extern "C" fn execveat(
    dfd: c_int,
    path: *const c_char,
    argv: *const *mut c_char,
    envp: *const *mut c_char,
    flags: c_int,
) -> c_int {
    let val = dlsym(RTLD_NEXT, b"execveat\0".as_ptr() as *const c_char);
    let function: ExecveatFn = std::mem::transmute(val);

    let com = Command::new(argv);
    let mut connecton = match get_uds_connection() {
        Ok(con) => con,
        Err(_) => {
            return function(dfd, path, argv, envp, flags);
        }
    };
    let current_path = match env::current_dir() {
        Ok(path) => path,
        Err(_) => return function(dfd, path, argv, envp, flags),
    };

    let val: schema::SearchRequest = schema::SearchRequest {
        args: com.argv,
        path: current_path.to_string_lossy().to_string(),
    };

    let _ = connecton.write_all(&val.encode_to_vec()[..]);
    let _ = connecton.shutdown(std::net::Shutdown::Both);
    function(dfd, path, argv, envp, flags)
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
