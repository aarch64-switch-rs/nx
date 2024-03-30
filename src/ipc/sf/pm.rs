use crate::result::*;
use crate::version;

ipc_sf_define_interface_trait! {
    trait IInformationInterface {
        get_program_id [0, version::VersionInterval::all()]: (process_id: u64) => (program_id: u64);
    }
}

ipc_sf_define_interface_trait! {
    trait IDebugMonitorInterface {
        get_application_process_id_deprecated [5, version::VersionInterval::to(version::Version::new(4,1,0))]: () => (process_id: u64);
        get_application_process_id [4, version::VersionInterval::from(version::Version::new(5,0,0))]: () => (process_id: u64);
    }
}