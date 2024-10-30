//! Common library utilities

use crate::result::*;
use crate::thread;
use crate::diag::abort;
use crate::diag::log;
use crate::diag::log::Logger;
use alloc::string::String;
use alloc::string::ToString;
use core::str;
use core::ptr;
use core::fmt;
use core::panic;

pub mod rc;

#[doc(hidden)]
pub trait AsInner<Inner: ?Sized> {
    fn as_inner(&self) -> &Inner;
}

#[doc(hidden)]
#[allow(dead_code)] // not used on all platforms
pub trait AsInnerMut<Inner: ?Sized> {
    fn as_inner_mut(&mut self) -> &mut Inner;
}

#[doc(hidden)]
pub trait IntoInner<Inner> {
    fn into_inner(self) -> Inner;
}
/// Represents a 16-byte UUID
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct Uuid {
    /// The UUID byte array
    pub uuid: [u8; 0x10]
}
//api_mark_request_command_parameters_types_as_copy!(Uuid);

/// Represents a pair of a pointer and a size
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct PointerAndSize {
    /// The pointer address
    pub address: *mut u8,
    /// The pointer size
    pub size: usize
}

impl PointerAndSize {
    /// Creates an empty, thus invalid [`PointerAndSize`] (with a null pointer and size `0`)
    #[inline]
    pub const fn empty() -> Self {
        Self { address: ptr::null_mut(), size: 0 }
    }

    /// Creates a [`PointerAndSize`]
    /// 
    /// # Arguments
    /// 
    /// * `address`: The address
    /// * `size`: The size
    #[inline]
    pub const fn new(address: *mut u8, size: usize) -> Self {
        Self { address, size }
    }

    /// Checks whether the [`PointerAndSize`] is valid
    /// 
    /// Essentially, this checks that the pointer isn't null and that the size is non-zero
    #[inline]
    pub fn is_valid(&self) -> bool {
        !self.address.is_null() && (self.size != 0)
    }
}

/// Represents a pair of a pointer and a size
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct ConstPointerAndSize {
    /// The pointer address
    pub address: *const u8,
    /// The pointer size
    pub size: usize
}

impl ConstPointerAndSize {
    /// Creates an empty, thus invalid [`ConstPointerAndSize`] (with a null pointer and size `0`)
    #[inline]
    pub const fn empty() -> Self {
        Self { address: ptr::null(), size: 0 }
    }

    /// Creates a [`ConstPointerAndSize`]
    /// 
    /// # Arguments
    /// 
    /// * `address`: The address
    /// * `size`: The size
    #[inline]
    pub const fn new(address: *const u8, size: usize) -> Self {
        Self { address, size }
    }

    /// Checks whether the [`PointerAndSize`] is valid
    /// 
    /// Essentially, this checks that the pointer isn't null and that the size is non-zero
    #[inline]
    pub fn is_valid(&self) -> bool {
        !self.address.is_null() && (self.size != 0)
    }
}

pub(crate) const fn const_usize_min(a: usize, b: usize) -> usize {
    // TODO: const min traits
    if a > b { b } else { a }
}
pub(crate) const fn const_usize_max(a: usize, b: usize) -> usize {
    // TODO: const min traits
    if a < b { b } else { a }
}

/// Represents a C-like string of a given size (mostly like a C `char[S]` array)
/// 
/// Note that `char` is 4-bytes in Rust for encoding reasons, thus we must stick to `u8` arrays
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ArrayString<const S: usize> {
    /// The actual array (like `char[S]` in C)
    pub c_str: [u8; S]
}
impl<const S: usize> crate::ipc::server::RequestCommandParameter<ArrayString<S>> for ArrayString<S> {
    fn after_request_read(ctx: &mut crate::ipc::server::ServerContext) -> Result<Self> {
        Ok(ctx.raw_data_walker.advance_get())
    }
}

impl<const S: usize> crate::ipc::server::ResponseCommandParameter for ArrayString<S> {
        fn before_response_write(_raw: &Self, ctx: &mut crate::ipc::server::ServerContext) -> Result<()> {
        ctx.raw_data_walker.advance::<Self>();
        Ok(())
    }

    fn after_response_write(raw: &Self, ctx: &mut crate::ipc::server::ServerContext) -> Result<()> {
        ctx.raw_data_walker.advance_set(*raw);
        Ok(())
    }
}

