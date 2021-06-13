use chrono::{DateTime, Utc};
use std::sync::mpsc::Receiver;

enum LogType {
    Error,
    Log,
    Warn,
    Debug,
}

pub struct Logger {
    receiver: Receiver<String>,
    name: String,
}

impl Logger {
    pub fn new(logger_receiver: Receiver<String>, name: String) -> Logger {
        Logger {
            receiver: logger_receiver,
            name,
        }
    }

    pub fn log(&self, msg: &str) {
        self.write(LogType::Log, msg);
    }

    pub fn error(&self, msg: &str) {
        self.write(LogType::Error, msg);
    }

    pub fn warn(&self, msg: &str) {
        self.write(LogType::Warn, msg);
    }

    pub fn debug(&self, msg: &str) {
        self.write(LogType::Debug, msg);
    }

    pub fn run(&self) {
        for notification in &self.receiver {
            println!("{}", notification);
        }
    }

    fn write(&self, log_type: LogType, msg: &str) {
        let now: DateTime<Utc> = Utc::now();
        match log_type {
            LogType::Error => println!("[Rik] - {} [{}] {}", now, self.name, msg),
            LogType::Log => println!("[Rik] - {} [{}] {}", now, self.name, msg),
            LogType::Warn => println!("[Rik] - {} [{}] {}", now, self.name, msg),
            LogType::Debug => println!("[Rik] - {} [{}] {}", now, self.name, msg),
        }
    }
}
