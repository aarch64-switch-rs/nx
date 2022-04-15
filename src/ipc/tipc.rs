use super::*;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum CommandType {
    Invalid = 0,
    CloseSession = 15
}

pub mod client;

pub mod server;