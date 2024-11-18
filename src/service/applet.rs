use core::sync::atomic::AtomicU64;

use crate::ipc::sf;
use crate::ipc::sf::sm;
use crate::mem;
use crate::service;
use crate::sync;
use crate::{result::*, svc};

pub use crate::ipc::sf::applet::*;


use generic_once_cell::OnceCell;


pub static ALL_SYSTEM_APPLET_PROXY_SERVICE: OnceCell<sync::sys::mutex::Mutex,mem::Shared<AllSystemAppletProxiesService>> = OnceCell::new();
pub static LIBRARY_APPLET_PROXY: OnceCell<sync::sys::mutex::Mutex,mem::Shared<LibraryAppletProxy>> = OnceCell::new();
pub static WINDOW_CONTROLLER: OnceCell<sync::sys::mutex::Mutex,mem::Shared<WindowController>> = OnceCell::new();
pub static GLOBAL_ARUID: AtomicU64 = AtomicU64::new(0);

#[inline]
pub fn initialize() -> Result<()>{
    let so = service::new_service_object::<AllSystemAppletProxiesService>();
    let _ = ALL_SYSTEM_APPLET_PROXY_SERVICE.set(mem::Shared::new(so?));

    let applet_attr = AppletAttribute::default();
    let applet_proxy  = ALL_SYSTEM_APPLET_PROXY_SERVICE.get().unwrap().lock().open_library_applet_proxy(
        sf::ProcessId::new(),
        sf::CopyHandle::from(svc::CURRENT_PROCESS_PSEUDO_HANDLE),
        sf::InMapAliasBuffer::from_other_var(&applet_attr),
    );
    let _ = LIBRARY_APPLET_PROXY.set(mem::Shared::new(applet_proxy?));

    let window_controller = LIBRARY_APPLET_PROXY.get().unwrap().lock().get_window_controller();
    let _ = WINDOW_CONTROLLER.set(mem::Shared::new(window_controller?));

    let aruid = WINDOW_CONTROLLER.get().unwrap().lock().get_applet_resource_user_id();
    GLOBAL_ARUID.store(aruid?, core::sync::atomic::Ordering::Release);

    Ok(())
}

/// Finalizes library applet support, dropping the shared resources. pub(crate) as it should only run in rrt0.rs
pub(crate) fn finalize() {

    WINDOW_CONTROLLER.get().map(|inner| inner.lock().session.close());
    LIBRARY_APPLET_PROXY.get().map(|inner| inner.lock().session.close());
    ALL_SYSTEM_APPLET_PROXY_SERVICE.get().map(|inner| inner.lock().session.close());
}

pub fn get_window_controller() -> mem::Shared<WindowController> {
    WINDOW_CONTROLLER.get().unwrap().clone()
}

pub fn get_library_applet_proxy() -> mem::Shared<LibraryAppletProxy> {
    LIBRARY_APPLET_PROXY.get().unwrap().clone()
}

pub fn get_system_proxy_service() -> mem::Shared<AllSystemAppletProxiesService> {
    ALL_SYSTEM_APPLET_PROXY_SERVICE.get().unwrap().clone()
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
