use crate::result::*;
use crate::results;
use core::mem;
use core::ptr;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ParcelHeader {
    pub payload_size: u32,
    pub payload_offset: u32,
    pub objects_size: u32,
    pub objects_offset: u32
}

impl ParcelHeader {
    pub const fn new() -> Self {
        Self { payload_size: 0, payload_offset: 0, objects_size: 0, objects_offset: 0 }
    }
}

const PAYLOAD_SIZE: usize = 0x200;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ParcelPayload {
    pub header: ParcelHeader,
    pub payload: [u8; PAYLOAD_SIZE]
}

impl ParcelPayload {
    pub const fn new() -> Self {
        Self { header: ParcelHeader::new(), payload: [0; PAYLOAD_SIZE] }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ParcelData {
    pub parcel_type: u32,
    pub unk: u32,
    pub handle: i32,
    pub zero: [u8; 0xC],
    pub dispdrv: [u8; 8],
    pub zero_2: [u8; 8],
}

pub struct Parcel {
    payload: ParcelPayload,
    read_offset: usize,
    write_offset: usize
}

impl Parcel {
    pub const fn new() -> Self {
        Self { payload: ParcelPayload::new(), read_offset: 0, write_offset: 0 }
    }

    pub fn read_raw_unaligned(&mut self, out_data: *mut u8, data_size: usize) -> Result<()> {
        result_return_if!((self.read_offset + data_size) > PAYLOAD_SIZE, results::lib::gpu::ResultParcelNotEnoughReadSpace);

        unsafe {
            ptr::copy((&mut self.payload.payload as *mut _ as *mut u8).offset(self.read_offset as isize), out_data, data_size);
        }
        self.read_offset += data_size;
        Ok(())
    }

    pub fn read_raw(&mut self, out_data: *mut u8, data_size: usize) -> Result<()> {
        self.read_raw_unaligned(out_data, (data_size + 3) & !3)
    }

    pub fn write_raw_unaligned(&mut self, data: *const u8, data_size: usize) -> Result<()> {
        result_return_if!((self.write_offset + data_size) > PAYLOAD_SIZE, results::lib::gpu::ResultParcelNotEnoughWriteSpace);

        unsafe {
            ptr::copy(data, (&mut self.payload.payload as *mut _ as *mut u8).offset(self.write_offset as isize), data_size);
        }
        self.write_offset += data_size;
        Ok(())
    }

    pub fn write_reserve_raw(&mut self, data_size: usize) -> Result<*mut u8> {
        let actual_size = (data_size + 3) & !3;
        result_return_if!((self.write_offset + actual_size) > PAYLOAD_SIZE, results::lib::gpu::ResultParcelNotEnoughWriteSpace);

        let buf = unsafe { (&mut self.payload.payload as *mut _ as *mut u8).offset(self.write_offset as isize) };
        self.write_offset += actual_size;
        Ok(buf)
    }

    pub fn write_raw(&mut self, data: *const u8, data_size: usize) -> Result<()> {
        self.write_raw_unaligned(data, (data_size + 3) & !3)
    }

    pub fn write_unaligned<T>(&mut self, t: T) -> Result<()> {
        self.write_raw_unaligned(&t as *const T as *const u8, mem::size_of::<T>())
    }

    pub fn write<T>(&mut self, t: T) -> Result<()> {
        self.write_raw(&t as *const T as *const u8, mem::size_of::<T>())
    }

    pub fn read_unaligned<T>(&mut self) -> Result<T> {
        let mut t: T = unsafe {
            mem::zeroed()
        };
        self.read_raw_unaligned(&mut t as *mut T as *mut u8, mem::size_of::<T>())?;
        Ok(t)
    }

    pub fn read<T>(&mut self) -> Result<T> {
        let mut t: T = unsafe {
            mem::zeroed()
        };
        self.read_raw(&mut t as *mut T as *mut u8, mem::size_of::<T>())?;
        Ok(t)
    }

    pub fn write_str(&mut self, string: &str) -> Result<()> {
        let len = string.len();
        self.write(len as u32)?;
        let str_bytes = string.as_bytes();
        let str_write_buf = self.write_reserve_raw((len + 1) * 2)? as *mut u16;

        for i in 0..len {
            unsafe {
                let cur = str_write_buf.offset(i as isize);
                *cur = str_bytes[i] as u16;
            }
        }
        Ok(())
    }

    pub fn write_interface_token(&mut self, token: &str) -> Result<()> {
        let value: u32 = 0x100;
        self.write(value)?;
        self.write_str(token)
    }

    pub fn read_sized_raw(&mut self, out_data: *mut u8) -> Result<usize> {
        let len = self.read::<i32>()? as usize;
        let fd_count = self.read::<i32>()?;
        result_return_unless!(fd_count == 0, results::lib::gpu::ResultParcelFdsNotSupported);

        self.read_raw(out_data, len)?;
        Ok(len)
    }

    pub fn read_sized<T>(&mut self) -> Result<T> {
        let mut t: T = unsafe {
            mem::zeroed()
        };
        let len = self.read_sized_raw(&mut t as *mut T as *mut u8)?;
        result_return_unless!(len == mem::size_of::<T>(), results::lib::gpu::ResultParcelReadSizeMismatch);
        Ok(t)
    }

    pub fn write_sized_raw(&mut self, data: *const u8, data_size: usize) -> Result<()> {
        let len = data_size as i32;
        self.write(len)?;
        let fd_count: i32 = 0;
        self.write(fd_count)?;

        self.write_raw(data, data_size)?;
        Ok(())
    }

    pub fn write_sized<T>(&mut self, t: T) -> Result<()> {
        self.write_sized_raw(&t as *const T as *const u8, mem::size_of::<T>())
    }

    pub fn load_from(&mut self, payload: ParcelPayload) {
        self.payload = payload;
        self.read_offset = 0;
        self.write_offset = payload.header.payload_size as usize;
    }

    pub fn end_write(&mut self) -> Result<(ParcelPayload, usize)> {
        self.payload.header.payload_size = self.write_offset as u32;
        self.payload.header.payload_offset = mem::size_of::<ParcelHeader>() as u32;
        let payload_len = self.payload.header.payload_offset + self.payload.header.payload_size;
        self.payload.header.objects_offset = payload_len;
        self.payload.header.objects_size = 0;
        Ok((self.payload, payload_len as usize))
    }
}