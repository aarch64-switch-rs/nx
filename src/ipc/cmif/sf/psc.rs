use crate::result::*;
use crate::ipc::cmif::sf;
use crate::mem;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum ModuleId {
    Lm = 41
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum State {
    Awake,
    ReadyAwaken,
    ReadySleep,
    ReadySleepCritical,
    ReadyAwakenCritical,
    ReadyShutdown
}

pub trait IPmModule {
    nipc_cmif_interface_define_command!(initialize: (id: ModuleId, dependencies: sf::InMapAliasBuffer) => (event_handle: sf::CopyHandle));
    nipc_cmif_interface_define_command!(get_request: () => (state: State, flags: u32));
    nipc_cmif_interface_define_command!(acknowledge: () => ());
    nipc_cmif_interface_define_command!(finalize: () => ());
    nipc_cmif_interface_define_command!(acknowledge_ex: (state: State) => ());
}

pub trait IPmService {
    nipc_cmif_interface_define_command!(get_pm_module: () => (pm_module: mem::Shared<dyn sf::IObject>));
}