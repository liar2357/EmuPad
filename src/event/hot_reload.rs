use crate::{app::state::AppState, error::Error, event::log::Logger};
use notify::{Config as NotifyConfig, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

const DEBOUNCE_DURATION: Duration = Duration::from_millis(100);
const CHECK_INTERVAL: Duration = Duration::from_millis(10);

pub fn watch_file_change(
    app_state: Arc<AppState>,
    config_path: Option<PathBuf>,
    is_shutdown: Arc<AtomicBool>,
    logger: Arc<Logger>,
) -> Result<(), Error> {
    logger.trace("Thread Start: Change");

    let Some(config_path) = config_path else {
        return Err(Error::Config("config path is not present".to_string()));
    };

    let config_dir = config_path
        .parent()
        .expect("config path has no parent")
        .to_path_buf();

    logger.info(format!("file change monitor: {:?}", config_dir));

    // 最後に変更を検知した時刻
    let last_change = Arc::new(Mutex::new(None::<Instant>));
    let last_change_callback = Arc::clone(&last_change);

    let mut watcher = RecommendedWatcher::new(
        move |result: notify::Result<notify::Event>| {
            let Ok(event) = result else {
                return;
            };

            let is_changed = event.paths.iter().any(|path| path == &config_path);

            if !is_changed {
                return;
            }

            match event.kind {
                EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                    if let Ok(mut last_change) = last_change_callback.lock() {
                        // 最後に変更された時刻を更新する
                        *last_change = Some(Instant::now());
                    }
                }
                _ => {}
            }
        },
        NotifyConfig::default(),
    )?;

    watcher.watch(&config_dir, RecursiveMode::NonRecursive)?;

    loop {
        if is_shutdown.load(Ordering::Relaxed) {
            break;
        }

        let should_reload = {
            let Ok(last_change) = last_change.lock() else {
                thread::sleep(CHECK_INTERVAL);
                continue;
            };

            last_change
                .map(|changed_at| changed_at.elapsed() >= DEBOUNCE_DURATION)
                .unwrap_or(false)
        };

        if should_reload {
            if let Ok(mut last_change) = last_change.lock() {
                *last_change = None;
            }

            logger.info("file change detected");
            app_state.reload_config()?;
        }

        thread::sleep(CHECK_INTERVAL);
    }

    logger.trace("Thread End: Change");
    Ok(())
}
