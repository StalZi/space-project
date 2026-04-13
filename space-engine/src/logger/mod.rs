use std::sync::OnceLock;

#[derive(Debug)]
pub struct Logger {
    is_enabled: bool,
    verbose: bool,
}

#[derive(PartialEq)]
pub enum LogLevel {
    Info,
    Success,
    Warning,
    Error,
    Verbose,
}

static GLOBAL_LOGGER: OnceLock<Logger> = OnceLock::new();

// 3. Create a helper function to initialize/access the logger

impl Logger {
    pub fn create(is_enabled: bool, verbose: bool) {
        GLOBAL_LOGGER
            .set(Logger {
                is_enabled,
                verbose,
            })
            .unwrap();
        Logger::get_logger().log("Logger initialized", LogLevel::Success);
    }

    pub fn get_logger() -> &'static Logger {
        GLOBAL_LOGGER.get().unwrap()
    }

    pub fn log(&self, message: impl AsRef<str>, level: LogLevel) {
        if !self.is_enabled {
            return;
        }
        if (level == LogLevel::Verbose) & (!self.verbose) {
            return;
        }
        // Using ANSI escape codes for colored output with rgb format: "\x1b[38;2;<R>;<G>;<B>m", or standard format: "\x1b[<NUMBER>>m"
        let (left_ansi, level_str, right_ansi) = match level {
            LogLevel::Info => ("", "INFO", ""),
            LogLevel::Success => ("\x1b[32m", "SUCCESS", "\x1b[0m"),
            LogLevel::Warning => ("\x1b[33m", "WARNING", "\x1b[0m"),
            LogLevel::Error => ("\x1b[31m", "ERROR", "\x1b[0m"),
            LogLevel::Verbose => ("\x1b[38;2;0;196;181m", "VERBOSE", "\x1b[0m"),
        };
        println!(
            "{}[{}]: {}{}",
            left_ansi,
            level_str,
            message.as_ref(),
            right_ansi
        );
    }

    pub fn log_list(&self, header: impl AsRef<str>, list:  impl IntoIterator<Item = impl AsRef<str> + std::fmt::Debug>, level: LogLevel) {
        if !self.is_enabled {
            return;
        }
        if (level == LogLevel::Verbose) & (!self.verbose) {
            return;
        }

        // Using ANSI escape codes for colored output
        let (left_ansi, level_str, right_ansi) = match level {
            LogLevel::Info => ("", "INFO", ""),
            LogLevel::Success => ("\x1b[32m", "SUCCESS", "\x1b[0m"),
            LogLevel::Warning => ("\x1b[33m", "WARNING", "\x1b[0m"),
            LogLevel::Error => ("\x1b[31m", "ERROR", "\x1b[0m"),
            LogLevel::Verbose => ("\x1b[38;2;0;196;181m", "VERBOSE", "\x1b[0m"),
        };
        println!("====================================");
        println!(
            "{}[{}]: {}{}",
            left_ansi,
            level_str,
            header.as_ref(),
            right_ansi
        );
        

        let (left_ansi, level_str, right_ansi) = match level {
            LogLevel::Verbose => ("\x1b[36m", "VERBOSE LIST", "\x1b[0m"),
            _ => ("\x1b[38;2;235;122;42m", "LIST", "\x1b[0m"),
        };
        for element in list {
            println!(
                "{}[{}]: {:?}{}",
                left_ansi,
                level_str,
                element,
                right_ansi
            );
        }

        println!("====================================");
    }
}
