use chrono::Local;
use serde::Serialize;
use std::io::{self, Write};

pub const DEFAULT_TIME_FORMAT: &str = "%d %b %Y %H:%M:%S";

pub enum LoggerMode {
    Text,
    Json,
}

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
    mode: LoggerMode,
}

impl<'a> Rlogger<'a> {
    pub fn new(output: Box<dyn io::Write + Send>, time_format: &'a str, mode: LoggerMode) -> Self {
        Rlogger {
            output,
            time_format,
            mode,
        }
    }
}

impl<'a> Default for Rlogger<'a> {
    fn default() -> Self {
        Rlogger {
            output: Box::new(io::stdout()),
            time_format: DEFAULT_TIME_FORMAT,
            mode: LoggerMode::Text,
        }
    }
}

#[derive(Serialize)]
struct LogEntry<'a> {
    level: &'a str,
    time: String,
    message: &'a str,
}

impl<'a> Rlogger<'a> {
    fn log(&mut self, level: &str, message: &str) {
        let formatted_time = Local::now().format(self.time_format).to_string();
        let _ = match self.mode {
            LoggerMode::Text => writeln!(self.output, "[{}] {} {}", level, formatted_time, message),
            LoggerMode::Json => {
                let entry = LogEntry {
                    level,
                    time: formatted_time,
                    message,
                };
                let json_string = serde_json::to_string(&entry).unwrap_or_else(|_| {
                    "{\"level\":\"ERROR\",\"time\":\"N/A\",\"message\":\"Failed to serialize log entry\"}".to_string()
                });
                writeln!(self.output, "{}", json_string)
            }
        };
        if level == "FATAL" {
            std::process::exit(1);
        }
    }
}

impl<'a> Logger for Rlogger<'a> {
    fn debug(&mut self, message: &str) {
        self.log("DEBUG", message);
    }

    fn info(&mut self, message: &str) {
        self.log("INFO", message);
    }

    fn warn(&mut self, message: &str) {
        self.log("WARN", message);
    }

    fn error(&mut self, message: &str) {
        self.log("ERROR", message);
    }

    fn fatal(&mut self, message: &str) {
        self.log("FATAL", message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::{Arc, Mutex};

    struct TestWriter {
        buffer: Arc<Mutex<Vec<u8>>>,
    }

    impl TestWriter {
        fn new(buffer: Arc<Mutex<Vec<u8>>>) -> Self {
            Self { buffer }
        }
    }

    impl Write for TestWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            let mut inner = self.buffer.lock().unwrap();
            inner.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn text_logger(buffer: Arc<Mutex<Vec<u8>>>) -> Rlogger<'static> {
        let writer = Box::new(TestWriter::new(buffer));
        Rlogger::new(writer, DEFAULT_TIME_FORMAT, LoggerMode::Text)
    }

    fn json_logger(buffer: Arc<Mutex<Vec<u8>>>) -> Rlogger<'static> {
        let writer = Box::new(TestWriter::new(buffer));
        Rlogger::new(writer, DEFAULT_TIME_FORMAT, LoggerMode::Json)
    }

    fn captured_string(buffer: Arc<Mutex<Vec<u8>>>) -> String {
        let bytes = buffer.lock().unwrap();
        String::from_utf8(bytes.clone()).expect("output is valid UTF-8")
    }

    #[test]
    fn default_time_format_is_correct() {
        let logger = Rlogger::default();
        assert_eq!(logger.time_format, DEFAULT_TIME_FORMAT);
    }

    #[test]
    fn default_mode_is_text() {
        let logger = Rlogger::default();
        assert!(matches!(logger.mode, LoggerMode::Text));
    }

    #[test]
    fn new_constructor_sets_fields() {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let writer = Box::new(TestWriter::new(buffer.clone()));
        let fmt = "%Y"; // custom time format
        let logger = Rlogger::new(writer, fmt, LoggerMode::Json);
        assert_eq!(logger.time_format, fmt);
        assert!(matches!(logger.mode, LoggerMode::Json));
    }

