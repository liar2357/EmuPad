use std::{
    fs,
    io::{BufRead, BufReader},
    os::unix::net::UnixListener,
    str::FromStr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::{
    app::state::AppState, coordinate::transform::Transform, error::Error, event::log::Logger,
    socket::structs::SocketCommand,
};

pub fn start_socket_server(
    listener: UnixListener,
    app_state: Arc<AppState>,
    is_shutdown: Arc<AtomicBool>,
    socket_path: String,
    logger: Arc<Logger>,
) -> Result<(), Error> {
    logger.trace("Thread Steat: Socket");
    logger.info(format!("Listening: {}", socket_path));

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(s) => s,
            Err(e) => {
                logger.error(format!("Accept error: {e}"));
                continue;
            }
        };

        let mut reader = BufReader::new(stream);

        let mut line = String::new();

        if let Err(e) = reader.read_line(&mut line) {
            logger.error(format!("Read error: {e}"));
            continue;
        }

        let trimed = line.trim();
        logger.info(format!("Socket Recieved: {}", trimed));

        let cmd = match SocketCommand::from_str(trimed) {
            Ok(v) => v,
            Err(_) => {
                logger.error(format!("Unknown command: {}", line.trim()));
                continue;
            }
        };

        let is_shutdown_recived = matches!(cmd, SocketCommand::ShutdownApp);

        if let Err(e) = socket_command_handler(cmd, &app_state, Arc::clone(&is_shutdown)) {
            logger.error(e);
            break;
        }

        if is_shutdown_recived {
            break;
        }
    }

    let _ = fs::remove_file(&socket_path);

    logger.trace("Thread End: Socket");
    Ok(())
}

fn socket_command_handler(
    cmd: SocketCommand,
    app_state: &Arc<AppState>,
    is_shutdown: Arc<AtomicBool>,
) -> Result<(), Error> {
    match cmd {
        SocketCommand::SetDirection(n) => match Transform::from_u8(n) {
            Some(t) => {
                app_state.set_transform_direction(t);
            }
            None => {
                return Err(Error::Control("invalid direction number".to_string()));
            }
        },
        SocketCommand::SwitchProfile(i) => {
            app_state.switch_profile(i)?;
        }
        SocketCommand::ReloadApp => {
            app_state.reload_config()?;
        }
        SocketCommand::ShutdownApp => {
            is_shutdown.store(true, Ordering::Relaxed);
        }
    }

    Ok(())
}
