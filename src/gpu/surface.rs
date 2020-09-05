use super::*;
use crate::results;
use crate::gpu::binder;
use crate::gpu::ioctl;
use crate::svc;
use crate::ipc::sf;
use crate::service::nv;
use crate::service::vi;
use crate::service::dispdrv;
use crate::mem;
use core::mem as cmem;
use core::ptr;

extern crate alloc;

const MAX_BUFFERS: usize = 8;

pub type LayerDestroyFn = fn(vi::LayerId, mem::Shared<vi::ApplicationDisplayService>) -> Result<()>;

pub struct Surface<NS: nv::INvDrvService + 'static> {
    binder: binder::Binder,
    nvdrv_srv: mem::Shared<NS>,
    application_display_service: mem::Shared<vi::ApplicationDisplayService>,
    width: u32,
    height: u32,
    buffer_data: *mut u8,
    buffer_alloc_layout: alloc::alloc::Layout,
    single_buffer_size: usize,
    buffer_count: u32,
    slot_has_requested: [bool; MAX_BUFFERS],
    graphic_buf: GraphicBuffer,
    color_fmt: ColorFormat,
    pixel_fmt: PixelFormat,
    layout: Layout,
    display_id: vi::DisplayId,
    layer_id: vi::LayerId,
    layer_destroy_fn: LayerDestroyFn,
    nvhost_fd: nv::Fd,
    nvmap_fd: nv::Fd,
    nvhostctrl_fd: nv::Fd,
    vsync_event_handle: svc::Handle,
    buffer_event_handle: svc::Handle
}

impl<NS: nv::INvDrvService> Surface<NS> {
    pub fn new(binder_handle: i32, nvdrv_srv: mem::Shared<NS>, application_display_service: mem::Shared<vi::ApplicationDisplayService>, nvhost_fd: u32, nvmap_fd: u32, nvhostctrl_fd: u32, hos_binder_driver: mem::Shared<dispdrv::HOSBinderDriver>, buffer_count: u32, display_id: vi::DisplayId, layer_id: vi::LayerId, width: u32, height: u32, color_fmt: ColorFormat, pixel_fmt: PixelFormat, layout: Layout, layer_destroy_fn: LayerDestroyFn) -> Result<Self> {
        let mut binder = binder::Binder::new(binder_handle, hos_binder_driver)?;
        binder.increase_refcounts()?;
        let _ = binder.connect(ConnectionApi::Cpu, false)?;
        let vsync_event_handle = application_display_service.get().get_display_vsync_event(display_id)?;
        let buffer_event_handle = binder.get_native_handle(dispdrv::NativeHandleType::BufferEvent)?;
        let mut surface = Self { binder: binder, nvdrv_srv: nvdrv_srv, application_display_service: application_display_service, width: width, height: height, buffer_data: ptr::null_mut(), buffer_alloc_layout: alloc::alloc::Layout::new::<u8>(), single_buffer_size: 0, buffer_count: buffer_count, slot_has_requested: [false; MAX_BUFFERS], graphic_buf: unsafe { cmem::zeroed() }, color_fmt: color_fmt, pixel_fmt: pixel_fmt, layout: layout, display_id: display_id, layer_id: layer_id, layer_destroy_fn: layer_destroy_fn, nvhost_fd: nvhost_fd, nvmap_fd: nvmap_fd, nvhostctrl_fd: nvhostctrl_fd, vsync_event_handle: vsync_event_handle.handle, buffer_event_handle: buffer_event_handle.handle };
        surface.initialize()?;
        Ok(surface)
    }

    fn do_ioctl<I: ioctl::Ioctl>(&mut self, i: &mut I) -> Result<()> {
        let fd = match I::get_fd() {
            ioctl::IoctlFd::NvHost => self.nvhost_fd,
            ioctl::IoctlFd::NvMap => self.nvmap_fd,
            ioctl::IoctlFd::NvHostCtrl => self.nvhostctrl_fd,
        };

        let err = self.nvdrv_srv.get().ioctl(fd, I::get_id(), sf::Buffer::from_var(i), sf::Buffer::from_var(i))?;
        nv::convert_error_code(err)
    }

