use std::fmt::{Display, Formatter, Result as FmtResult};

const RESET: &str = "\x1b[0m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const GREEN: &str = "\x1b[32m";
const CYAN: &str = "\x1b[36m";

#[repr(u8)]
#[derive(Clone, Copy)]
enum LogType {
    Error = 0,
    Warning = 1,
    Information = 2,
    Trace = 3,
}

impl LogType {
    fn level(&self) -> u8 {
        *self as u8
    }

    fn color(&self) -> &'static str {
        match self {
            Self::Trace => CYAN,
            Self::Information => GREEN,
            Self::Warning => YELLOW,
            Self::Error => RED,
        }
    }
}

impl Display for LogType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let s = match self {
            Self::Trace => "TRACE",
            Self::Information => "INFO ",
            Self::Warning => "WARN ",
            Self::Error => "ERROR",
        };

        write!(f, "{s}")
    }
}

fn logging<T: Display>(log_type: LogType, content: T, log_level: u8) {
    if log_level >= log_type.level() {
        eprintln!("{}[{}]{} {}", log_type.color(), log_type, RESET, content);
    }
}

pub struct Logger {
    verbose: u8,
}
impl Logger {
    pub fn new(verbose: u8) -> Self {
        Self { verbose }
    }

    pub fn trace<T: Display>(&self, content: T) {
        logging(LogType::Trace, content, self.verbose);
    }

    pub fn info<T: Display>(&self, content: T) {
        logging(LogType::Information, content, self.verbose);
    }

    pub fn warn<T: Display>(&self, content: T) {
        logging(LogType::Warning, content, self.verbose);
    }

    pub fn error<T: Display>(&self, content: T) {
        logging(LogType::Error, content, self.verbose);
    }
}