impl<const S: usize> crate::ipc::client::RequestCommandParameter for ArrayString<S> {
    fn before_request_write(_raw: &Self, walker: &mut crate::ipc::DataWalker, _ctx: &mut crate::ipc::CommandContext) -> crate::result::Result<()> {
        walker.advance::<Self>();
        Ok(())
    }

    fn before_send_sync_request(raw: &Self, walker: &mut crate::ipc::DataWalker, _ctx: &mut crate::ipc::CommandContext) -> crate::result::Result<()> {
        walker.advance_set(*raw);
        Ok(())
    }
}


impl<const S: usize> crate::ipc::client::ResponseCommandParameter<ArrayString<S>> for ArrayString<S> {
    fn after_response_read(walker: &mut crate::ipc::DataWalker, _ctx: &mut crate::ipc::CommandContext) -> crate::result::Result<Self> {
        Ok(walker.advance_get())
    }
}

impl<const S: usize> fmt::Debug for ArrayString<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str_data = self.get_str().unwrap_or("<invalid-str>");
        write!(f, "{}", str_data)
    }
}

impl<const S: usize> PartialEq for ArrayString<S> {
    fn eq(&self, other: &Self) -> bool {
        if let Ok(self_str) = self.get_str() {
            if let Ok(other_str) = other.get_str() {
                return self_str == other_str;
            }
        }
        false
    }
}

impl<const S: usize> Eq for ArrayString<S> {}

impl<const S: usize> Default for ArrayString<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const S: usize> ArrayString<S> {
    /// Creates an empty [`CString`]
    pub const fn new() -> Self {
        Self { c_str: [0; S] }
    }

    /// Creates a [`CString`] from a given byte array
    /// 
    /// # Arguments
    /// 
    /// * `raw_bytes`: Byte array to use
    pub const fn from_raw(raw_bytes: [u8; S]) -> Self {
        Self { c_str: raw_bytes }
    }

    /// Creates a [`CString`] from a given `&str`
    /// 
    /// This creates an empty [`CString`] and calls [`CString::set_str`] on it
    /// 
    /// # Arguments
    /// 
    /// * `string`: The `&str` to use
    pub const fn from_str(string: &str) -> Self {
        let mut cstr = Self::new();
        cstr.set_str(string);
        cstr
    }

    /// Creates a [`CString`] from a given `String`
    /// 
    /// This creates an empty [`CString`] and calls [`CString::set_string`] on it
    /// 
    /// # Arguments
    /// 
    /// * `string`: The `String` to use
    pub fn from_string(string: String) -> Self {
        let mut cstr = Self::new();
        cstr.set_string(string);
        cstr
    }

    const fn copy_str_to(string: &str, ptr: *mut u8, ptr_len: usize) {
        unsafe {
            ptr::write_bytes(ptr, 0, ptr_len);
            if !string.is_empty() {
                ptr::copy(string.as_ptr(), ptr, const_usize_min(string.as_bytes().len(), ptr_len - 1));
            }
        }
    }
    
    fn copy_string_to(string: String, ptr: *mut u8, ptr_len: usize) {
        unsafe {
            ptr::write_bytes(ptr, 0, ptr_len);
            if !string.is_empty() {
                ptr::copy(string.as_ptr(), ptr, core::cmp::min(ptr_len - 1, string.len()));
            }
        }
    }

