use chrono::Utc;
use std::fmt;

#[derive(Clone, Debug)]
pub enum LogLevel {
    INFO,
    WARN,
    ERROR,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct LogEntry {
    pub timestamp: String,
    pub module: String,
    pub severity: LogLevel,
    pub message: String,
}

pub struct Logger {
    pub entries: Vec<LogEntry>,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            entries: Vec::with_capacity(100),
        }
    }

    pub fn log(&mut self, module: &str, severity: LogLevel, message: &str) {
        let entry = LogEntry {
            timestamp: Utc::now().to_rfc3339(),
            module: module.to_string(),
            severity,
            message: message.to_string(),
        };
        self.entries.push(entry);
        if self.entries.len() > 1000 {
            self.entries.remove(0);
        }
    }

    pub fn info(&mut self, module: &str, message: &str) {
        self.log(module, LogLevel::INFO, message);
    }

    pub fn warn(&mut self, module: &str, message: &str) {
        self.log(module, LogLevel::WARN, message);
    }

    pub fn error(&mut self, module: &str, message: &str) {
        self.log(module, LogLevel::ERROR, message);
    }
}
