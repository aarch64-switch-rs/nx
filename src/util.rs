use crate::result::*;
use crate::results;
use crate::thread;
use crate::diag::assert;
use crate::diag::log;
use crate::diag::log::Logger;
use alloc::string::String;
use core::str;
use core::ptr;
use core::panic;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct PointerAndSize {
    pub address: *mut u8,
    pub size: usize
}

impl PointerAndSize {
    pub const fn new(address: *mut u8, size: usize) -> Self {
        Self { address: address, size: size }
    }

    pub fn is_valid(&self) -> bool {
        !self.address.is_null() && (self.size != 0)
    }
}

fn copy_str_to(string: &str, ptr: *mut u8, ptr_len: usize) -> Result<()> {
    unsafe {
        ptr::write_bytes(ptr, 0, ptr_len);
        if !string.is_empty() {
            ptr::copy(string.as_ptr(), ptr, core::cmp::min(ptr_len - 1, string.len()));
        }
    }
    Ok(())
}

fn copy_string_to(string: String, ptr: *mut u8, ptr_len: usize) -> Result<()> {
    unsafe {
        ptr::write_bytes(ptr, 0, ptr_len);
        if !string.is_empty() {
            ptr::copy(string.as_ptr(), ptr, core::cmp::min(ptr_len - 1, string.len()));
        }
    }
    Ok(())
}

fn read_str_from(ptr: *const u8, ptr_len: usize) -> Result<&'static str> {
    unsafe {
        match core::str::from_utf8(core::slice::from_raw_parts(ptr, ptr_len)) {
            Ok(name) => Ok(name.trim_matches('\0')),
            Err(_) => Err(results::lib::util::ResultInvalidConversion::make())
        }
    }
}

fn read_string_from(ptr: *const u8, ptr_len: usize) -> Result<String> {
    Ok(String::from(read_str_from(ptr, ptr_len)?))
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CString<const S: usize> {
    pub c_str: [u8; S]
}

impl<const S: usize> CString<S> {
    pub const fn new() -> Self {
        Self { c_str: [0; S] }
    }

    pub fn from_str(string: &str) -> Result<Self> {
        let mut cstr = Self::new();
        cstr.set_str(string)?;
        Ok(cstr)
    }

    pub fn from_string(string: String) -> Result<Self> {
        let mut cstr = Self::new();
        cstr.set_string(string)?;
        Ok(cstr)
    }

    pub fn set_str(&mut self, string: &str) -> Result<()> {
        copy_str_to(string, &mut self.c_str as *mut _ as *mut u8, S)
    }

    pub fn set_string(&mut self, string: String) -> Result<()> {
        copy_string_to(string, &mut self.c_str as *mut _ as *mut u8, S)
    }

    pub fn get_str(&self) -> Result<&'static str> {
        read_str_from(&self.c_str as *const _ as *const u8, S)
    }

    pub fn get_string(&self) -> Result<String> {
        read_string_from(&self.c_str as *const _ as *const u8, S)
    }
}

pub fn on_panic_handler<L: Logger>(info: &panic::PanicInfo, assert_mode: assert::AssertMode, rc: ResultCode) -> ! {
    let thread_name = match thread::get_current_thread().name.get_str() {
        Ok(name) => name,
        _ => "<unknown>",
    };
    diag_log!(L { log::LogSeverity::Fatal, true } => "Panic! at thread '{}' -> {}", thread_name, info);
    assert::assert(assert_mode, rc)
}