    fn read_str_from(ptr: *const u8, str_len: usize) -> Result<&'static str> {
        if str_len == 0 {
            Ok("")
        }
        else {
            unsafe {
                match core::str::from_utf8(core::slice::from_raw_parts(ptr, str_len)) {
                    Ok(name) => Ok(name.trim_end_matches('\0')),
                    Err(_) => rc::ResultInvalidUtf8Conversion::make_err()
                }
            }
        }
    }
    
    fn read_string_from(ptr: *const u8, str_len: usize) -> Result<String> {
        Ok(String::from(Self::read_str_from(ptr, str_len)?))
    }

    /// Returns the length of the [`CString`]
    /// 
    /// This is similar to C's `strlen()` function, thus taking into account the string's NUL-termination
    pub fn len(&self) -> usize {
        for i in 0..S {
            if self.c_str[i] == 0 {
                return i;
            }
        }

        S
    }

    /// Returns whether this [`CString`] is empty
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Sets a `&str` as the contents of this [`CString`]
    /// 
    /// This will copy at max `S - 1` bytes/chars in order to ensure that the string is NUL-terminated
    /// 
    /// # Arguments
    /// 
    /// * `string`: The `&str` to set
    pub const fn set_str(&mut self, string: &str) {
        Self::copy_str_to(string, &mut self.c_str as *mut _ as *mut u8, S)
    }

    /// Sets a `String` as the contents of this [`CString`]
    /// 
    /// This will copy at max `S - 1` bytes/chars in order to ensure that the string is NUL-terminated
    /// 
    /// # Arguments
    /// 
    /// * `string`: The `String` to set
    pub fn set_string(&mut self, string: String) {
        Self::copy_string_to(string, &mut self.c_str as *mut _ as *mut u8, S)
    }

    /// Gets a `&str` corresponding to this [`CString`]
    pub fn get_str(&self) -> Result<&'static str> {
        Self::read_str_from(&self.c_str as *const _ as *const u8, self.len())
    }

    /// Gets a `String` corresponding to this [`CString`]
    pub fn get_string(&self) -> Result<String> {
        Self::read_string_from(&self.c_str as *const _ as *const u8, self.len())
    }
}

impl<S: AsRef<str>, const LEN: usize> From<S> for ArrayString<LEN> {
    fn from(value: S) -> Self {
        let reffed_val: &str = value.as_ref();
        Self::from_str(reffed_val)
    }
}

/// Represents a C-like 16-bit string of a given size (mostly like a C `char16_t[S]` array)
/// 
/// Note that `char` is 4-bytes in Rust for encoding reasons, thus we must stick to `u16` arrays
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ArrayWideString<const S: usize> {
    /// The actual array (like `char16_t[S]` in C)
    pub c_str: [u16; S]
}

impl<const S: usize> crate::ipc::server::RequestCommandParameter<ArrayWideString<S>> for ArrayWideString<S> {
    fn after_request_read(ctx: &mut crate::ipc::server::ServerContext) -> Result<Self> {
        Ok(ctx.raw_data_walker.advance_get())
    }
}

impl<const S: usize> crate::ipc::server::ResponseCommandParameter for ArrayWideString<S> {
        fn before_response_write(_raw: &Self, ctx: &mut crate::ipc::server::ServerContext) -> Result<()> {
        ctx.raw_data_walker.advance::<Self>();
        Ok(())
    }

    fn after_response_write(raw: &Self, ctx: &mut crate::ipc::server::ServerContext) -> Result<()> {
        ctx.raw_data_walker.advance_set(*raw);
        Ok(())
    }
}

impl<const S: usize> crate::ipc::client::RequestCommandParameter for ArrayWideString<S> {
    fn before_request_write(_raw: &Self, walker: &mut crate::ipc::DataWalker, _ctx: &mut crate::ipc::CommandContext) -> crate::result::Result<()> {
        walker.advance::<Self>();
        Ok(())
    }

    fn before_send_sync_request(raw: &Self, walker: &mut crate::ipc::DataWalker, _ctx: &mut crate::ipc::CommandContext) -> crate::result::Result<()> {
        walker.advance_set(*raw);
        Ok(())
    }
}


impl<const S: usize> crate::ipc::client::ResponseCommandParameter<ArrayWideString<S>> for ArrayWideString<S> {
    fn after_response_read(walker: &mut crate::ipc::DataWalker, _ctx: &mut crate::ipc::CommandContext) -> crate::result::Result<Self> {
        Ok(walker.advance_get())
    }
}

impl<const S: usize> fmt::Debug for ArrayWideString<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str_data = self.get_string().unwrap_or("<invalid-str>".to_string());
        write!(f, "{}", str_data)
    }
}

impl<const S: usize> PartialEq for ArrayWideString<S> {
    fn eq(&self, other: &Self) -> bool {
        if let Ok(self_str) = self.get_string() {
            if let Ok(other_str) = other.get_string() {
                return self_str == other_str;
            }
        }
        false
    }
}

impl<const S: usize> Eq for ArrayWideString<S> {}

impl<const S: usize> Default for ArrayWideString<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const S: usize> ArrayWideString<S> {
    /// Creates an empty [`CString16`]
    pub const fn new() -> Self {
        Self { c_str: [0; S] }
    }

    /// Creates a [`CString16`] from a given byte array
    /// 
    /// # Arguments
    /// 
    /// * `raw_bytes`: Byte array to use
    pub const fn from_raw(raw_bytes: [u16; S]) -> Self {
        Self { c_str: raw_bytes }
    }

