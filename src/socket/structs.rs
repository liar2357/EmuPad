use std::{
    fmt::{Display, Formatter, Result as fResult},
    str::FromStr,
};

#[derive(Debug)]
pub enum SocketCommand {
    SetDirection(u8),
    SwitchProfile(usize),
    ReloadApp,
    ShutdownApp,
}

impl SocketCommand {
    pub fn get_all_comands_string() -> String {
        [
            Self::SetDirection(0),
            Self::SwitchProfile(0),
            Self::ReloadApp,
            Self::ShutdownApp,
        ]
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<String>>()
        .join("\n")
        .to_string()
    }

    pub fn print_all() {
        eprintln!("Available Commands");
        eprintln!("------------------");
        eprintln!("{}", Self::get_all_comands_string());
    }
}

impl Display for SocketCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> fResult {
        let s = match self {
            Self::SetDirection(_) => "set_direction <number>",
            Self::SwitchProfile(_) => "switch_profile <index>",
            Self::ReloadApp => "reload_app",
            Self::ShutdownApp => "shutdown_app",
        };

        write!(f, "{s}")
    }
}

impl FromStr for SocketCommand {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "reload_app" => Ok(SocketCommand::ReloadApp),
            "shutdown_app" => Ok(Self::ShutdownApp),

            _ => {
                let (command, arg) = s.split_once('-').ok_or(())?;

                match command {
                    "set_direction" => {
                        let num = arg.parse::<u8>().map_err(|_| ())?;
                        Ok(Self::SetDirection(num))
                    }
                    "switch_profile" => {
                        let index = arg.parse::<usize>().map_err(|_| ())?;
                        Ok(Self::SwitchProfile(index))
                    }
                    _ => Err(()),
                }
            }
        }
    }
}
