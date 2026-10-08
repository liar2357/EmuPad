use crate::{config::structs::DeviceSelector, error::Error};
use evdev::{AbsoluteAxisCode, Device, PropType};
use std::{
    fs,
    path::{Path, PathBuf},
};

const INPUT_DIR: &str = "/dev/input";

/// Information about a discovered input device.
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub path: PathBuf,
    pub name: Option<String>,
    pub vendor: u16,
    pub product: u16,
}

/// Discover an input device according to the specified selector.
pub fn discover_device(selector: &DeviceSelector) -> Result<DeviceInfo, Error> {
    match selector {
        DeviceSelector::Path(path) => open_and_describe(path),

        DeviceSelector::Name(name) => discover_by_name(name),

        DeviceSelector::VendorProduct { vendor, product } => {
            discover_by_vendor_product(*vendor, *product)
        }

        DeviceSelector::Touchscreen => discover_touchscreen(),
    }
}

/// Discover a touchscreen by its kernel-reported device name.
///
/// The name must match exactly.
pub fn discover_by_name(name: &str) -> Result<DeviceInfo, Error> {
    if name.is_empty() {
        return Err(Error::Config("Device name is empty".to_string()));
    }

    let candidates = enumerate_devices()?;

    let mut matches = Vec::new();

    for (path, device) in candidates {
        if device.name() == Some(name) && is_touchscreen(&device) {
            matches.push(device_info(path, &device));
        }
    }

    select_unique(matches)
}

/// Discover a device by vendor/product ID.
///
/// The device must also satisfy the touchscreen capability check.
pub fn discover_by_vendor_product(vendor: u16, product: u16) -> Result<DeviceInfo, Error> {
    let candidates = enumerate_devices()?;

    let mut matches = Vec::new();

    for (path, device) in candidates {
        let id = device.input_id();

        if id.vendor() == vendor && id.product() == product && is_touchscreen(&device) {
            matches.push(device_info(path, &device));
        }
    }

    select_unique(matches)
}

/// Discover a touchscreen automatically by its evdev capabilities.
pub fn discover_touchscreen() -> Result<DeviceInfo, Error> {
    let candidates = enumerate_devices()?;

    let matches = candidates
        .into_iter()
        .filter(|(_, device)| is_touchscreen(device))
        .map(|(path, device)| device_info(path, &device))
        .collect();

    select_unique(matches)
}

/// Enumerate `/dev/input/event*` devices.
///
/// Devices that cannot be opened are ignored here because `/dev/input`
/// may contain devices which are inaccessible to the current process.
fn enumerate_devices() -> Result<Vec<(PathBuf, Device)>, Error> {
    let entries = fs::read_dir(INPUT_DIR)?;

    let mut devices = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };

        let path = entry.path();

        if !is_event_device(&path) {
            continue;
        }

        let device = match Device::open(&path) {
            Ok(device) => device,
            Err(_) => continue,
        };

        devices.push((path, device));
    }

    Ok(devices)
}

/// Check whether a path represents `/dev/input/event*`.
fn is_event_device(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };

    let Some(number) = file_name.strip_prefix("event") else {
        return false;
    };

    !number.is_empty() && number.chars().all(|c| c.is_ascii_digit())
}

/// Determine whether an evdev device is a touchscreen.
///
/// Required capabilities:
/// - INPUT_PROP_DIRECT
/// - ABS_MT_SLOT
/// - ABS_MT_TRACKING_ID
/// - ABS_MT_POSITION_X
/// - ABS_MT_POSITION_Y
fn is_touchscreen(device: &Device) -> bool {
    let has_direct_property = device.properties().contains(PropType::DIRECT);

    let Some(axes) = device.supported_absolute_axes() else {
        return false;
    };

    let has_multitouch_axes = axes.contains(AbsoluteAxisCode::ABS_MT_SLOT)
        && axes.contains(AbsoluteAxisCode::ABS_MT_TRACKING_ID)
        && axes.contains(AbsoluteAxisCode::ABS_MT_POSITION_X)
        && axes.contains(AbsoluteAxisCode::ABS_MT_POSITION_Y);

    has_direct_property && has_multitouch_axes
}

/// Convert an evdev device into our public device information.
fn device_info(path: PathBuf, device: &Device) -> DeviceInfo {
    let id = device.input_id();

    DeviceInfo {
        path,
        name: device.name().map(str::to_owned),
        vendor: id.vendor(),
        product: id.product(),
    }
}

/// Ensure that discovery produced exactly one device.
fn select_unique(matches: Vec<DeviceInfo>) -> Result<DeviceInfo, Error> {
    match matches.len() {
        0 => Err(Error::DeviceNotFound),

        1 => Ok(matches.into_iter().next().unwrap()),

        _ => Err(Error::MultipleDevicesFound {
            paths: matches.into_iter().map(|device| device.path).collect(),
        }),
    }
}

/// Open an explicitly specified device path and return its information.
fn open_and_describe(path: &Path) -> Result<DeviceInfo, Error> {
    let device = Device::open(path)?;

    Ok(device_info(path.to_path_buf(), &device))
}
