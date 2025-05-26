use super::ncm;
use crate::ipc::sf;
use crate::version;

ipc_sf_define_default_client_for_interface!(LocationResolver);
ipc_sf_define_interface_trait! {
    trait LocationResolver {
        redirect_program_path [1, version::VersionInterval::all()]: (program_id: ncm::ProgramId, path_buf: sf::InPointerBuffer<u8>) => ();
    }
}

ipc_sf_define_default_client_for_interface!(RegisteredLocationResolver);
ipc_sf_define_interface_trait! {
    trait RegisteredLocationResolver {
        register_program_path_deprecated [1, version::VersionInterval::to(version::Version::new(8, 1, 1))]: (program_id: ncm::ProgramId, path_buf: sf::InPointerBuffer<u8>) => ();
        register_program_path [1, version::VersionInterval::from(version::Version::new(9, 0, 0))]: (program_id: ncm::ProgramId, owner_id: ncm::ProgramId, path_buf: sf::InPointerBuffer<u8>) => ();
        redirect_program_path_deprecated [3, version::VersionInterval::to(version::Version::new(8, 1, 1))]: (program_id: ncm::ProgramId, path_buf: sf::InPointerBuffer<u8>) => ();
        redirect_program_path [3, version::VersionInterval::from(version::Version::new(9, 0, 0))]: (program_id: ncm::ProgramId, owner_id: ncm::ProgramId, path_buf: sf::InPointerBuffer<u8>) => ();
    }
}

ipc_sf_define_default_client_for_interface!(LocationResolverManager);
ipc_sf_define_interface_trait! {
    trait LocationResolverManager {
        open_location_resolver [0, version::VersionInterval::all()]: (storage_id: ncm::StorageId) => (resolver: LocationResolver | session_type!(LocationResolver) );
        open_registered_location_resolver [1, version::VersionInterval::all()]: () => (resolver: RegisteredLocationResolver | session_type!(RegisteredLocationResolver) );
    }
}
