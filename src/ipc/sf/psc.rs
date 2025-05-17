use crate::ipc::sf;
use crate::version;

use nx_derive::{Request, Response};

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum ModuleId {
    Lm = 0x29,
    // TODO: more
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum State {
    FullAwake = 0,
    MinimumAwake = 1,
    SleepReady = 2,
    EssentialServicesSleepReady = 3,
    EssentialServicesAwake = 4,
    ShutdownReady = 5,
    Invalid = 6,
}

ipc_sf_define_default_client_for_interface!(PmModule);
ipc_sf_define_interface_trait! {
    trait PmModule {
        initialize [0, version::VersionInterval::all()]: (id: ModuleId, dependencies: sf::InMapAliasBuffer<ModuleId>) =>  (event_handle: sf::CopyHandle) (event_handle: sf::CopyHandle);
        get_request [1, version::VersionInterval::all()]: () => (state: State, flags: u32) (state: State, flags: u32);
        acknowledge [2, version::VersionInterval::all()]: () => () ();
        finalize [3, version::VersionInterval::all()]: () => () ();
        acknowledge_ex [4, version::VersionInterval::from(version::Version::new(5,1,0))]: (state: State) =>  () ();
    }
}

ipc_sf_define_default_client_for_interface!(PmService);
ipc_sf_define_interface_trait! {
    trait PmService {
        get_pm_module [0, version::VersionInterval::all()]: () => (pm_module: PmModule) (pm_module: session_type!(PmModule));
    }
}
