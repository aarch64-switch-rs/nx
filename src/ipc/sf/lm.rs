use crate::result::*;
use crate::ipc::sf;
use crate::mem;

bit_enum! {
    LogDestination (u32) {
        Tma = bit!(0),
        Uart = bit!(1),
        UartSleeping = bit!(2),
        All = 0xFFFF
    }
}

pub trait ILogger {
    ipc_cmif_interface_define_command!(log: (log_buf: sf::InAutoSelectBuffer<u8>) => ());
    ipc_cmif_interface_define_command!(set_destination: (log_destination: LogDestination) => ());
}

pub trait ILogService {
    ipc_cmif_interface_define_command!(open_logger: (process_id: sf::ProcessId) => (logger: mem::Shared<dyn sf::IObject>));
}