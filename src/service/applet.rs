use core::sync::atomic::AtomicU64;

use crate::ipc::sf;
use crate::ipc::sf::sm;
use crate::service;
use crate::sync::{ReadGuard, RwLock};
use crate::{result::*, svc};

pub use crate::ipc::sf::applet::*;

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

static ALL_SYSTEM_APPLET_PROXY_SERVICE: RwLock<Option<AllSystemAppletProxiesService>> =
    RwLock::new(None);
static LIBRARY_APPLET_PROXY: RwLock<Option<LibraryAppletProxy>> = RwLock::new(None);
static WINDOW_CONTROLLER: RwLock<Option<WindowController>> = RwLock::new(None);
pub static GLOBAL_ARUID: AtomicU64 = AtomicU64::new(0);

/// Attempts to initialize the module, or returns if the module has already been initialized.
#[inline]
pub fn initialize() -> Result<()> {
    let mut app_proxy_service_guard = ALL_SYSTEM_APPLET_PROXY_SERVICE.write();
    if app_proxy_service_guard.is_some() {
        //already initialized
        return Ok(());
    }

    let app_proxy_service = service::new_service_object::<AllSystemAppletProxiesService>()?;

    let applet_attr = AppletAttribute::default();
    let applet_proxy = app_proxy_service.open_library_applet_proxy(
        sf::ProcessId::new(),
        sf::CopyHandle::from(svc::CURRENT_PROCESS_PSEUDO_HANDLE),
        sf::InMapAliasBuffer::from_other_var(&applet_attr),
    )?;

    let window_controller = applet_proxy.get_window_controller()?;

    let aruid = window_controller.get_applet_resource_user_id()?;

    *app_proxy_service_guard = Some(app_proxy_service);
    *LIBRARY_APPLET_PROXY.write() = Some(applet_proxy);
    *WINDOW_CONTROLLER.write() = Some(window_controller);
    GLOBAL_ARUID.store(aruid, core::sync::atomic::Ordering::Release);

    Ok(())
}

/// Returns whether the module has been successfully initialized.
pub fn is_initialized() -> bool {
    ALL_SYSTEM_APPLET_PROXY_SERVICE.read().is_some()
}

/// Finalizes library applet support, dropping the shared resources. pub(crate) as it should only run in rrt0.rs
pub(crate) fn finalize() {
    let mut app_proxy_service_guard = ALL_SYSTEM_APPLET_PROXY_SERVICE.write();

    *WINDOW_CONTROLLER.write() = None;
    *LIBRARY_APPLET_PROXY.write() = None;
    *app_proxy_service_guard = None;
}

pub fn get_window_controller<'a>() -> ReadGuard<'a, Option<WindowController>> {
    WINDOW_CONTROLLER.read()
}

pub fn get_library_applet_proxy<'a>() -> ReadGuard<'a, Option<LibraryAppletProxy>> {
    LIBRARY_APPLET_PROXY.read()
}

pub fn get_system_proxy_service<'a>() -> ReadGuard<'a, Option<AllSystemAppletProxiesService>> {
    ALL_SYSTEM_APPLET_PROXY_SERVICE.read()
}
