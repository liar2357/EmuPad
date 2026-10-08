use crate::{coordinate::transform::Transform, error::Error};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, str::FromStr};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Profile {
    pub configs: Vec<Config>,
}
impl Default for Profile {
    fn default() -> Self {
        Self {
            configs: vec![Config::default()],
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Config {
    pub device: DeviceConfig,
    pub gesture: GestureConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct DeviceConfig {
    pub selector: DeviceSelector,
    pub polling_rate_ms: u64,
}
impl Default for DeviceConfig {
    fn default() -> Self {
        Self {
            selector: DeviceSelector::Touchscreen,
            polling_rate_ms: 3,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct GestureConfig {
    pub tap_threshold_ms: u64,
    pub long_press_threshold_ms: u64,
    pub movement_threshold: f64,
    pub release_grace_threshold_ms: u64,
    pub default_direction: Transform,
    pub move_sensitivity_percent: u16,
    pub scroll_sensitivity_percent: u16,
}
impl Default for GestureConfig {
    fn default() -> Self {
        Self {
            tap_threshold_ms: 250,
            long_press_threshold_ms: 500,
            movement_threshold: 15.0,
            release_grace_threshold_ms: 100,
            default_direction: Transform::Normal,
            move_sensitivity_percent: 80,
            scroll_sensitivity_percent: 50,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(try_from = "String")]
pub enum DeviceSelector {
    Path(PathBuf),
    Name(String),
    VendorProduct { vendor: u16, product: u16 },
    Touchscreen,
}

impl Serialize for DeviceSelector {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl FromStr for DeviceSelector {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        // 自動検出
        if value == "auto" {
            return Ok(Self::Touchscreen);
        }

        let Some((kind, value)) = value.split_once(':') else {
            return Err(Error::InvalidDeviceSelector(value.to_string()));
        };

        match kind {
            "path" => Ok(Self::Path(PathBuf::from(value))),

            "name" => Ok(Self::Name(value.to_string())),

            "vp" => {
                let Some((vendor, product)) = value.split_once('/') else {
                    return Err(Error::InvalidDeviceSelector(value.to_string()));
                };

                let vendor = u16::from_str_radix(vendor, 16)
                    .map_err(|_| Error::InvalidDeviceSelector(value.to_string()))?;

                let product = u16::from_str_radix(product, 16)
                    .map_err(|_| Error::InvalidDeviceSelector(value.to_string()))?;

                Ok(Self::VendorProduct { vendor, product })
            }

            _ => Err(Error::InvalidDeviceSelector(value.to_string())),
        }
    }
}

impl TryFrom<String> for DeviceSelector {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl std::fmt::Display for DeviceSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Path(path) => {
                write!(f, "path:{}", path.display())
            }

            Self::Name(name) => {
                write!(f, "name:{name}")
            }

            Self::VendorProduct { vendor, product } => {
                write!(f, "vp:{vendor:04X}/{product:04X}")
            }

            Self::Touchscreen => {
                write!(f, "auto")
            }
        }
    }
}