    fn initialize(&mut self) -> Result<()> {
        let kind = Kind::Generic_16BX2;
        let scan_fmt = DisplayScanFormat::Progressive;
        let pid: u32 = 42;
        let bpp = calculate_bpp(self.color_fmt);
        let aligned_width = align_width(bpp, self.width);
        let aligned_width_bytes = aligned_width * bpp;
        let aligned_height = align_height(self.height);
        let stride = aligned_width;
        self.single_buffer_size = (aligned_width_bytes * aligned_height) as usize;
        let usage = GraphicsAllocatorUsage::HardwareComposer() | GraphicsAllocatorUsage::HardwareRender() | GraphicsAllocatorUsage::HardwareTexture();
        let buf_size = self.buffer_count as usize * self.single_buffer_size;
        self.buffer_alloc_layout = unsafe { alloc::alloc::Layout::from_size_align_unchecked(buf_size, mem::PAGE_ALIGNMENT) };

        let mut ioctl_create: ioctl::NvMapCreate = unsafe { cmem::zeroed() };
        ioctl_create.size = buf_size as u32;
        self.do_ioctl(&mut ioctl_create)?;

        let mut ioctl_getid: ioctl::NvMapGetId = unsafe { cmem::zeroed() };
        ioctl_getid.handle = ioctl_create.handle;
        self.do_ioctl(&mut ioctl_getid)?;

        self.buffer_data = unsafe { alloc::alloc::alloc(self.buffer_alloc_layout) };
        svc::set_memory_attribute(self.buffer_data, buf_size, 8, svc::MemoryAttribute::Uncached())?;

        let mut ioctl_alloc: ioctl::NvMapAlloc = unsafe { cmem::zeroed() };
        ioctl_alloc.handle = ioctl_create.handle;
        ioctl_alloc.heap_mask = 0;
        ioctl_alloc.flags = ioctl::AllocFlags::ReadOnly;
        ioctl_alloc.align = mem::PAGE_ALIGNMENT as u32;
        ioctl_alloc.kind = Kind::Pitch;
        ioctl_alloc.address = self.buffer_data;
        self.do_ioctl(&mut ioctl_alloc)?;

        self.graphic_buf.header.magic = GRAPHIC_BUFFER_HEADER_MAGIC;
        self.graphic_buf.header.width = self.width;
        self.graphic_buf.header.height = self.height;
        self.graphic_buf.header.stride = stride;
        self.graphic_buf.header.pixel_format = self.pixel_fmt;
        self.graphic_buf.header.gfx_alloc_usage = usage;
        self.graphic_buf.header.pid = pid;
        self.graphic_buf.header.buffer_size = ((cmem::size_of::<GraphicBuffer>() - cmem::size_of::<GraphicBufferHeader>()) / cmem::size_of::<u32>()) as u32;
        self.graphic_buf.map_id = ioctl_getid.id;
        self.graphic_buf.magic = GRAPHIC_BUFFER_MAGIC;
        self.graphic_buf.pid = pid;
        self.graphic_buf.gfx_alloc_usage = usage;
        self.graphic_buf.pixel_format = self.pixel_fmt;
        self.graphic_buf.external_pixel_format = self.pixel_fmt;
        self.graphic_buf.stride = stride;
        self.graphic_buf.full_size = self.single_buffer_size as u32;
        self.graphic_buf.plane_count = 1;
        self.graphic_buf.planes[0].width = self.width;
        self.graphic_buf.planes[0].height = self.height;
        self.graphic_buf.planes[0].color_format = self.color_fmt;
        self.graphic_buf.planes[0].layout = self.layout;
        self.graphic_buf.planes[0].pitch = aligned_width_bytes;
        self.graphic_buf.planes[0].map_handle = ioctl_create.handle;
        self.graphic_buf.planes[0].kind = kind;
        self.graphic_buf.planes[0].block_height_log2 = BLOCK_HEIGHT_LOG2;
        self.graphic_buf.planes[0].display_scan_format = scan_fmt;
        self.graphic_buf.planes[0].size = self.single_buffer_size;

        for i in 0..self.buffer_count {
            let mut graphic_buf_copy = self.graphic_buf;
            graphic_buf_copy.planes[0].offset = i * self.single_buffer_size as u32;
            self.binder.set_preallocated_buffer(i as i32, graphic_buf_copy)?;
        }

        Ok(())
    }

