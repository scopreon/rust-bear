use libc::{c_char, c_int, dlsym, pid_t, posix_spawn_file_actions_t, posix_spawnattr_t, RTLD_NEXT};
use prost::Message;
use rust_bear_proto::minibear::schema;
use std::env;
use std::ffi::CStr;
use std::fmt;
use std::io::{self, Write};
use std::os::unix::net::UnixStream;
use which::which;

fn get_full_compiler_path(exec: &str) -> String {
    match which(exec) {
        Ok(path) => path
            .to_str()
            .map(|s| s.to_string())
            .unwrap_or("default".to_string()),
        Err(_) => "default".to_string(),
    }
}

macro_rules! intercept {
    (
        fn $name:ident(
            $( $arg:ident : $argty:ty ),* $(,)?
        ) -> $ret:ty {
            capture($argv:ident);
        }

    ) => {
        #[no_mangle]
        unsafe extern "C" fn $name( $( $arg : $argty ),* ) -> $ret{
            type Type =
            fn($( $argty ),*) -> $ret;
                let val = dlsym(RTLD_NEXT, format!("{}\0",stringify!($name)).as_bytes().as_ptr() as *const c_char);
            let function: Type = std::mem::transmute(val);

            let _ = send_command(Command::new($argv));
            function($($arg),*)
        }
    };
}

intercept!(
    fn posix_spawn(
        pid: *const pid_t,
        path: *const c_char,
        file_actions: *const posix_spawn_file_actions_t,
        attrp: *const posix_spawnattr_t,
        argv: *const *mut c_char,
        envp: *const *mut c_char,
    ) -> c_int {
        capture(argv);
    }
);
intercept!(
    fn execve(path: *const c_char, argv: *const *mut c_char, envp: *const *mut c_char) -> c_int {
        capture(argv);
    }
);

intercept!(
    fn execveat(
        dfd: c_int,
        path: *const c_char,
        argv: *const *mut c_char,
        envp: *const *mut c_char,
        flags: c_int,
    ) -> c_int {
        capture(argv);
    }
);

fn send_command(command: Command) -> io::Result<()> {
    let mut connecton = get_uds_connection()?;
    let current_path = env::current_dir()?;
    let val: schema::SearchRequest = schema::SearchRequest {
        args: command.argv,
        path: current_path.to_string_lossy().to_string(),
    };

    let _ = connecton.write_all(&val.encode_to_vec()[..])?;
    let _ = connecton.shutdown(std::net::Shutdown::Both)?;
    Ok(())
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
        let mut counter = 0;
        while !pointer.read().is_null() {
            let cstring = CStr::from_ptr(pointer.read());
            if counter == 0 {
                data.push(get_full_compiler_path(cstring.to_str().unwrap()));
            } else {
                data.push(cstring.to_string_lossy().into_owned());
            }

            pointer = pointer.add(1);
            counter += 1;
        }

        Command { argv: data }
    }
}
