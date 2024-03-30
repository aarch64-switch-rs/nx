//! `LogManager` logger implementation

use super::*;
use crate::rrt0;
use crate::mem;
use crate::ipc::sf;
use crate::service;
use crate::service::lm;
use crate::service::lm::ILogService;
use crate::service::lm::ILogger;
use crate::svc;

/// Represents a logger through [`LogService`][`lm::LogService`] services
pub struct LmLogger {
    logger: Option<mem::Shared<dyn ILogger>>
}

impl Logger for LmLogger {
    fn new() -> Self {
        let logger = match service::new_service_object::<lm::LogService>() {
            Ok(log_srv) => {
                match log_srv.get().open_logger(sf::ProcessId::new()) {
                    Ok(logger_obj) => Some(logger_obj),
                    Err(_) => None
                }
            },
            Err(_) => None
        };

        Self { logger }
    }

    fn log(&mut self, metadata: &LogMetadata) {
        if let Some(logger_obj) = &self.logger {
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
            log_packet.set_line_number(metadata.line_number);
    
            let mod_name = match rrt0::get_module_name().path.get_string() {
                Ok(name) => name,
                Err(_) => String::from("aarch64-switch-rs (invalid module name)")
            };
            log_packet.set_module_name(mod_name);

            log_packet.set_text_log(metadata.msg.clone());

            let thread_name = match cur_thread.name.get_str() {
                Ok(name) => name,
                _ => "aarch64-switch-rs (invalid thread name)",
            };
            log_packet.set_thread_name(String::from(thread_name));

            for packet in log_packet.encode_packet() {
                let _ = logger_obj.get().log(sf::Buffer::from_array(&packet));
            }
        }
    }
}