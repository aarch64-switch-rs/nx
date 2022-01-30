use crate::svc;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u64)]
pub enum FunctionId {
    Invalid = 0,
    GenerateRandomBytes = 0xC3000006
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct Input {
    pub function_id: FunctionId,
    pub arguments: [u64; 7],
}
const_assert!(core::mem::size_of::<Input>() == 0x40);

impl Input {
    pub const fn new(function_id: FunctionId) -> Self {
        Self { function_id, arguments: [0; 7] }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(u64)]
pub enum Result {
    #[default]
    Success = 0,
    NotImplemented = 1,
    InvalidArgument = 2,
    InProgress = 3,
    NoAsyncOperation = 4,
    InvalidAsyncOperation = 5,
    NotPermitted = 6
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct Output {
    pub result: Result,
    pub arguments: [u64; 7],
}
const_assert!(core::mem::size_of::<Output>() == 0x40);

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct Arguments {
    pub arguments: [u64; 8]
}

impl Arguments {
    pub fn from_input(input: Input) -> Self {
        unsafe {
            core::mem::transmute(input)
        }
    }

    pub fn to_output(self) -> Output {
        unsafe {
            core::mem::transmute(self)
        }
    }
}

pub const GENERATE_RANDOM_BYTES_MAX_SIZE: usize = 0x38;

pub fn generate_random_bytes(dst: *mut u8, size: usize) -> Result {
    let mut input = Input::new(FunctionId::GenerateRandomBytes);
    input.arguments[0] = size as u64;

    let output = svc::call_secure_monitor(input);
    if output.result == Result::Success {
        unsafe {
            core::ptr::copy(output.arguments.as_ptr().offset(1) as *const u8, dst, size);
        }
    }
    output.result
}