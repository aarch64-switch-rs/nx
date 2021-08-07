use crate::result::*;
use crate::results;
use crate::svc;
use crate::thread;
use core::ptr;
use core::mem;
use arrayvec::ArrayVec;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum HandleMode {
    Copy = 0,
    Move = 1
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum BufferFlags {
    Normal = 0,
    NonSecure = 1,
    Invalid = 2,
    NonDevice = 3
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct BufferDescriptor {
    pub size_low: u32,
    pub address_low: u32,
    pub bits: u32,
}

impl BufferDescriptor {
    pub const fn empty() -> Self {
        Self { size_low: 0, address_low: 0, bits: 0 }
    }

    pub fn new(buffer: *const u8, buffer_size: usize, flags: BufferFlags) -> Self {
        unsafe {
            let address_low = buffer as usize as u32;
            let address_mid = ((buffer as usize) >> 32) as u32;
            let address_high = ((buffer as usize) >> 36) as u32;
            let size_low = buffer_size as u32;
            let size_high = (buffer_size >> 32) as u32;

            let mut bits: u32 = 0;
            write_bits!(0, 1, bits, flags as u32);
            write_bits!(2, 23, bits, address_high);
            write_bits!(24, 27, bits, size_high);
            write_bits!(28, 31, bits, address_mid);

            Self { size_low: size_low, address_low: address_low, bits: bits }
        }
    }

    pub const fn get_address(&self) -> *mut u8 {
        let address_high = read_bits!(2, 23, self.bits);
        let address_mid = read_bits!(28, 31, self.bits);
        (self.address_low as usize | ((address_mid as usize) << 32) | ((address_high as usize) << 36)) as *mut u8
    }

    pub const fn get_size(&self) -> usize {
        let size_high = read_bits!(24, 27, self.bits);
        self.size_low as usize | ((size_high as usize) << 32)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct SendStaticDescriptor {
    bits: u32,
    address_low: u32,
}

impl SendStaticDescriptor {
    pub const fn empty() -> Self {
        Self { bits: 0, address_low: 0 }
    }

    pub fn new(buffer: *const u8, buffer_size: usize, index: u32) -> Self {
        unsafe {
            let address_low = buffer as usize as u32;
            let address_mid = ((buffer as usize) >> 32) as u32;
            let address_high = ((buffer as usize) >> 36) as u32;

            let mut bits: u32 = 0;
            write_bits!(0, 5, bits, index);
            write_bits!(6, 11, bits, address_high);
            write_bits!(12, 15, bits, address_mid);
            write_bits!(16, 31, bits, buffer_size as u32);

            Self { bits: bits, address_low: address_low }
        }
    }

    pub const fn get_address(&self) -> *mut u8 {
        let address_high = read_bits!(6, 11, self.bits);
        let address_mid = read_bits!(12, 15, self.bits);
        (self.address_low as usize | ((address_mid as usize) << 32) | ((address_high as usize) << 36)) as *mut u8
    }

    pub const fn get_size(&self) -> usize {
        read_bits!(16, 31, self.bits) as usize
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct ReceiveStaticDescriptor {
    address_low: u32,
    bits: u32,
}

impl ReceiveStaticDescriptor {
    pub const fn empty() -> Self {
        Self { address_low: 0, bits: 0 }
    }

    pub fn new(buffer: *const u8, buffer_size: usize) -> Self {
        unsafe {
            let address_low = buffer as usize as u32;
            let address_high = ((buffer as usize) >> 32) as u32;

            let mut bits: u32 = 0;
            write_bits!(0, 15, bits, address_high);
            write_bits!(16, 31, bits, buffer_size as u32);

            Self { address_low: address_low, bits: bits }
        }
    }

    pub const fn get_address(&self) -> *mut u8 {
        let address_high = read_bits!(0, 15, self.bits);
        (self.address_low as usize | ((address_high as usize) << 32)) as *mut u8
    }

    pub const fn get_size(&self) -> usize {
        read_bits!(16, 31, self.bits) as usize
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct CommandHeader {
    bits_1: u32,
    bits_2: u32,
}

impl CommandHeader {
    pub const fn empty() -> Self {
        Self { bits_1: 0, bits_2: 0 }
    }

    pub const fn encode_receive_static_type(receive_static_count: u32) -> u32 {
        let mut static_type: u32 = 0;
        if receive_static_count > 0 {
            static_type += 2;
            if receive_static_count != 0xFF {
                static_type += receive_static_count;
            }
        }
        static_type
    }

    pub const fn decode_receive_static_type(receive_static_type: u32) -> u32 {
        let mut count: u32 = 0;
        if receive_static_type > 0 {
            if receive_static_type == 2 {
                count = 0xFF;
            }
            else if receive_static_type > 2 {
                count = receive_static_type - 2;
            }
        }
        count
    }

    pub const fn new(command_type: u32, send_static_count: u32, send_buffer_count: u32, receive_buffer_count: u32, exchange_buffer_count: u32, data_word_count: u32, receive_static_count: u32, has_special_header: bool) -> Self {
        let mut bits_1: u32 = 0;
        write_bits!(0, 15, bits_1, command_type);
        write_bits!(16, 19, bits_1, send_static_count);
        write_bits!(20, 23, bits_1, send_buffer_count);
        write_bits!(24, 27, bits_1, receive_buffer_count);
        write_bits!(28, 31, bits_1, exchange_buffer_count);

        let mut bits_2: u32 = 0;
        write_bits!(0, 9, bits_2, data_word_count);
        write_bits!(10, 13, bits_2, Self::encode_receive_static_type(receive_static_count));
        write_bits!(31, 31, bits_2, has_special_header as u32);

        Self { bits_1: bits_1, bits_2: bits_2 }
    }

    pub const fn get_command_type(&self) -> u32 {
        let raw_type = read_bits!(0, 15, self.bits_1);
        unsafe {
            mem::transmute(raw_type)
        }
    }

    pub const fn get_send_static_count(&self) -> u32 {
        read_bits!(16, 19, self.bits_1)
    }

    pub const fn get_send_buffer_count(&self) -> u32 {
        read_bits!(20, 23, self.bits_1)
    }

    pub const fn get_receive_buffer_count(&self) -> u32 {
        read_bits!(24, 27, self.bits_1)
    }

    pub const fn get_exchange_buffer_count(&self) -> u32 {
        read_bits!(28, 31, self.bits_1)
    }

    pub const fn get_data_word_count(&self) -> u32 {
        read_bits!(0, 9, self.bits_2)
    }

    pub const fn get_receive_static_count(&self) -> u32 {
        Self::decode_receive_static_type(read_bits!(10, 13, self.bits_2))
    }

    pub const fn get_has_special_header(&self) -> bool {
        read_bits!(31, 31, self.bits_2) != 0
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct CommandSpecialHeader {
    bits: u32,
}

impl CommandSpecialHeader {
    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    pub const fn new(send_process_id: bool, copy_handle_count: u32, move_handle_count: u32) -> Self {
        let mut bits: u32 = 0;
        write_bits!(0, 0, bits, send_process_id as u32);
        write_bits!(1, 4, bits, copy_handle_count);
        write_bits!(5, 8, bits, move_handle_count);

        Self { bits: bits }
    }

    pub const fn get_send_process_id(&self) -> bool {
        read_bits!(0, 0, self.bits) != 0
    }

    pub const fn get_copy_handle_count(&self) -> u32 {
        read_bits!(1, 4, self.bits)
    }

    pub const fn get_move_handle_count(&self) -> u32 {
        read_bits!(5, 8, self.bits)
    }
}

pub const DATA_PADDING: u32 = 16;

bit_enum! {
    BufferAttribute (u8) {
        In = bit!(0),
        Out = bit!(1),
        MapAlias = bit!(2),
        Pointer = bit!(3),
        FixedSize = bit!(4),
        AutoSelect = bit!(5),
        MapTransferAllowsNonSecure = bit!(6),
        MapTransferAllowsNonDevice = bit!(7)
    }
}

const MAX_COUNT: usize = 8;

#[derive(Copy, Clone)]
pub struct DataWalker {
    ptr: *mut u8,
    cur_offset: isize
}

impl DataWalker {
    pub fn empty() -> Self {
        Self { ptr: ptr::null_mut(), cur_offset: 0 }
    }

    pub fn new(ptr: *mut u8) -> Self {
        Self { ptr: ptr, cur_offset: 0 }
    }

    pub fn advance<T>(&mut self) {
        let align_of_type = core::mem::align_of::<T>() as isize;
        self.cur_offset += align_of_type - 1;
        self.cur_offset -= self.cur_offset % align_of_type;
        self.cur_offset += core::mem::size_of::<T>() as isize;
    }

    pub fn advance_get<T>(&mut self) -> T {
        unsafe {
            let align_of_type = core::mem::align_of::<T>() as isize;
            self.cur_offset += align_of_type - 1;
            self.cur_offset -= self.cur_offset % align_of_type;
            let offset = self.cur_offset;
            self.cur_offset += core::mem::size_of::<T>() as isize;

            let data_ref = self.ptr.offset(offset) as *const T;
            data_ref.read_volatile()
        }
    }

    pub fn advance_set<T>(&mut self, t: T) {
        unsafe {
            let align_of_type = core::mem::align_of::<T>() as isize;
            self.cur_offset += align_of_type - 1;
            self.cur_offset -= self.cur_offset % align_of_type;
            let offset = self.cur_offset;
            self.cur_offset += core::mem::size_of::<T>() as isize;

            let data_ref = self.ptr.offset(offset) as *mut T;
            data_ref.write_volatile(t);
        }
    }

    pub fn reset(&mut self) {
        self.cur_offset = 0;
    }

    pub fn reset_with(&mut self, ptr: *mut u8) {
        self.reset();
        self.ptr = ptr;
    }

    pub fn get_offset(&self) -> isize {
        self.cur_offset
    }
}

#[inline(always)]
pub fn get_ipc_buffer() -> *mut u8 {
    unsafe {
        &mut (*thread::get_thread_local_storage()).ipc_buffer as *mut _ as *mut u8
    }
}

#[inline(always)]
pub fn read_array_from_buffer<T: Copy>(buffer: *mut u8, count: u32, array: &mut ArrayVec<[T; MAX_COUNT]>) -> *mut u8 {
    unsafe {
        let tmp_buffer = buffer as *mut T;
        array.clear();
        let _ = array.try_extend_from_slice(core::slice::from_raw_parts(tmp_buffer, count as usize));
        tmp_buffer.offset(count as isize) as *mut u8
    }
}

#[inline(always)]
pub fn write_array_to_buffer<T: Copy>(buffer: *mut u8, count: u32, array: &ArrayVec<[T; MAX_COUNT]>) -> *mut u8 {
    unsafe {
        let tmp_buffer = buffer as *mut T;
        core::ptr::copy(array.as_ptr(), tmp_buffer, count as usize);
        tmp_buffer.offset(count as isize) as *mut u8
    }
}

#[inline(always)]
pub fn get_aligned_data_offset(data_words_offset: *mut u8, base_offset: *mut u8) -> *mut u8 {
    unsafe {
        let align = DATA_PADDING as usize - 1;
        let data_offset = (data_words_offset as usize - base_offset as usize + align) & !align;
        (data_offset + base_offset as usize) as *mut u8
    }
}

pub mod cmif;

pub mod tipc;
