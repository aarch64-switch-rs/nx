use crate::result::*;
use crate::ipc::cmif::sf;
use crate::mem;

bit_enum! {
    LogDestination (u32) {
        TMA = bit!(0),
        UART = bit!(1),
        UARTSleeping = bit!(2),
        All = 0xFFFF
    }
}

pub trait ILogger {
    ipc_cmif_interface_define_command!(log: (log_buf: sf::InAutoSelectBuffer) => ());
    ipc_cmif_interface_define_command!(set_destination: (log_destination: LogDestination) => ());
}

pub trait ILogService {
    ipc_cmif_interface_define_command!(open_logger: (process_id: sf::ProcessId) => (logger: mem::Shared<dyn sf::IObject>));
}