    #[test]
    fn text_mode_debug_logs_correct_format() {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let mut logger = text_logger(buffer.clone());
        logger.debug("hello world");

        let output = captured_string(buffer);
        assert!(output.contains("[DEBUG]"), "output should contain level");
        assert!(
            output.contains("hello world"),
            "output should contain message"
        );
        assert!(
            output.chars().any(|c| c.is_ascii_digit()),
            "output should contain a timestamp"
        );
        assert!(!output.contains('{'), "text mode must not output JSON");
    }

    #[test]
    fn text_mode_all_levels_appear_correctly() {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let mut logger = text_logger(buffer.clone());

        logger.info("info msg");
        logger.warn("warn msg");
        logger.error("error msg");

        let output = captured_string(buffer);
        for (level, msg) in &[
            ("INFO", "info msg"),
            ("WARN", "warn msg"),
            ("ERROR", "error msg"),
        ] {
            let prefix = format!("[{}]", level);
            assert!(
                output.contains(&prefix),
                "missing level header '{}'",
                prefix
            );
            assert!(
                output.contains(msg),
                "missing message '{}' for level '{}'",
                msg,
                level
            );
        }
    }

    #[test]
    fn text_mode_respects_custom_time_format() {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let writer = Box::new(TestWriter::new(buffer.clone()));
        let mut logger = Rlogger::new(writer, "%H:%M", LoggerMode::Text); // only hour:minute

        logger.info("test");
        let output = captured_string(buffer);

        let re = regex::Regex::new(r"^\[INFO\] \d{2}:\d{2} test\n$").unwrap();
        assert!(
            re.is_match(&output),
            "output '{}' did not match expected pattern",
            output.trim_end()
        );
    }

    #[test]
    fn json_mode_produces_valid_json() {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let mut logger = json_logger(buffer.clone());
        logger.info("a json message");

        let output = captured_string(buffer);
        let line = output.trim_end(); // remove trailing newline

        let parsed: serde_json::Value =
            serde_json::from_str(line).expect("output should be valid JSON");
        assert_eq!(parsed["level"], "INFO");
        assert_eq!(parsed["message"], "a json message");
        assert!(
            parsed["time"].is_string() && !parsed["time"].as_str().unwrap().is_empty(),
            "time field must be a non-empty string"
        );
    }

    #[test]
    fn json_mode_all_levels() {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let mut logger = json_logger(buffer.clone());
        logger.debug("d");
        logger.info("i");
        logger.warn("w");
        logger.error("e");

        let output = captured_string(buffer);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 4, "expected 4 log lines");

        let expected_levels = ["DEBUG", "INFO", "WARN", "ERROR"];
        for (line, expected) in lines.iter().zip(expected_levels.iter()) {
            let v: serde_json::Value = serde_json::from_str(line).expect("line is valid JSON");
            assert_eq!(v["level"], *expected);
            assert!(v["time"].is_string());
        }
    }

    #[test]
    fn json_mode_respects_custom_time_format() {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let writer = Box::new(TestWriter::new(buffer.clone()));
        let mut logger = Rlogger::new(writer, "%Y", LoggerMode::Json); // only year

        logger.info("custom time");
        let output = captured_string(buffer);
        let line = output.trim_end();
        let v: serde_json::Value = serde_json::from_str(line).unwrap();
        let time = v["time"].as_str().unwrap();
        // Should be a 4‑digit year
        assert!(
            time.len() == 4 && time.chars().all(|c| c.is_ascii_digit()),
            "expected 4‑digit year, got '{}'",
            time
        );
    }

    #[test]
    fn multiple_messages_are_newline_separated() {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let mut logger = text_logger(buffer.clone());
        logger.info("first");
        logger.warn("second");

        let output = captured_string(buffer);
        assert_eq!(output.matches('\n').count(), 2, "should have two newlines");
        assert!(output.starts_with("[INFO]"));
        assert!(output.lines().nth(1).unwrap().starts_with("[WARN]"));
    }

    #[test]
    fn empty_message_is_logged() {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let mut logger = text_logger(buffer.clone());
        logger.info("");
        let output = captured_string(buffer);
        assert!(output.contains("[INFO]"));
    }
}
