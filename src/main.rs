use clap::Parser;
use emu_pad::{
    app::run::run,
    error::Error,
    event::{comandline::Args, log::Logger, signal::run_signal_thread},
};
use signal_hook::{consts::SIGINT, iterator::Signals};
use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};

fn main() -> Result<(), Error> {
    let args = Args::parse();
    let logger = Arc::new(Logger::new(args.verbose));

    let mut join_hundlers: Vec<JoinHandle<Result<(), Error>>> = vec![];

    let mut signals = Signals::new([SIGINT]).unwrap();
    let handle = signals.handle();

    let logger_c = Arc::clone(&logger);
    join_hundlers.push(thread::spawn(move || {
        run_signal_thread(&mut signals, logger_c)
    }));

    run(&mut join_hundlers, &args.config, &logger)?;

    handle.close();

    for jh in join_hundlers {
        if let Err(e) = jh.join() {
            logger.error(format!("{:?}", e));
        }
    }

    logger.info("Successfuly Shutdown!");

    Ok(())
}
