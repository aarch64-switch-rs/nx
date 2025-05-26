use crate::ipc::sf;
use crate::version;

define_bit_enum! {
    LogDestination (u32) {
        Tma = bit!(0),
        Uart = bit!(1),
        UartSleeping = bit!(2),
        All = 0xFFFF
    }
}

ipc_sf_define_default_client_for_interface!(Logger);
ipc_sf_define_interface_trait! {
    trait Logger {
        log [0, version::VersionInterval::all()]: (log_buf: sf::InAutoSelectBuffer<u8>) => ();
        set_destination [1, version::VersionInterval::from(version::Version::new(3,0,0)), mut]: (log_destination: LogDestination) => ();
    }
}

//ipc_sf_define_default_client_for_interface!(LogService);
ipc_sf_define_interface_trait! {
    trait Logging {
        open_logger [0, version::VersionInterval::all(), mut]: (raw_process_id: u64) =>  (logger: Logger | session_type!(Logger) );
    }
}

