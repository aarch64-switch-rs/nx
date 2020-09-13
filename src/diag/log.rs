use crate::thread;
use crate::result::*;
use crate::mem;
use crate::ipc::sf;
use alloc::string::String;

pub type LogSeverity = logpacket::detail::LogSeverity;

pub struct LogMetadata {
    pub severity: LogSeverity,
    pub verbosity: bool,
    pub msg: String,
    pub file_name: &'static str,
    pub fn_name: &'static str,
    pub line_no: u32
}

impl LogMetadata {
    pub fn new(severity: LogSeverity, verbosity: bool, msg: String, file_name: &'static str, fn_name: &'static str, line_no: u32) -> Self {
        Self {
            severity: severity,
            verbosity: verbosity,
            msg: msg,
            file_name: file_name,
            fn_name: fn_name,
            line_no: line_no,
        }
    }
}

pub trait Logger {
    fn new() -> Self;
    fn log(&mut self, metadata: &LogMetadata);
}

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
    let thread_name = match thread::get_current_thread().name.get_str() {
        Ok(name) => name,
        _ => "<unknown>",
    };
    format!("[ {} (severity: {}, verbosity: {}) from {} in thread {}, at {}:{} ] {}", log_type, severity_str, metadata.verbosity, metadata.fn_name, thread_name, metadata.file_name, metadata.line_no, metadata.msg)
}

use crate::svc;

pub struct SvcOutputLogger;

impl Logger for SvcOutputLogger {
    fn new() -> Self {
        Self {}
    }

    fn log(&mut self, metadata: &LogMetadata) {
        let msg = format_plain_string_log_impl(metadata, "SvcOutputLog");
        let _ = svc::output_debug_string(msg.as_ptr(), msg.len());
    }
}

use crate::service;
use crate::service::fspsrv;
use crate::service::fspsrv::IFileSystemProxy;

pub struct FsAccessLogLogger {
    service: Result<mem::Shared<fspsrv::FileSystemProxy>>
}

impl Logger for FsAccessLogLogger {
    fn new() -> Self {
        Self { service: service::new_service_object() }
    }

    fn log(&mut self, metadata: &LogMetadata) {
        let msg = format_plain_string_log_impl(metadata, "FsAccessLog");
        match self.service {
            Ok(ref mut fspsrv) => {
                let _ = fspsrv.get().output_access_log_to_sd_card(sf::Buffer::from_const(msg.as_ptr(), msg.len()));
            },
            _ => {}
        }
    }
}


use crate::service::lm;
use crate::service::lm::ILogService;
use crate::service::lm::ILogger;

pub struct LmLogger {
    service: Result<mem::Shared<lm::LogService>>,
    logger: Result<mem::Shared<lm::Logger>>
}

impl Logger for LmLogger {
    fn new() -> Self {
        let mut service = service::new_service_object::<lm::LogService>();
        let logger = match service {
            Ok(ref mut service_obj) => match service_obj.get().open_logger(sf::ProcessId::new()) {
                Ok(logger_obj) => Ok(logger_obj.to::<lm::Logger>()),
                Err(rc) => Err(rc),
            },
            Err(rc) => Err(rc),
        };
        Self { service: service, logger: logger }
    }

    fn log(&mut self, metadata: &LogMetadata) {
        if self.service.is_ok() {
            if self.logger.is_ok() {
                let mut log_packet = logpacket::LogPacket::new();

                if let Ok(process_id) = svc::get_process_id(svc::CURRENT_PROCESS_PSEUDO_HANDLE) {
                    log_packet.set_process_id(process_id);
                }

                let cur_thread = thread::get_current_thread();
                if let Ok(thread_id) = cur_thread.get_id() {
                    log_packet.set_thread_id(thread_id);
                }

                log_packet.set_file_name(String::from(metadata.file_name));
                log_packet.set_function_name(String::from(metadata.fn_name));
                log_packet.set_line_number(metadata.line_no);
                log_packet.set_module_name(String::from("aarch64-switch-rs"));
                log_packet.set_text_log(metadata.msg.clone());
                let thread_name = match cur_thread.name.get_str() {
                    Ok(name) => name,
                    _ => "<unknown>",
                };
                log_packet.set_thread_name(String::from(thread_name));
                let log_buf = log_packet.encode();

                match &mut self.logger {
                    Ok(ref mut logger) => {
                        let _ = logger.get().log(sf::Buffer::from_const(log_buf.as_ptr(), log_buf.len()));
                    },
                    _ => {}
                }
            }
        }
    }
}