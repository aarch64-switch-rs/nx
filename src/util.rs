use crate::result::*;
use crate::results;
use crate::thread;
use crate::diag::assert;
use crate::diag::log;
use crate::diag::log::Logger;
use alloc::string::String;
use core::str;
use core::ptr;
use core::fmt;
use core::panic;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct Uuid {
    pub uuid: [u8; 0x10]
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct PointerAndSize {
    pub address: *mut u8,
    pub size: usize
}

impl PointerAndSize {
    pub const fn empty() -> Self {
        Self { address: ptr::null_mut(), size: 0 }
    }
    
    pub const fn new(address: *mut u8, size: usize) -> Self {
        Self { address: address, size: size }
    }

    pub fn is_valid(&self) -> bool {
        !self.address.is_null() && (self.size != 0)
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CString<const S: usize> {
    pub c_str: [u8; S]
}

impl<const S: usize> fmt::Debug for CString<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str_data = match self.get_str() {
            Ok(got_str) => got_str,
            Err(_) => "<empty>"
        };
        write!(f, "{}", str_data)
    }
}

impl<const S: usize> PartialEq for CString<S> {
    fn eq(&self, other: &Self) -> bool {
        if let Ok(self_str) = self.get_str() {
            if let Ok(other_str) = other.get_str() {
                return self_str == other_str;
            }
        }
        false
    }
}

impl<const S: usize> Eq for CString<S> {}

impl<const S: usize> Default for CString<S> {
    fn default() -> Self {
        Self::new()
    }
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
        Ok(String::from(Self::read_str_from(ptr, ptr_len)?))
    }

    pub fn set_str(&mut self, string: &str) -> Result<()> {
        Self::copy_str_to(string, &mut self.c_str as *mut _ as *mut u8, S)
    }

    pub fn set_string(&mut self, string: String) -> Result<()> {
        Self::copy_string_to(string, &mut self.c_str as *mut _ as *mut u8, S)
    }

    pub fn get_str(&self) -> Result<&'static str> {
        Self::read_str_from(&self.c_str as *const _ as *const u8, S)
    }

    pub fn get_string(&self) -> Result<String> {
        Self::read_string_from(&self.c_str as *const _ as *const u8, S)
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CString16<const S: usize> {
    pub c_str: [u16; S]
}

impl<const S: usize> fmt::Debug for CString16<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Ok(string) = self.get_string() {
            write!(f, "{}", string)
        }
        else {
            write!(f, "<null>")
        }
    }
}

impl<const S: usize> PartialEq for CString16<S> {
    fn eq(&self, other: &Self) -> bool {
        if let Ok(self_str) = self.get_string() {
            if let Ok(other_str) = other.get_string() {
                return self_str == other_str;
            }
        }
        false
    }
}

impl<const S: usize> Eq for CString16<S> {}

impl<const S: usize> Default for CString16<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const S: usize> CString16<S> {
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

    fn copy_str_to(string: &str, ptr: *mut u16, ptr_len: usize) -> Result<()> {
        let mut encode_buf: [u16; 2] = [0; 2];
        let mut i: isize = 0;
        unsafe {
            ptr::write_bytes(ptr, 0, ptr_len);
            for ch in string.chars() {
                let enc = ch.encode_utf16(&mut encode_buf);
                *ptr.offset(i) = enc[0];

                i += 1;
                if i as usize > (ptr_len - 1) {
                    break;
                }
            }
        }
        Ok(())
    }
    
    fn read_string_from(ptr: *const u16, ptr_len: usize) -> Result<String> {
        let mut string = String::new();
        unsafe {
            let tmp_slice = core::slice::from_raw_parts(ptr, ptr_len);
            for ch_v in core::char::decode_utf16(tmp_slice.iter().cloned()) {
                if let Ok(ch) = ch_v {
                    if ch == '\0' {
                        break;
                    }
                    string.push(ch);
                }
                else {
                    break;
                }
            }
        }
        Ok(string)
    }

    pub fn set_str(&mut self, string: &str) -> Result<()> {
        Self::copy_str_to(string, &mut self.c_str as *mut _ as *mut u16, S)
    }

    pub fn set_string(&mut self, string: String) -> Result<()> {
        self.set_str(string.as_str())
    }

    pub fn get_string(&self) -> Result<String> {
        Self::read_string_from(&self.c_str as *const _ as *const u16, S)
    }
}

pub fn simple_panic_handler<L: Logger>(info: &panic::PanicInfo, assert_mode: assert::AssertMode) -> ! {
    let thread_name = match thread::get_current_thread().name.get_str() {
        Ok(name) => name,
        _ => "<unknown>",
    };
    diag_log!(L { log::LogSeverity::Fatal, true } => "Panic! at thread '{}' -> {}\n", thread_name, info);

    assert::assert(assert_mode, results::lib::assert::ResultAssertionFailed::make());
    loop {}
}