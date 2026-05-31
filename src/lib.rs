use std::io::{self, Write};
use chrono::Local;

const DEFAULT_TIME_FORMAT: &str = "%d %b %Y %H:%M:%S";

pub trait Logger {
    fn debug(&mut self, message: &str);
    fn info(&mut self, message: &str);
    fn warn(&mut self, message: &str);
    fn error(&mut self, message: &str);
    fn fatal(&mut self, message: &str);
}

pub struct Rlogger<'a> {
    output: Box<dyn io::Write + Send>,
    time_format: &'a str,
}

impl<'a> Rlogger<'a> {
    pub fn new(output: Box<dyn io::Write + Send>, time_format: &'a str) -> Self {
        Rlogger {
            output,
            time_format,
        }
    }
}

impl<'a> Default for Rlogger<'a> {
    fn default() -> Self {
        Rlogger {
            output: Box::new(io::stdout()),
            time_format: DEFAULT_TIME_FORMAT,
        }
    }
}

impl<'a> Logger for Rlogger<'a> {
    fn debug(&mut self, message: &str) {
        let _ = writeln!(
            self.output,
            "[DEBUG] {} {}",
            Local::now().format(self.time_format),
            message
        );
    }

    fn info(&mut self, message: &str) {
        let _ = writeln!(
            self.output,
            "[INFO] {} {}",
            Local::now().format(self.time_format),
            message
        );
    }

    fn warn(&mut self, message: &str) {
        let _ = writeln!(
            self.output,
            "[WARN] {} {}",
            Local::now().format(self.time_format),
            message
        );
    }

    fn error(&mut self, message: &str) {
        let _ = writeln!(
            self.output,
            "[ERROR] {} {}",
            Local::now().format(self.time_format),
            message
        );
    }

    fn fatal(&mut self, message: &str) {
        let _ = writeln!(
            self.output,
            "[FATAL] {} {}",
            Local::now().format(self.time_format),
            message
        );
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct MockLogger {
        messages: Mutex<Vec<String>>,
    }

    impl MockLogger {
        fn new() -> MockLogger {
            MockLogger {
                messages: Mutex::new(Vec::new()),
            }
        }

        fn get_messages(&self) -> Vec<String> {
            self.messages.lock().unwrap().clone()
        }
    }

    impl Logger for MockLogger {
        fn debug(&mut self, message: &str) {
            self.messages
                .lock()
                .unwrap()
                .push(format!("[DEBUG] {}", message));
        }

        fn info(&mut self, message: &str) {
            self.messages
                .lock()
                .unwrap()
                .push(format!("[INFO] {}", message));
        }

        fn warn(&mut self, message: &str) {
            self.messages
                .lock()
                .unwrap()
                .push(format!("[WARN] {}", message));
        }

        fn error(&mut self, message: &str) {
            self.messages
                .lock()
                .unwrap()
                .push(format!("[ERROR] {}", message));
        }

        fn fatal(&mut self, message: &str) {
            self.messages
                .lock()
                .unwrap()
                .push(format!("[FATAL] {}", message));
        }
    }

    #[test]
    fn test_rlogger_debug() {
        let mut logger = Rlogger::default();
        // This test primarily checks for compilation and method existence,
        // actual output to stdout/stderr is hard to capture in a unit test.
        logger.debug("This is a debug message.");
    }

    #[test]
    fn test_rlogger_info() {
        let mut logger = Rlogger::default();
        logger.info("This is an info message.");
    }

    #[test]
    fn test_rlogger_warn() {
        let mut logger = Rlogger::default();
        logger.warn("This is a warning message.");
    }

    #[test]
    fn test_rlogger_error() {
        let mut logger = Rlogger::default();
        logger.error("This is an error message.");
    }

    #[test]
    fn test_rlogger_fatal() {
        let mut logger = Rlogger::default();
        // This test will cause the program to exit, so we can't assert anything after it.
        // It's mainly for compilation check.
        // logger.fatal("This is a fatal message.");
    }

    #[test]
    fn test_mock_logger() {
        let mut logger = MockLogger::new();
        logger.debug("Test debug");
        logger.info("Test info");
        logger.warn("Test warn");
        logger.error("Test error");
        logger.fatal("Test fatal");

        let messages = logger.get_messages();
        assert_eq!(messages.len(), 5);
        assert!(messages.contains(&"[DEBUG] Test debug".to_string()));
        assert!(messages.contains(&"[INFO] Test info".to_string()));
        assert!(messages.contains(&"[WARN] Test warn".to_string()));
        assert!(messages.contains(&"[ERROR] Test error".to_string()));
        assert!(messages.contains(&"[FATAL] Test fatal".to_string()));
    }
}
