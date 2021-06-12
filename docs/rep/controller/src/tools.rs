use chrono::{DateTime, Utc};

enum LogType {
    Error,
    Log,
    Warn,
    Debug,
}

pub struct Logger {
    name: String,
}

impl Logger {
    pub fn new(name: String) -> Logger {
        Logger { name }
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
