use crate::result::*;
use crate::ipc::sf;
use crate::version;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum ModuleId {
    Lm = 0x29,
    // TODO: more
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum State {
    FullAwake = 0,
    MinimumAwake = 1,
    SleepReady = 2,
    EssentialServicesSleepReady = 3,
    EssentialServicesAwake = 4,
    ShutdownReady = 5,
    Invalid = 6
}

//api_mark_request_command_parameters_types_as_copy!(ModuleId, State);

ipc_sf_define_default_interface_client!(PmModule);
ipc_sf_define_interface_trait! {
	trait PmModule {
        initialize [0, version::VersionInterval::all()]: (id: ModuleId, dependencies: sf::InMapAliasBuffer<ModuleId>) => (event_handle: sf::CopyHandle);
        get_request [1, version::VersionInterval::all()]: () => (state: State, flags: u32);
        acknowledge [2, version::VersionInterval::all()]: () => ();
        finalize [3, version::VersionInterval::all()]: () => ();
        acknowledge_ex [4, version::VersionInterval::from(version::Version::new(5,1,0))]: (state: State) => ();
    }
}

ipc_sf_define_default_interface_client!(PmService);
ipc_sf_define_interface_trait! {
	trait PmService {
        get_pm_module [0, version::VersionInterval::all()]: () => (pm_module: PmModule);
    }
}