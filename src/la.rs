use crate::arm;
use crate::result::*;
use crate::sync;
use crate::mem;
use crate::ipc::sf;
use crate::service::applet;
use crate::service::applet::ILibraryAppletCreator;
use crate::service::applet::ILibraryAppletAccessor;
use crate::service::applet::IStorage;
use crate::wait;
use crate::svc;
use core::mem as cmem;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct CommonArguments {
    pub version: u32,
    pub size: u32,
    pub la_api_version: u32,
    pub theme_color: u32,
    pub play_startup_sound: bool,
    pub pad: [u8; 7],
    pub system_tick: u64
}

pub struct LibraryAppletHolder {
    accessor: mem::Shared<dyn ILibraryAppletAccessor>,
    state_changed_event_handle: svc::Handle
}

impl LibraryAppletHolder {
    pub fn new(accessor: mem::Shared<dyn ILibraryAppletAccessor>) -> Result<Self> {
        let state_changed_event_h = accessor.get().get_applet_state_changed_event()?;

        Ok(Self {
            accessor,
            state_changed_event_handle: state_changed_event_h.handle
        })
    }

    #[inline]
    pub fn get_accessor(&self) -> mem::Shared<dyn ILibraryAppletAccessor> {
        self.accessor.clone()
    }

    #[inline]
    pub fn push_in_data_storage(&mut self, storage: mem::Shared<dyn IStorage>) -> Result<()> {
        self.accessor.get().push_in_data(storage)
    }
    
    pub fn push_in_data<T: Copy>(&mut self, t: T) -> Result<()> {
        let t_st = create_write_storage(t)?;
        self.push_in_data_storage(t_st)
    }

    #[inline]
    pub fn start(&mut self) -> Result<()> {
        self.accessor.get().start()
    }

    #[inline]
    pub fn join(&mut self) -> Result<()> {
        wait::wait_handles(&[self.state_changed_event_handle], -1)?;
        Ok(())
    }

    #[inline]
    pub fn pop_out_data_storage(&mut self) -> Result<mem::Shared<dyn IStorage>> {
        self.accessor.get().pop_out_data()
    }

    pub fn pop_out_data<O: Copy + Default>(&mut self) -> Result<O> {
        let o_st = self.pop_out_data_storage()?;
        read_storage(o_st)
    }
}

impl Drop for LibraryAppletHolder {
    fn drop(&mut self) {
        let _ = svc::close_handle(self.state_changed_event_handle);
    }
}

static mut G_CREATOR: sync::Locked<Option<mem::Shared<dyn ILibraryAppletCreator>>> = sync::Locked::new(false, None);

#[inline]
pub fn initialize(creator: mem::Shared<dyn ILibraryAppletCreator>) {
    unsafe {
        G_CREATOR.set(Some(creator));
    }
}

#[inline]
pub fn is_initialized() -> bool {
    unsafe {
        G_CREATOR.get().is_some()
    }
}

#[inline]
pub fn finalize() {
    unsafe {
        G_CREATOR.set(None);
    }
}

#[inline]
pub fn get_creator() -> Result<&'static mem::Shared<dyn ILibraryAppletCreator>> {
    unsafe {
        G_CREATOR.get().as_ref().ok_or(super::rc::ResultNotInitialized::make())
    }
}

pub fn read_storage<T: Copy + Default>(storage: mem::Shared<dyn IStorage>) -> Result<T> {
    let mut t: T = Default::default();

    let storage_accessor = storage.get().open()?;
    storage_accessor.get().read(0, sf::Buffer::from_other_mut_var(&mut t))?;

    Ok(t)
}

pub fn write_storage<T: Copy>(storage: mem::Shared<dyn IStorage>, t: T) -> Result<()> {
    result_return_unless!(is_initialized(), super::rc::ResultNotInitialized);

    let storage_accessor = storage.get().open()?;
    storage_accessor.get().write(0, sf::Buffer::from_other_var(&t))?;

    Ok(())
}

pub fn create_write_storage<T: Copy>(t: T) -> Result<mem::Shared<dyn IStorage>> {
    result_return_unless!(is_initialized(), super::rc::ResultNotInitialized);

    let storage = get_creator()?.get().create_storage(cmem::size_of::<T>())?;
    write_storage(storage.clone(), t)?;

    Ok(storage)
}

pub fn create_library_applet(id: applet::AppletId, mode: applet::LibraryAppletMode, mut common_args: CommonArguments) -> Result<LibraryAppletHolder> {
    result_return_unless!(is_initialized(), super::rc::ResultNotInitialized);

    let accessor = get_creator()?.get().create_library_applet(id, mode)?;

    let mut holder = LibraryAppletHolder::new(accessor)?;
    
    common_args.system_tick = arm::get_system_tick();
    holder.push_in_data(common_args)?;

    Ok(holder)
}

pub fn launch_wait_simple<I: Copy, O: Copy + Default>(id: applet::AppletId, common_args: CommonArguments, input: I) -> Result<O> {
    let mut holder = create_library_applet(id, applet::LibraryAppletMode::AllForeground, common_args)?;
    holder.push_in_data(input)?;
    holder.start()?;
    holder.join()?;
    holder.pop_out_data()
}

pub mod psel;