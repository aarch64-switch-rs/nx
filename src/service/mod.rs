use crate::ipc::sf;
use crate::mem;
use crate::svc;
use crate::result::*;

pub mod sm;
use crate::service::sm::IUserInterface;

pub trait IClientObject: sf::IObject {
    fn new(session: sf::Session) -> Self where Self: Sized;
}

pub trait INamedPort: IClientObject {
    fn get_name() -> &'static str;
    fn post_initialize(&mut self) -> Result<()>;
}

pub trait IService: IClientObject {
    fn get_name() -> &'static str;
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
    let session_handle = sm.get().get_service_handle(sm::ServiceName::new(T::get_name()))?;
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