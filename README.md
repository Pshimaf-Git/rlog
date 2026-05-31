# rlog

A simple, flexible logging library for Rust with support for text and JSON output.

## Features

- **Multiple Log Levels**: Supports `DEBUG`, `INFO`, `WARN`, `ERROR`, and `FATAL`.
- **Output Modes**: Switch between human-readable `Text` and machine-readable `Json`.
- **Customizable Output**: Log to `stdout`, `stderr`, files, or any type implementing `Write + Send`.
- **Custom Time Formats**: Flexible timestamp formatting using `chrono` patterns.
- **Panic on Fatal**: Automatically exits the process with code `1` when a `FATAL` message is logged.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rlog = { git = "https://github.com/Pshimaf-Git/rlog" }
```

## Usage

### Basic Usage (Text Mode)

By default, `Rlogger` logs to `stdout` in text mode.

```rust
use rlog::{Rlogger};

fn main() {
    let mut logger = Rlogger::default(); // use default logger settings
    
    logger.info("Application started");
    logger.debug("Debugging some logic");
    logger.warn("This is a warning");
    logger.error("Something went wrong");
}
```

### JSON Mode

Perfect for structured logging and integration with log management systems.

```rust
use rlog::{Rlogger, LoggerMode};
use std::io;

fn main() {
    let mut logger = Rlogger::new(
        Box::new(io::stderr()), // logging into stderr 
        "%Y-%m-%dT%H:%M:%S%z",
        LoggerMode::Json
    );

    logger.info("Structured log message");
}
```

### Custom Output (Logging to a File)

```rust
use rlog::{Rlogger, LoggerMode, DEFAULT_TIME_FORMAT};
use std::fs::File;

fn main() -> std::io::Result<()> {
    let file = File::create("myapp.log")?;
    let mut logger = Rlogger::new(
        Box::new(file),
        DEFAULT_TIME_FORMAT,
        LoggerMode::Text
    );

    logger.info("This message goes to a file");
    Ok(())
}
```

## API

### `Logger` Trait
The core trait for logging functionality:
- `debug(&mut self, message: &str)`
- `info(&mut self, message: &str)`
- `warn(&mut self, message: &str)`
- `error(&mut self, message: &str)`
- `fatal(&mut self, message: &str)` (Calls `process::exit(1)`)

### `Rlogger` Struct
The main implementation of the `Logger` trait.
- `Rlogger::new(output: Box<dyn Write + Send>, time_format: &str, mode: LoggerMode)`
- `Rlogger::default()`: Logs to `stdout` in `Text` mode with `%d %b %Y %H:%M:%S` format.

### `LoggerMode` Enum
- `Text`: Human-readable format: `[LEVEL] TIMESTAMP MESSAGE`
- `Json`: JSON object: `{"level":"...","time":"...","message":"..."}`

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