    /// Creates a [`CString16`] from a given `String`
    /// 
    /// This creates an empty [`CString16`] and calls [`CString16::set_string`] on it
    /// 
    /// # Arguments
    /// 
    /// * `string`: The `String` to use
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
    
    fn read_string_from(ptr: *const u16, str_len: usize) -> Result<String> {
        let mut string = String::new();
        if str_len > 0 {
            unsafe {
                let tmp_slice = core::slice::from_raw_parts(ptr, str_len);
                for ch_v in core::char::decode_utf16(tmp_slice.iter().cloned()) {
                    if let Ok(ch) = ch_v {
                        string.push(ch);
                    }
                    else {
                        break;
                    }
                }
            }
        }
        Ok(string)
    }

    /// Returns the length of the [`CString16`]
    /// 
    /// This is similar to C's `strlen()` function, thus taking into account the string's NUL-termination
    pub fn len(&self) -> usize {
        for i in 0..S {
            if self.c_str[i] == 0 {
                return i;
            }
        }

        S
    }

    /// Returns if this [`CString16`] is empty
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Sets a `&str` as the contents of this [`CString16`]
    /// 
    /// This will copy at max `S - 1` bytes/chars in order to ensure that the string is NUL-terminated
    /// 
    /// # Arguments
    /// 
    /// * `string`: The `&str` to set
    pub fn set_str(&mut self, string: &str) -> Result<()> {
        Self::copy_str_to(string, &mut self.c_str as *mut _ as *mut u16, S)
    }

    /// Sets a `String` as the contents of this [`CString16`]
    /// 
    /// This will copy at max `S - 1` bytes/chars in order to ensure that the string is NUL-terminated
    /// 
    /// # Arguments
    /// 
    /// * `string`: The `String` to set
    pub fn set_string(&mut self, string: String) -> Result<()> {
        self.set_str(string.as_str())
    }

    /// Gets a `String` corresponding to this [`CString16`]
    pub fn get_string(&self) -> Result<String> {
        Self::read_string_from(&self.c_str as *const _ as *const u16, self.len())
    }

    /// Returns a copy of this [`CString16`] but with all chars byte-swapped
    /// 
    /// Essentially, this calls `swap_bytes()` on all the string copy array elements
    pub fn swap_chars(&self) -> Self {
        let mut self_copy = *self;

        for i in 0..S {
            self_copy.c_str[i] = self.c_str[i].swap_bytes();
        }

        self_copy
    }
}

impl<const S: usize> core::str::FromStr for ArrayWideString<S> {
    type Err = ResultCode;
    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        let mut cstr = Self::new();
        cstr.set_str(s)?;
        Ok(cstr)
    }
}

/// Same as C's `strlen()`
/// 
/// # Arguments
/// 
/// * `str_ptr`: The `const char*`-like ptr to use
/// # SAFETY: There must be a null byte present in the string, or at some point after the pointer and within valid memory. This function will read infinitely until a null is read or crash occurs.
pub unsafe fn str_ptr_len(str_ptr: *const u8) -> usize {
    (0usize..)
    .find(|&offset| (*str_ptr.add(offset)) == 0)
    .expect("There will be a null byte (or crash) eventually")
}

/// Simplified panic handler using a provided [`Logger`] type, available as a helpful default panic handler
/// 
/// This handler does the following:
/// * Logs the panic information via [`diag_log!`] macro and the provided [`Logger`] type
/// * Aborts with [`ResultPanicked`][`super::rc::ResultPanicked`] and the specified desired [`AbortLevel`][`abort::AbortLevel`]
/// 
/// # Arguments
/// 
/// * `info`: `PanicInfo` object got from the actual panic handler
/// * `desired_level`: Desired [`AbortLevel`][`abort::AbortLevel`] to abort with
pub fn simple_panic_handler<L: Logger>(info: &panic::PanicInfo, desired_level: abort::AbortLevel) -> ! {
    let thread_name = match unsafe {thread::current().as_ref()}.map(|t|t.name.get_str()) {
        Some(Ok(name)) => name,
        _ => "<unknown>",
    };
    diag_log!(L { log::LogSeverity::Fatal, true } => "Panic! at thread '{}' -> {}\n", thread_name, info);

    abort::abort(desired_level, super::rc::ResultPanicked::make())
}