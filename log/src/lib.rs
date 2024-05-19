use std::sync::{Mutex, Arc};
use std::fmt;

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Status,
    Fatal,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let level_str = match self {
            LogLevel::Debug => "\x1B[36m[Debug] ",
            LogLevel::Info => "\x1B[32m[Info] ",
            LogLevel::Warning => "\x1B[33m[Warning] ",
            LogLevel::Error => "\x1B[31m[Error] ",
            LogLevel::Status => "\x1B[33m[Status] ",
            LogLevel::Fatal => "\x1B[31m[Fatal] ",
        };
        write!(f, "{}", level_str)
    }
}

#[derive(Clone)]
pub struct Logger {
    logger_name: String,
    log_level: LogLevel,
    mutex: Arc<Mutex<()>>,
}

impl Logger {
    pub fn new(logger_name: &str, log_level: LogLevel) -> Self {
        Logger {
            logger_name: logger_name.to_string(),
            log_level,
            mutex: Arc::new(Mutex::new(())),
        }
    }

    pub fn log<T: fmt::Display>(&self, log_level: LogLevel, message: T) {
        if log_level < self.log_level {
            return;
        }

        let level_str = format!("{}", log_level);
        let _lock = self.mutex.lock().unwrap();
        println!("{}{}{}", level_str, message, "\x1B[0m");
    }

    pub fn get_log_level(&self) -> LogLevel {
        self.log_level
    }
}

impl fmt::Display for Logger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Logger Name: {}, Level: {}",
            self.logger_name, self.log_level
        )
    }
}
