use std::{
    fs, io,
    os::unix::net::{UnixListener, UnixStream},
    path::Path,
};

pub fn bind_socket(socket_path: &str) -> io::Result<UnixListener> {
    // ソケットファイルが残っているか
    if Path::new(socket_path).exists() {
        // 接続できるなら既に起動中
        if UnixStream::connect(socket_path).is_ok() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "Application is already running",
            ));
        }

        // 接続できなければ古いソケットなので削除
        fs::remove_file(socket_path)?;
    }

    UnixListener::bind(socket_path)
}
