use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("device not found")]
    DeviceNotFound,

    #[error("multiple matching touchscreen devices found: {paths:?}")]
    MultipleDevicesFound { paths: Vec<PathBuf> },

    #[error("input device error: {0}")]
    DeviceError(#[from] std::io::Error),

    #[error("commandline args error: {0}")]
    ArgumentError(#[from] clap::Error),

    #[error("reload monitor error: {0}")]
    ReloadError(#[from] notify::Error),

    #[error("output error: {0}")]
    Output(String),

    #[error("configuration error: {0}")]
    Config(String),

    #[error("process error: {0}")]
    Process(String),

    #[error("control error: {0}")]
    Control(String),

    #[error("invalid device selector: {0}")]
    InvalidDeviceSelector(String),
}
