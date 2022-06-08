use crate::result::*;
use crate::ipc::sf;
use crate::mem;
use crate::version;

define_bit_enum! {
    LogDestination (u32) {
        Tma = bit!(0),
        Uart = bit!(1),
        UartSleeping = bit!(2),
        All = 0xFFFF
    }
}

ipc_sf_define_interface_trait! {
    trait ILogger {
        log [0, version::VersionInterval::all()]: (log_buf: sf::InAutoSelectBuffer<u8>) => ();
        set_destination [1, version::VersionInterval::from(version::Version::new(3,0,0))]: (log_destination: LogDestination) => ();
    }
}

ipc_sf_define_interface_trait! {
    trait ILogService {
        open_logger [0, version::VersionInterval::all()]: (process_id: sf::ProcessId) => (logger: mem::Shared<dyn ILogger>);
    }
}