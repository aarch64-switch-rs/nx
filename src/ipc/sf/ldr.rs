use crate::result::*;
use crate::ipc::sf;
use crate::version;

use super::ncm;

ipc_sf_define_default_interface_client!(ShellInterface);
ipc_sf_define_interface_trait! {
	trait ShellInterface {
        set_program_argument_deprecated [0, version::VersionInterval::to(version::Version::new(10,2,0))]: (program_id: ncm::ProgramId, args_size: u32, args_buf: sf::InPointerBuffer<u8>) =>  () ();
        set_program_argument [0, version::VersionInterval::from(version::Version::new(11,0,0))]: (program_id: ncm::ProgramId, args_buf: sf::InPointerBuffer<u8>) =>  () ();
        flush_arguments [1, version::VersionInterval::all()]: () => () ();
        atmosphere_register_external_code [65000, version::VersionInterval::all()]: (program_id: ncm::ProgramId) =>  (session_handle: sf::MoveHandle) (session_handle: sf::MoveHandle);
        atmosphere_unregister_external_code [65001, version::VersionInterval::all()]: (program_id: ncm::ProgramId) =>  () ();
    }
}