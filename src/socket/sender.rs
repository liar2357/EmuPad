use std::{env, io::Write, os::unix::net::UnixStream};

pub fn send_socket_command(cmd: String) -> std::io::Result<()> {
    let socket = format!(
        "{}/{}.sock",
        std::env::var("XDG_RUNTIME_DIR").unwrap(),
        env!("CARGO_PKG_NAME")
    );

    let mut stream = UnixStream::connect(socket)?;
    writeln!(stream, "{cmd}")?;

    Ok(())
}
