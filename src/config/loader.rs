use crate::{config::structs::Profile, event::log::Logger};
use directories::ProjectDirs;
use std::{fs, io, path::PathBuf};

fn get_or_create_config_path() -> io::Result<PathBuf> {
    let dirs = ProjectDirs::from("", "", env!("CARGO_PKG_NAME"))
        .ok_or_else(|| io::Error::other("Failed to get config directory"))?;

    let path = dirs.config_dir().join("config.toml");

    if !path.is_file() {
        fs::create_dir_all(dirs.config_dir())?;

        let default = Profile::default();
        let toml = toml::to_string_pretty(&default).map_err(io::Error::other)?;

        fs::write(&path, toml)?;
    }

    Ok(path)
}

fn try_get_custom_path(custom_path: &Option<PathBuf>) -> io::Result<PathBuf> {
    match custom_path {
        Some(path) => {
            if path.is_file() {
                path.canonicalize()
            } else {
                Err(io::Error::other("Failed to get custom config path"))
            }
        }
        None => Err(io::Error::other("Failed to get custom config path")),
    }
}

pub fn resolve_config_path(custom_path: &Option<PathBuf>, logger: &Logger) -> Option<PathBuf> {
    match try_get_custom_path(custom_path) {
        Ok(path) => Some(path),
        Err(e) => {
            logger.info(e);
            match get_or_create_config_path() {
                Ok(path) => {
                    logger.info("Fallback to default config path");
                    Some(path)
                }
                Err(e) => {
                    logger.info(e);
                    None
                }
            }
        }
    }
}

pub fn load_config(path: &Option<PathBuf>, logger: &Logger) -> Profile {
    match path {
        Some(conf) => match fs::read_to_string(conf) {
            Ok(content) => match toml::from_str(&content) {
                Ok(v) => {
                    logger.trace("Config Load Successfull");
                    v
                }
                Err(e) => {
                    logger.warn(format!("Config parse error: {e}"));
                    Profile::default()
                }
            },
            Err(e) => {
                logger.warn(format!("Config parse error: {e}"));
                Profile::default()
            }
        },
        None => {
            logger.info("Fallback to default config setting");
            Profile::default()
        }
    }
}
