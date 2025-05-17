use crate::ipc::sf::sm;
use crate::result::*;
use crate::service;

use crate::sync::Mutex;

pub use crate::ipc::sf::mii::*;

#[inline]
pub fn get_device_id() -> Result<CreateId> {
    use service::set::{ISystemSettingsClient, SystemSettingsService};

    service::new_service_object::<SystemSettingsService>()?.get_mii_author_id()
}

impl service::IService for StaticService {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("mii:e")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}

static G_STATIC_SRV: Mutex<Option<StaticService>> = Mutex::new(None);
static G_DB_SRV: Mutex<Option<DatabaseService>> = Mutex::new(None);

pub fn initialize() -> Result<()> {
    let static_service = service::new_service_object::<StaticService>()?;
    let db_service = static_service.get_database_service(SpecialKeyCode::Normal)?;
    *G_STATIC_SRV.lock() = Some(static_service);
    *G_DB_SRV.lock() = Some(db_service);
    Ok(())
}

pub(crate) fn finalize() {
    *G_DB_SRV.lock() = None;
    *G_STATIC_SRV.lock() = None;
}
