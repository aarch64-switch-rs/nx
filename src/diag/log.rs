//! Logging support and utils

use crate::thread;
use alloc::string::String;

/// Represents the logging severity
pub type LogSeverity = logpacket::detail::LogSeverity;

/// Represents the metadata used on a logging context
pub struct LogMetadata {
    /// The log severity
    pub severity: LogSeverity,
    /// Whether the log is verbose
    pub verbosity: bool,
    /// The log message
    pub msg: String,
    /// The source file name
    pub file_name: &'static str,
    /// The source function name
    pub fn_name: &'static str,
    /// The source line number
    pub line_number: u32,
}

impl LogMetadata {
    /// Creates a new [`LogMetadata`]
    ///
    /// # Arguments
    ///
    /// * `severity`: The log severity
    /// * `verbosity`: Whether the log is verbose
    /// * `msg`: The log message
    /// * `file_name`: The source file name
    /// * `fn_name`: The source function name
    /// * `line_number`: The source line number
    #[inline]
    pub const fn new(
        severity: LogSeverity,
        verbosity: bool,
        msg: String,
        file_name: &'static str,
        fn_name: &'static str,
        line_number: u32,
    ) -> Self {
        Self {
            severity,
            verbosity,
            msg,
            file_name,
            fn_name,
            line_number,
        }
    }
}

/// Represents a logging object
pub trait Logger {
    /// Creates a new logging object
    fn new() -> Self;
    /// Logs with the given metadata
    ///
    /// # Arguments
    ///
    /// * `metadata`: The metadata to log
    fn log(&mut self, metadata: &LogMetadata);
}

/// Wrapper for logging a single log
///
/// Essentially creates a [`Logger`] and logs with it
///
/// # Arguments
///
/// * `metadata`: The metadata to log
pub fn log_with<L: Logger>(metadata: &LogMetadata) {
    let mut logger = L::new();
    logger.log(metadata);
}

fn format_plain_string_log_impl(metadata: &LogMetadata, log_type: &str) -> String {
    let severity_str = match metadata.severity {
        LogSeverity::Trace => "Trace",
        LogSeverity::Info => "Info",
        LogSeverity::Warn => "Warn",
        LogSeverity::Error => "Error",
        LogSeverity::Fatal => "Fatal",
    };
    let thread_name = match unsafe { thread::current().as_ref().unwrap().name.get_str() } {
        Ok(name) => name,
        _ => "<unknown>",
    };
    format!(
        "[ {} (severity: {}, verbosity: {}) from {} in thread {}, at {}:{} ] {}",
        log_type,
        severity_str,
        metadata.verbosity,
        metadata.fn_name,
        thread_name,
        metadata.file_name,
        metadata.line_number,
        metadata.msg
    )
}

pub mod svc;

#[cfg(feature = "services")]
pub mod fs;

#[cfg(feature = "services")]
pub mod lm;
