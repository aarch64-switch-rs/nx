use core::result;
use core::fmt;

const MODULE_BITS: u32 = 9;
const DESCRIPTION_BITS: u32 = 13;
const DEFAULT_VALUE: u32 = 0;
const SUCCESS_VALUE: u32 = DEFAULT_VALUE;

const fn pack_value(module: u32, description: u32) -> u32 {
    module | (description << MODULE_BITS)
}

const fn unpack_module(value: u32) -> u32 {
    value & !(!DEFAULT_VALUE << MODULE_BITS)
}

const fn unpack_description(value: u32) -> u32 {
    (value >> MODULE_BITS) & !(!DEFAULT_VALUE << DESCRIPTION_BITS)
}

#[derive(Copy, Clone, PartialEq, Eq, Default)]
#[repr(C)]
pub struct ResultCode {
    value: u32
}

impl ResultCode {
    pub const fn new(value: u32) -> Self {
        Self { value }
    }

    pub const fn new_err<T>(value: u32) -> Result<T> {
        Err(Self::new(value))
    }
    
    pub const fn is_success(&self) -> bool {
        self.value == SUCCESS_VALUE
    }
    
    pub const fn is_failure(&self) -> bool {
        !self.is_success()
    }
    
    pub const fn get_value(&self) -> u32 {
        self.value
    }
    
    pub const fn get_module(&self) -> u32 {
        unpack_module(self.value)
    }
    
    pub const fn get_description(&self) -> u32 {
        unpack_description(self.value)
    }
}

impl fmt::Debug for ResultCode {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        write!(fmt, "{:#X}", self.value)
    }
}

impl fmt::Display for ResultCode {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        write!(fmt, "{:0>4}-{:0>4}", 2000 + self.get_module(), self.get_description())
    }
}

pub type Result<T> = result::Result<T, ResultCode>;

#[inline(always)]
pub fn pack<T>(rc: ResultCode, value: T) -> Result<T> {
    if rc.is_success() {
        Ok(value)
    }
    else {
        Err(rc)
    }
}

#[inline(always)]
pub fn unpack<T>(rc: &Result<T>) -> ResultCode {
    match rc {
        Ok(_) => ResultSuccess::make(),
        Err(rc) => *rc
    }
}

pub trait ResultBase {
    fn get_module() -> u32;
    fn get_description() -> u32;

    #[inline(always)]
    fn get_value() -> u32 {
        pack_value(Self::get_module(), Self::get_description())
    }

    #[inline(always)]
    fn make() -> ResultCode {
        ResultCode::new(Self::get_value())
    }

    #[inline(always)]
    fn make_err<T>() -> Result<T> {
        Err(Self::make())
    }

    #[inline(always)]
    fn matches(rc: ResultCode) -> bool {
        rc.get_value() == Self::get_value()
    }
}

result_define!(Success: 0, 0);