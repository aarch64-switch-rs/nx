use crate::ipc::client;
use crate::ipc::sf;
use crate::mem;
use crate::svc;
use crate::result::*;

pub mod sm;
use crate::service::sm::IUserInterface;

pub trait INamedPort: client::IClientObject {
    fn get_name() -> &'static str;
    fn post_initialize(&mut self) -> Result<()>;
}

pub trait IService: client::IClientObject {
    fn get_name() -> sm::ServiceName;
    fn as_domain() -> bool;
    fn post_initialize(&mut self) -> Result<()>;
}

pub fn new_named_port_object<T: INamedPort + 'static>() -> Result<mem::Shared<T>> {
    let handle = svc::connect_to_named_port(T::get_name().as_ptr())?;
    let mut object = T::new(sf::Session::from_handle(handle));
    object.post_initialize()?;
    Ok(mem::Shared::new(object))
}

pub fn new_service_object<T: IService + 'static>() -> Result<mem::Shared<T>> {
    let sm = new_named_port_object::<sm::UserInterface>()?;
    let session_handle = sm.get().get_service_handle(T::get_name())?;
    sm.get().detach_client(sf::ProcessId::new())?;
    let mut object = T::new(sf::Session::from_handle(session_handle.handle));
    if T::as_domain() {
        object.convert_to_domain()?;
    }
    object.post_initialize()?;
    Ok(mem::Shared::new(object))
}

pub mod psm;

pub mod fspsrv;

pub mod lm;

pub mod vi;

pub mod nv;

pub mod dispdrv;

pub mod fatal;

pub mod hid;

pub mod applet;

pub mod psc;

pub mod pm;

pub mod set;

pub mod mii;

pub mod spl;

pub mod usb;

pub mod ldr;