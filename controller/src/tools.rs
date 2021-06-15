use chrono::{DateTime, Utc};
use std::sync::mpsc::Receiver;

#[allow(dead_code)]
pub enum LogType {
    Error,
    Log,
    Warn,
    Debug,
}

pub struct Logger {
    receiver: Receiver<Logging>,
    name: String,
}

impl Logger {
    pub fn new(logger_receiver: Receiver<Logging>, name: String) -> Logger {
        Logger {
            receiver: logger_receiver,
            name,
        }
    }

    pub fn run(&self) {
        for notification in &self.receiver {
            match notification.log_type {
                LogType::Error => self.error(&notification.message),
                LogType::Log => self.log(&notification.message),
                LogType::Warn => self.warn(&notification.message),
                LogType::Debug => self.debug(&notification.message),
            }
        }
    }

    fn log(&self, msg: &str) {
        let now: DateTime<Utc> = Utc::now();
        println!("[Rik] - {} [{}] {}", now, self.name, msg);
    }

    fn error(&self, msg: &str) {
        let now: DateTime<Utc> = Utc::now();
        println!("[Rik] - {} [{}] {}", now, self.name, msg);
    }

    fn warn(&self, msg: &str) {
        let now: DateTime<Utc> = Utc::now();
        println!("[Rik] - {} [{}] {}", now, self.name, msg);
    }

    fn debug(&self, msg: &str) {
        let now: DateTime<Utc> = Utc::now();
        println!("[Rik] - {} [{}] {}", now, self.name, msg);
    }
}

pub struct Logging {
    pub message: String,
    pub log_type: LogType,
}
