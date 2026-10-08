use crate::{
    error::Error,
    event::log::Logger,
    socket::{sender::send_socket_command, structs::SocketCommand},
};
use signal_hook::{consts::SIGINT, iterator::SignalsInfo};
use std::sync::Arc;

pub fn run_signal_thread(signals: &mut SignalsInfo, logger: Arc<Logger>) -> Result<(), Error> {
    logger.trace("Thread Start: Signal");
    for signal in signals.forever() {
        match signal {
            SIGINT => {
                logger.trace("Signal: Received Ctrl+C");
                let _ = send_socket_command(SocketCommand::ShutdownApp.to_string());
            }
            _ => unreachable!(),
        }
    }

    logger.trace("Thread End: Signal");
    Ok(())
}
