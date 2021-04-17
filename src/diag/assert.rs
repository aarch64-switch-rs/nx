use crate::result::*;
use crate::svc;
use crate::crt0;
use crate::ipc::cmif::sf;
use crate::service;
use crate::service::cmif::fatal;
use crate::service::cmif::fatal::IService;
use core::mem;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum AssertMode {
    ProcessExit,
    FatalThrow,
    SvcBreak,
    Panic
}

pub fn assert(mode: AssertMode, rc: ResultCode) {
    if rc.is_failure() {
        match mode {
            AssertMode::ProcessExit => {
                crt0::exit(rc);
            },
            AssertMode::FatalThrow => {
                match service::cmif::new_service_object::<fatal::Service>() {
                    Ok(fatal) => {
                        let _ = fatal.get().throw_with_policy(rc, fatal::Policy::ErrorScreen, sf::ProcessId::new());
                    },
                    _ => {}
                };
            },
            AssertMode::SvcBreak => {
                svc::break_(svc::BreakReason::Panic, &rc as *const _ as *const u8, mem::size_of::<ResultCode>());
            },
            AssertMode::Panic => {
                let res: Result<()> = Err(rc);
                res.unwrap();
            },
        }
    }
}