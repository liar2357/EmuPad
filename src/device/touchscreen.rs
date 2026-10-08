use evdev::Device;

use crate::{
    config::structs::DeviceConfig,
    device::discover::discover_device,
    error::Error,
    event::log::Logger,
    input::{frame::TouchFrame, state::TouchState},
};

pub struct Touchscreen {
    device: Device,
    state: TouchState,
}

impl Touchscreen {
    pub fn open(config: &DeviceConfig, logger: &Logger) -> Result<Self, Error> {
        let info = discover_device(&config.selector)?;

        logger.info(format!("lithen: {:?}{}", info.path, {
            if info.name.is_some() {
                format!("/ {}", info.name.clone().unwrap())
            } else {
                "".to_string()
            }
        }));

        let device = Device::open(&info.path)?;
        device.set_nonblocking(true)?;

        Ok(Self {
            device,
            state: TouchState::new(),
        })
    }

    pub fn reload_config(&mut self, config: &DeviceConfig) -> Result<(), Error> {
        let info = discover_device(&config.selector)?;

        self.device = Device::open(&info.path)?;
        self.state = TouchState::new();

        self.device.set_nonblocking(true)?;
        Ok(())
    }

    pub fn poll(&mut self) -> Result<Vec<TouchFrame>, Error> {
        let events: Vec<_> = match self.device.fetch_events() {
            Ok(events) => events.collect(),

            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => vec![],
            Err(e) => {
                // 本当のI/Oエラー
                return Err(e.into());
            }
        };

        let mut frames = Vec::new();

        for event in events {
            if let Some(frame) = self.state.process_event(event.destructure()) {
                frames.push(frame);
            }
        }

        Ok(frames)
    }
}
