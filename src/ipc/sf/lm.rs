use crate::ipc::sf;
use crate::result::*;
use crate::version;

define_bit_enum! {
    LogDestination (u32) {
        Tma = bit!(0),
        Uart = bit!(1),
        UartSleeping = bit!(2),
        All = 0xFFFF
    }
}

ipc_sf_define_default_interface_client!(Logger);
ipc_sf_define_interface_trait! {
    trait Logger {
        log [0, version::VersionInterval::all()]: (log_buf: sf::InAutoSelectBuffer<u8>) =>  () ();
        set_destination [1, version::VersionInterval::from(version::Version::new(3,0,0)), mut]: (log_destination: LogDestination) =>  () ();
    }
}

ipc_sf_define_default_interface_client!(LogService);
ipc_sf_define_interface_trait! {
    trait LogService {
        open_logger [0, version::VersionInterval::all(), mut]: (raw_process_id: u64) =>  (logger: Logger) (logger: session_type!(Logger));
    }
}
