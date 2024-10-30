use crate::result::*;
use crate::ipc::sf;
use crate::ipc::sf::CmifPidPlaceholder;
use crate::version;

use super::FileSystem;

ipc_sf_define_default_interface_client!(FileSystemProxy);
ipc_sf_define_interface_trait! {
	trait FileSystemProxy {
        set_current_process [1, version::VersionInterval::all()]: (process_id: sf::ProcessId, _pid_placeholder: CmifPidPlaceholder) => ();
        open_sd_card_filesystem [18, version::VersionInterval::all()]: () => (sd_filesystem: FileSystem);
        output_access_log_to_sd_card [1006, version::VersionInterval::all()]: (log_buf: sf::InMapAliasBuffer<u8>) => ();
    }
}