    fn finalize(&mut self) -> Result<()> {
        self.binder.disconnect(ConnectionApi::Cpu, DisconnectMode::AllLocal)?;
        self.binder.decrease_refcounts()?;

        let buf_size = self.buffer_count as usize * self.single_buffer_size;
        svc::set_memory_attribute(self.buffer_data, buf_size, 0, svc::MemoryAttribute::None())?;
        
        unsafe { alloc::alloc::dealloc(self.buffer_data, self.buffer_alloc_layout); }
        (self.layer_destroy_fn)(self.layer_id, self.application_display_service.clone())?;

        self.application_display_service.get().close_display(self.display_id)?;

        svc::close_handle(self.buffer_event_handle)?;
        svc::close_handle(self.vsync_event_handle)
    }

    pub fn dequeue_buffer(&mut self, is_async: bool) -> Result<(*mut u8, usize, i32, bool, MultiFence)> {
        let slot: i32;
        let has_fences: bool;
        let fences: MultiFence;
        if is_async {
            self.wait_buffer_event(-1)?;
            loop {
                match self.binder.dequeue_buffer(true, self.width, self.height, false, self.graphic_buf.gfx_alloc_usage) {
                    Ok((_slot, _has_fences, _fences)) => {
                        slot = _slot;
                        has_fences = _has_fences;
                        fences = _fences;
                        break;
                    },
                    Err(rc) => {
                        if results::lib::gpu::ResultBinderErrorCodeWouldBlock::matches(rc) {
                            continue;
                        }
                        return Err(rc);
                    },
                };
            }
        }
        else {
            let (_slot, _has_fences, _fences) = self.binder.dequeue_buffer(false, self.width, self.height, false, self.graphic_buf.gfx_alloc_usage)?;
            slot = _slot;
            has_fences = _has_fences;
            fences = _fences;
        }
        
        if !self.slot_has_requested[slot as usize] {
            self.binder.request_buffer(slot)?;
            self.slot_has_requested[slot as usize] = true;
        }

        let buf = unsafe { self.buffer_data.offset((slot as usize * self.single_buffer_size) as isize) };
        Ok((buf, self.single_buffer_size, slot, has_fences, fences))
    }

    pub fn queue_buffer(&mut self, slot: i32, fences: MultiFence) -> Result<()> {
        let mut qbi: QueueBufferInput = unsafe { cmem::zeroed() };
        qbi.swap_interval = 1;
        qbi.fences = fences;

        mem::flush_data_cache(self.buffer_data, self.single_buffer_size * self.buffer_count as usize);

        self.binder.queue_buffer(slot, qbi)?;
        Ok(())
    }

    pub fn wait_fences(&mut self, fences: MultiFence, timeout: i32) -> Result<()> {
        for i in 0..fences.fence_count {
            let mut ioctl_syncptwait: ioctl::NvHostCtrlSyncptWait = unsafe { cmem::zeroed() };
            ioctl_syncptwait.fence = fences.fences[i as usize];
            ioctl_syncptwait.timeout = timeout;

            if let Err(_) = self.do_ioctl(&mut ioctl_syncptwait) {
                // Don't error, but stop waiting for fences
                break;
            }
        }
        Ok(())
    }

    pub fn set_visible(&mut self, visible: bool) -> Result<()> {
        let system_display_service = self.application_display_service.get().get_system_display_service()?.to::<vi::SystemDisplayService>();
        system_display_service.get().set_layer_visibility(visible, self.layer_id)
    }

    pub fn wait_buffer_event(&mut self, timeout: i64) -> Result<()> {
        svc::wait_synchronization(&self.buffer_event_handle, 1, timeout)?;
        svc::reset_signal(self.buffer_event_handle)
    }

    pub fn wait_vsync_event(&mut self, timeout: i64) -> Result<()> {
        svc::wait_synchronization(&self.vsync_event_handle, 1, timeout)?;
        svc::reset_signal(self.vsync_event_handle)
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }

    pub fn get_color_format(&self) -> ColorFormat {
        self.color_fmt
    }

    pub fn compute_stride(&self) -> u32 {
        let bpp = calculate_bpp(self.color_fmt);
        align_width(bpp, self.width) * bpp
    }
}

impl<NS: nv::INvDrvService> Drop for Surface<NS> {
    fn drop(&mut self) {
        let _ = self.finalize();
    }
}