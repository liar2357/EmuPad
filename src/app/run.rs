use crate::{
    app::state::AppState,
    config::loader::resolve_config_path,
    error::Error,
    event::{hot_reload::watch_file_change, log::Logger},
    socket::{bind::bind_socket, handler::start_socket_server},
};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

pub fn run(
    join_hundlers: &mut Vec<JoinHandle<Result<(), Error>>>,
    custom_path: &Option<PathBuf>,
    logger: &Arc<Logger>,
) -> Result<(), Error> {
    let socket_path = format!(
        "{}/{}.sock",
        std::env::var("XDG_RUNTIME_DIR").unwrap(),
        env!("CARGO_PKG_NAME")
    );

    let spc = socket_path.clone();

    let listener = match bind_socket(&socket_path) {
        Ok(listener) => listener,
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            logger.error("emu-pad is already running.");
            return Err(Error::Process("emu-pad is already running.".to_string()));
        }
        Err(e) => {
            logger.error(e);
            return Err(Error::Process("failer create socket".to_string()));
        }
    };

    let is_shutdown = Arc::new(AtomicBool::new(false));

    let config_path = resolve_config_path(custom_path, logger);

    let app_state = Arc::new(AppState::new(&config_path, logger)?);

    let app_state_c = Arc::clone(&app_state);
    let is_shutdown_c = Arc::clone(&is_shutdown);
    let logger_c = Arc::clone(logger);
    join_hundlers.push(thread::spawn(move || {
        start_socket_server(listener, app_state_c, is_shutdown_c, spc, logger_c)
    }));

    let app_state_c = Arc::clone(&app_state);
    let config_path_c = config_path.clone();
    let is_shutdown_c = Arc::clone(&is_shutdown);
    let logger_c = Arc::clone(logger);
    join_hundlers.push(thread::spawn(move || {
        watch_file_change(app_state_c, config_path_c, is_shutdown_c, logger_c)
    }));

    while !is_shutdown.load(Ordering::Relaxed) {
        for frame in app_state.poll()? {
            logger.trace(format!("{frame:?}"));

            for event in app_state.process(&frame) {
                logger.info(format!("{event:?}"));
                app_state.emit(event)?;
            }
        }

        thread::sleep(Duration::from_millis(app_state.get_polling_rate()));
    }

    logger.info("brake main loop");

    Ok(())
}
