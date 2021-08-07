#![no_std]
#![allow(incomplete_features)]
#![allow(non_snake_case)]
#![feature(llvm_asm)]
#![feature(global_asm)]
#![feature(alloc_error_handler)]
#![feature(const_generics)]
#![feature(const_raw_ptr_deref)]
#![feature(const_trait_impl)]
#![feature(specialization)]
#![feature(coerce_unsized)]
#![feature(linkage)]
#![feature(unsize)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(const_fn_transmute)]
#![feature(const_mut_refs)]
#![macro_use]

// Required assembly bits

global_asm!(include_str!("asm.s"));
global_asm!(include_str!("crt0.s"));
global_asm!(include_str!("arm.s"));
global_asm!(include_str!("mem.s"));
global_asm!(include_str!("svc.s"));

#[macro_use]
extern crate alloc;

#[macro_use]
extern crate derivative;

#[macro_use]
pub mod macros;

pub mod result;

pub mod results;

pub mod util;

pub mod mem;

pub mod dynamic;

pub mod sync;

pub mod thread;

pub mod hbl;

pub mod crt0;

pub mod svc;

pub mod smc;

pub mod ipc;

pub mod service;

pub mod diag;

pub mod gpu;

pub mod input;

pub mod vmem;

pub mod arm;

pub mod wait;

pub mod fs;

pub mod version;

pub use paste;

pub mod rand;
