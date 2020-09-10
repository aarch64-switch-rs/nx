use crate::thread;
use crate::result::*;
use crate::util;
use crate::mem;
use crate::ipc::sf;

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use core::mem as cmem;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum LogSeverity {
    Trace,
    Info,
    Warn,
    Error,
    Fatal,
}

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

use crate::svc;

pub struct SvcOutputLogger;

impl Logger for SvcOutputLogger {
    fn new() -> Self {
        Self {}
    }

    fn log(&mut self, metadata: &LogMetadata) {
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
        let msg = format!("[ SvcOutputLog (severity: {}, verbosity: {}) from {} in thread {}, at {}:{} ] {}", severity_str, metadata.verbosity, metadata.fn_name, thread_name, metadata.file_name, metadata.line_no, metadata.msg);
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
        let msg = format!("[ FsAccessLog (severity: {}, verbosity: {}) from {} in thread {}, at {}:{} ] {}", severity_str, metadata.verbosity, metadata.fn_name, thread_name, metadata.file_name, metadata.line_no, metadata.msg);
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

bit_enum! {
    LogPacketFlags (u8) {
        Head = bit!(0),
        Tail = bit!(1),
        LittleEndian = bit!(2)
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct LogPacketHeader {
    pub process_id: u64,
    pub thread_id: u64,
    pub flags: LogPacketFlags,
    pub pad: u8,
    pub severity: LogSeverity,
    pub verbosity: bool,
    pub payload_size: u32,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum LogDataChunkKey {
    LogSessionBegin,
    LogSessionEnd,
    TextLog,
    LineNumber,
    FileName,
    FunctionName,
    ModuleName,
    ThreadName,
    LogPacketDropCount,
    UserSystemClock,
    ProcessName,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct LogDataChunkHeader {
    pub key: LogDataChunkKey,
    pub length: u8,
}

impl LogDataChunkHeader {
    pub const fn new(key: LogDataChunkKey, length: u8) -> Self {
        Self { key: key, length: length }
    }

    pub const fn compute_chunk_size(&self) -> usize {
        if self.length == 0 {
            return 0;
        }
        cmem::size_of::<LogDataChunkHeader>() + self.length as usize
    }
}

const MAX_STRING_LEN: usize = 0x7F;

trait LogDataChunkBase {
    fn get_header(&self) -> LogDataChunkHeader;

    fn is_empty(&self) -> bool {
        self.get_header().length == 0
    }

     fn compute_chunk_size(&self) -> usize {
        self.get_header().compute_chunk_size()
    }
}

#[allow(safe_packed_borrows)]
#[derive(Copy, Clone)]
#[repr(C)]
#[repr(packed)]
struct LogDataChunk<T> {
    pub header: LogDataChunkHeader,
    pub value: T,
}

impl<T: Default> LogDataChunk<T> {
    pub const fn from(key: LogDataChunkKey, value: T) -> Self {
        Self { header: LogDataChunkHeader::new(key, cmem::size_of::<T>() as u8), value: value }
    }
}

impl<T> LogDataChunkBase for LogDataChunk<T> {
    fn get_header(&self) -> LogDataChunkHeader {
        self.header
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
struct LogDataStringChunk {
    pub header: LogDataChunkHeader,
    pub value: util::CString<{MAX_STRING_LEN + 1}>,
}

impl LogDataStringChunk {
    pub fn from(key: LogDataChunkKey, value: String) -> Self {
        let mut chunk = Self { header: LogDataChunkHeader::new(key, value.len() as u8), value: util::CString::new() };
        let rc = chunk.value.set_string(value);
        match rc {
            Err(_) => chunk.header.length = 0,
            _ => {},
        }
        chunk
    }
}

impl LogDataChunkBase for LogDataStringChunk {
    fn get_header(&self) -> LogDataChunkHeader {
        self.header
    }
}

struct LogPacketPayload {
    pub log_session_begin: LogDataChunk<bool>,
    pub log_session_end: LogDataChunk<bool>,
    pub text_log: LogDataStringChunk,
    pub line_number: LogDataChunk<u32>,
    pub file_name: LogDataStringChunk,
    pub function_name: LogDataStringChunk,
    pub module_name: LogDataStringChunk,
    pub thread_name: LogDataStringChunk,
    pub log_packet_drop_count: LogDataChunk<u64>,
    pub user_system_clock: LogDataChunk<u64>,
    pub process_name: LogDataStringChunk,
}

impl LogPacketPayload {
    pub fn compute_chunk_size(&self) -> usize {
        self.log_session_begin.compute_chunk_size() +
        self.log_session_end.compute_chunk_size() +
        self.text_log.compute_chunk_size() +
        self.line_number.compute_chunk_size() +
        self.file_name.compute_chunk_size() +
        self.function_name.compute_chunk_size() +
        self.module_name.compute_chunk_size() +
        self.thread_name.compute_chunk_size() +
        self.log_packet_drop_count.compute_chunk_size() +
        self.user_system_clock.compute_chunk_size() +
        self.process_name.compute_chunk_size()
    }
}

struct LogPacket {
    header: LogPacketHeader,
    payload: LogPacketPayload,
}

impl LogPacket {
    pub fn compute_chunk_size(&self) -> usize {
        cmem::size_of::<LogPacketHeader>() + self.payload.compute_chunk_size()
    }
}

fn compute_packet_count(msg_len: usize) -> usize {
    let mut remaining_len = msg_len;
    let mut packet_count: usize = 1;
    while remaining_len > MAX_STRING_LEN {
        packet_count += 1;
        remaining_len -= MAX_STRING_LEN;
    }
    
    packet_count
}

fn encode_payload_base<T: Copy>(buf: *mut u8, t: &T, size: usize) -> *mut u8 {
    unsafe {
        let tmp_buf = buf as *mut T;
        *tmp_buf = *t;
        buf.offset(size as isize)
    }
}

fn encode_payload<T: Copy + LogDataChunkBase>(buf: *mut u8, t: &T) -> *mut u8 {
    if t.is_empty() {
        return buf;
    }
    encode_payload_base(buf, t, t.compute_chunk_size())
}

fn encode_payload_buf<T: Copy>(buf: *mut u8, t: &T) -> *mut u8 {
    encode_payload_base(buf, t, cmem::size_of::<T>())
}

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
                unsafe {
                    let packet_count = compute_packet_count(metadata.msg.len());
                    let mut packets: Vec<LogPacket> = Vec::new();

                    for _ in 0..packet_count {
                        let mut packet: LogPacket = cmem::zeroed();
                        packet.header.flags |= LogPacketFlags::LittleEndian();
                        packet.header.severity = metadata.severity;
                        packet.header.verbosity = metadata.verbosity;
                        packets.push(packet);
                    }

                    if let Some(head_packet) = packets.get_mut(0) {
                        head_packet.header.flags |= LogPacketFlags::Head();

                        if let Ok(process_id) = svc::get_process_id(svc::CURRENT_PROCESS_PSEUDO_HANDLE) {
                            head_packet.header.process_id = process_id;
                        }

                        let cur_thread = thread::get_current_thread();
                        if let Ok(thread_id) = cur_thread.get_id() {
                            head_packet.header.thread_id = thread_id;
                        }

                        head_packet.payload.file_name = LogDataStringChunk::from(LogDataChunkKey::FileName, String::from(metadata.file_name));
                        head_packet.payload.function_name = LogDataStringChunk::from(LogDataChunkKey::FunctionName, String::from(metadata.fn_name));
                        head_packet.payload.line_number = LogDataChunk::<u32>::from(LogDataChunkKey::LineNumber, metadata.line_no);

                        // TODO: module name
                        head_packet.payload.module_name = LogDataStringChunk::from(LogDataChunkKey::ModuleName, String::from("aarch64-switch-rs"));
                        
                        let thread_name = match cur_thread.name.get_str() {
                            Ok(name) => name,
                            _ => "<unknown>",
                        };
                        head_packet.payload.thread_name = LogDataStringChunk::from(LogDataChunkKey::ThreadName, String::from(thread_name));

                        // TODO: Tick -> user system clock
                        // TODO: process name?
                    }

                    if let Some(tail_packet) = packets.get_mut(packet_count - 1) {
                        tail_packet.header.flags |= LogPacketFlags::Tail();
                    }

                    let mut remaining_len = metadata.msg.len();
                    let mut packet_i: usize = 0;
                    let mut text_log_offset: usize = 0;
                    while remaining_len > 0 {
                        let cur_len = match remaining_len > MAX_STRING_LEN {
                            true => MAX_STRING_LEN,
                            false => remaining_len,
                        };
                        let cur_log_str = &metadata.msg[text_log_offset..cur_len];
                        let packet = packets.get_mut(packet_i).unwrap();
                        packet.payload.text_log = LogDataStringChunk::from(LogDataChunkKey::TextLog, String::from(cur_log_str));
                        text_log_offset += cur_len;
                        remaining_len -= cur_len;
                        packet_i += 1;
                    }

                    for packet in packets.iter_mut() {
                        packet.header.payload_size = packet.compute_chunk_size() as u32;

                        let log_buf_size = packet.compute_chunk_size();
                        // Allocate the size of the LogPacket struct, since the struct holds the header + every possible chunk in the packet body with its full size, thus no packet will exceed that size.
                        // Check that the buffer size isn't bigger just in case, although I'm pretty sure that that will never happen.
                        if cmem::size_of::<LogPacket>() > log_buf_size {
                            let log_buf = alloc::alloc::alloc(alloc::alloc::Layout::new::<LogPacket>());
                            if !log_buf.is_null() {
                                let mut encode_buf = encode_payload_buf(log_buf, &packet.header);
                                encode_buf = encode_payload(encode_buf, &packet.payload.log_session_begin);
                                encode_buf = encode_payload(encode_buf, &packet.payload.log_session_end);
                                encode_buf = encode_payload(encode_buf, &packet.payload.text_log);
                                encode_buf = encode_payload(encode_buf, &packet.payload.line_number);
                                encode_buf = encode_payload(encode_buf, &packet.payload.file_name);
                                encode_buf = encode_payload(encode_buf, &packet.payload.function_name);
                                encode_buf = encode_payload(encode_buf, &packet.payload.module_name);
                                encode_buf = encode_payload(encode_buf, &packet.payload.thread_name);
                                encode_buf = encode_payload(encode_buf, &packet.payload.log_packet_drop_count);
                                encode_buf = encode_payload(encode_buf, &packet.payload.user_system_clock);
                                /* encode_buf = */ encode_payload(encode_buf, &packet.payload.process_name);

                                match &mut self.logger {
                                    Ok(ref mut logger) => {
                                        let _ = logger.get().log(sf::Buffer::from_mut(log_buf, log_buf_size));
                                    },
                                    _ => {}
                                }

                                alloc::alloc::dealloc(log_buf, alloc::alloc::Layout::new::<LogPacket>());
                            }
                        }
                    }
                }
            }
        }
    }
}