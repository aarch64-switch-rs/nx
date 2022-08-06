//! Userland library for Nintendo Switch homebrew (and other potential purposes), written in pure Rust and some assembly bits
//! 
//! # Contributing
//! 
//! You can always contribute to these libraries, report bugs, etc. at their [repository](https://github.com/aarch64-switch-rs/nx)
//! 
//! # Examples
//! 
//! Library examples are located at this other [repository](https://github.com/aarch64-switch-rs/examples)

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
#![feature(fn_traits)]
#![feature(negative_impls)]
#![feature(const_ptr_write)]
#![feature(stdsimd)]
#![macro_use]

use core::arch::global_asm;

// Required assembly bits (those which essentially cannot/shouldn't be inlined)

global_asm!(include_str!("asm.s"));
global_asm!(include_str!("rrt0.s"));
global_asm!(include_str!("mod0.s"));
global_asm!(include_str!("arm.s"));
global_asm!(include_str!("mem.s"));
global_asm!(include_str!("svc.s"));

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

#[cfg(feature = "smc")]
pub mod smc;

#[macro_use]
pub mod ipc;

#[cfg(feature = "services")]
pub mod service;

#[macro_use]
pub mod diag;

#[cfg(feature = "gpu")]
pub mod gpu;

#[cfg(feature = "input")]
pub mod input;

pub mod vmem;

pub mod arm;

pub mod wait;

#[cfg(feature = "fs")]
pub mod fs;

pub mod version;

#[cfg(feature = "rand")]
pub mod rand;

#[cfg(feature = "crypto")]
pub mod crypto;

#[cfg(feature = "la")]
pub mod la;
