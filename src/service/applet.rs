use core::sync::atomic::AtomicU64;

use crate::ipc::sf;
use crate::ipc::sf::sm;
use crate::service;
use crate::{result::*, svc};

pub use crate::ipc::sf::applet::*;

pub static GLOBAL_ARUID: AtomicU64 = AtomicU64::new(0);

pub(crate) fn initialize() -> Result<()> {
    let service = service::new_service_object::<AllSystemAppletProxiesService>()?;
    let applet_attr = AppletAttribute::default();
    let applet_proxy = service.open_library_applet_proxy(
        sf::ProcessId::new(),
        sf::CopyHandle::from(svc::CURRENT_PROCESS_PSEUDO_HANDLE),
        sf::InMapAliasBuffer::from_other_var(&applet_attr),
    )?;
    let window_controller = applet_proxy.get_window_controller()?;
    let aruid = window_controller.get_applet_resource_user_id()?;

    GLOBAL_ARUID.store(aruid, core::sync::atomic::Ordering::Release);

    Ok(())
}

impl service::IService for AllSystemAppletProxiesService {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("appletAE")
    }

    fn as_domain() -> bool {
        true
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}
