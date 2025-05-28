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
#[nx_derive::ipc_trait]
pub trait Logger {
    #[ipc_rid(0)]
    fn log(&self, log_buf: sf::InAutoSelectBuffer<u8>);
    #[ipc_rid(1)]
    #[version(version::VersionInterval::from(version::Version::new(3, 0, 0)))]
    fn set_destination(&mut self, log_destination: LogDestination);
}

//ipc_sf_define_default_client_for_interface!(LogService);
#[nx_derive::ipc_trait]
pub trait Logging {
    #[ipc_rid(0)]
    #[return_session]
    fn open_logger(&mut self, process_id: sf::ProcessId) -> Logger;
}
