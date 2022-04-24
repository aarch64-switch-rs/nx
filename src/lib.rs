#![no_std]
#![allow(incomplete_features)]
#![allow(non_snake_case)]
#![feature(alloc_error_handler)]
#![feature(adt_const_params)]
#![feature(generic_const_exprs)]
#![feature(const_trait_impl)]
#![feature(specialization)]
#![feature(coerce_unsized)]
#![feature(linkage)]
#![feature(unsize)]
#![feature(const_mut_refs)]
#![feature(derive_default_enum)]
#![feature(fn_traits)]
#![feature(untagged_unions)]
#![feature(negative_impls)]
#![feature(const_intrinsic_copy)]
#![feature(const_ptr_write)]
#![macro_use]

#[macro_use]
extern crate alloc;

#[macro_use]
extern crate static_assertions;

#[macro_use]
pub mod macros;

#[macro_use]
pub mod result;

pub mod rc;

#[macro_use]
pub mod util;

pub mod mem;

pub mod elf;

pub mod sync;

pub mod thread;

pub mod hbl;

#[macro_use]
pub mod rrt0;

pub mod svc;

pub mod smc;

#[macro_use]
pub mod ipc;

pub mod service;

#[macro_use]
pub mod diag;

pub mod gpu;

pub mod input;

pub mod vmem;

pub mod arm;

pub mod wait;

pub mod fs;

pub mod version;

pub mod rand;

#[cfg(target_pointer_width = "64")]
pub mod crypto;

pub use paste;