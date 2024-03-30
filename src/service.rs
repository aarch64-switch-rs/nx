//! Base service/named port support and wrappers

use crate::ipc::client;
use crate::ipc::sf;
use crate::mem;
use crate::svc;
use crate::result::*;

pub mod sm;
use crate::service::sm::IUserInterface;

/// Represents a named port interface
/// 
/// Interfaces which wrap named ports (see [`manage_named_port`][`svc::manage_named_port`] or [`connect_to_named_port`][`svc::connect_to_named_port`]) must implement this trait
pub trait INamedPort: client::IClientObject {
    /// Gets the name to be used to connect to the named port (via [`connect_to_named_port`][`svc::connect_to_named_port`])
    fn get_name() -> &'static str;
    /// This will get executed after connecting to the named port in [`new_named_port_object`], allowing for extra initialization
    /// 
    /// Some interfaces may have initialization commands (check [SM's case][`sm::UserInterface::register_client`]) which can be automatically called this way
    fn post_initialize(&mut self) -> Result<()>;
}

/// Represents a service interface
/// 
/// Interfaces which wrap services (see [SM][`sm::UserInterface`]) must implement this trait
pub trait IService: client::IClientObject {
    /// Gets the service's name
    fn get_name() -> sm::ServiceName;
    /// Gets whether the service should be used as a domain
    /// 
    /// If this is [`true`], the service will be converted to a domain after being accessed (see [`convert_to_domain`][`sf::Session::convert_to_domain`]) in [`new_service_object`]
    fn as_domain() -> bool;
    /// This will get executed after accessing the service in [`new_service_object`], allowing for extra initialization
    /// 
    /// Some interfaces may have initialization commands (check [SM's case][`sm::UserInterface::register_client`]) which can be automatically called this way
    fn post_initialize(&mut self) -> Result<()>;
}

/// Wrapper for connecting to a named port and instantiating the wrapper interface over the specified named port
/// 
/// For more information about this, check [`INamedPort`]
pub fn new_named_port_object<T: INamedPort + 'static>() -> Result<mem::Shared<T>> {
    let handle = svc::connect_to_named_port(T::get_name().as_ptr())?;
    let mut object = T::new(sf::Session::from_handle(handle));
    object.post_initialize()?;
    Ok(mem::Shared::new(object))
}

/// Wrapper for accessing a service and instantiating the wrapper interface over the specified service
/// 
/// For more information about this, check [`IService`]
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

pub mod fsp;

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

pub mod nfp;