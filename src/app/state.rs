use crate::{
    config::{loader::load_config, structs::Profile},
    coordinate::transform::Transform,
    device::touchscreen::Touchscreen,
    error::Error,
    event::log::Logger,
    gesture::recognizer::GestureRecognizer,
    input::frame::TouchFrame,
    output::{event::MouseEvent, mouse::VirtualMouse},
};
use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

pub struct AppState {
    touchscreen: Mutex<Touchscreen>,
    recognizer: Mutex<GestureRecognizer>,
    mouse: Mutex<VirtualMouse>,
    polling_rate_ms: AtomicU64,
    profile: Mutex<Profile>,
    config_index: Mutex<usize>,
    custom_config_path: Option<PathBuf>,
    logger: Arc<Logger>,
}
impl AppState {
    pub fn new(config_path: &Option<PathBuf>, logger: &Arc<Logger>) -> Result<Self, Error> {
        let profile = load_config(config_path, logger);

        let touchscreen = Mutex::new(Touchscreen::open(&profile.configs[0].device, logger)?);

        let recognizer = Mutex::new(GestureRecognizer::new(&profile.configs[0].gesture)?);

        let mouse = Mutex::new(VirtualMouse::new()?);

        let polling_rate_ms = AtomicU64::new(profile.configs[0].device.polling_rate_ms);

        let profile = Mutex::new(profile);

        Ok(Self {
            touchscreen,
            recognizer,
            mouse,
            polling_rate_ms,
            profile,
            config_index: Mutex::new(0),
            custom_config_path: config_path.clone(),
            logger: Arc::clone(logger),
        })
    }

    pub fn poll(&self) -> Result<Vec<TouchFrame>, Error> {
        self.touchscreen.lock().unwrap().poll()
    }

    pub fn process(&self, frame: &TouchFrame) -> Vec<MouseEvent> {
        self.recognizer.lock().unwrap().process(frame, &self.logger)
    }

    pub fn emit(&self, event: MouseEvent) -> Result<(), Error> {
        self.mouse.lock().unwrap().emit(event)
    }

    pub fn set_transform_direction(&self, new_direction: Transform) {
        self.recognizer
            .lock()
            .unwrap()
            .set_transform_direction(new_direction);
    }

    pub fn get_polling_rate(&self) -> u64 {
        self.polling_rate_ms.load(Ordering::Relaxed)
    }

    pub fn reload_config(&self) -> Result<(), Error> {
        let new_profile = load_config(&self.custom_config_path, &self.logger);

        self.touchscreen
            .lock()
            .unwrap()
            .reload_config(&new_profile.configs[0].device)?;

        self.recognizer
            .lock()
            .unwrap()
            .reload_config(&new_profile.configs[0].gesture)?;

        self.polling_rate_ms.store(
            new_profile.configs[0].device.polling_rate_ms,
            Ordering::Relaxed,
        );

        *self.profile.lock().unwrap() = new_profile;
        Ok(())
    }

    pub fn switch_profile(&self, new_index: usize) -> Result<(), Error> {
        let mut index = self.config_index.lock().unwrap();
        let profile = self.profile.lock().unwrap();

        *index = new_index % profile.configs.len();

        self.touchscreen
            .lock()
            .unwrap()
            .reload_config(&profile.configs[*index].device)?;

        self.recognizer
            .lock()
            .unwrap()
            .reload_config(&profile.configs[*index].gesture)?;

        self.polling_rate_ms.store(
            profile.configs[*index].device.polling_rate_ms,
            Ordering::Relaxed,
        );

        Ok(())
    }
}
