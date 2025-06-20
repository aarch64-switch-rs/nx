use core::alloc::Layout;

use alloc::alloc::Allocator;
use alloc::alloc::Global;
use alloc::boxed::Box;

use crate::ipc::sf;
use crate::ipc::sf::CopyHandle;
use crate::mem::alloc::Buffer;
use crate::mem::alloc::PAGE_ALIGNMENT;
use crate::mem::wait_for_permission;
use crate::result::Result;

use crate::service::socket::*;

use crate::service::new_service_object;
use crate::svc::Handle;
use crate::svc::MemoryPermission;
use crate::sync::{ReadGuard, RwLock, WriteGuard};

pub struct SocketService {
    _tmem_buffer: Buffer<u8>,
    tmem_handle: Handle,
    service: Box<dyn ISocketClient + Send + 'static>,
    _monitor_service: Box<dyn ISocketClient + Send + 'static>,
    _bsd_client_pid: u64
}

unsafe impl Sync for SocketService {}
unsafe impl Send for SocketService {}

macro_rules! dispatch_bsd {
    ($fn_name:ident(&self, $($name:ident: $t:ty),*) -> $r:ty) => {
        pub fn $fn_name(&self, $($name: $t),*) -> Result<$r> {
            self.service.$fn_name( $($name),*)
        }
    };
    ($fn_name:ident(&mut self, $($name:ident: $t:ty),*) -> $r:ty) => {
        #[inline(always)]
        pub fn $fn_name(&mut self, $($name: $t),*) -> Result<$r> {
            self.service.$fn_name( $($name),*)
        }
    };
}

impl SocketService {
    fn new(
        config: BsdServiceConfig,
        kind: BsdSrvkind,
        transfer_mem_buffer: Option<Buffer<u8>>,
    ) -> Result<Self> {
        let mut service = match kind {
            BsdSrvkind::Applet => Box::new(new_service_object::<AppletSocketService>()?)
                as Box<dyn ISocketClient + Send + Sync + 'static>,
            BsdSrvkind::System => Box::new(new_service_object::<SystemSocketService>()?)
                as Box<dyn ISocketClient + Send + Sync + 'static>,
            BsdSrvkind::User => Box::new(new_service_object::<UserSocketService>()?)
                as Box<dyn ISocketClient + Send + Sync + 'static>,
        };

        let mut monitor_service = match kind {
            BsdSrvkind::Applet => Box::new(new_service_object::<AppletSocketService>()?)
                as Box<dyn ISocketClient + Send + Sync + 'static>,
            BsdSrvkind::System => Box::new(new_service_object::<SystemSocketService>()?)
                as Box<dyn ISocketClient + Send + Sync + 'static>,
            BsdSrvkind::User => Box::new(new_service_object::<UserSocketService>()?)
                as Box<dyn ISocketClient + Send + Sync + 'static>,
        };

        let tmem_min_size = config.min_transfer_mem_size();

        let tmem_buffer: Buffer<u8> = if let Some(prepared_buffer) = transfer_mem_buffer
            && prepared_buffer.layout.size() >= tmem_min_size
        {
            prepared_buffer
        } else {
            let layout =
                unsafe { Layout::from_size_align_unchecked(tmem_min_size, PAGE_ALIGNMENT) };
            Buffer {
                ptr: Global.allocate(layout).unwrap().as_ptr().cast(),
                layout,
                allocator: Global,
            }
        };

        let tmem_handle = crate::svc::create_transfer_memory(
            tmem_buffer.ptr.cast(),
            tmem_buffer.layout.size(),
            MemoryPermission::None(),
        )?;

        let bsd_client_pid = service.register_client(config, sf::ProcessId::new(), tmem_buffer.layout.size(), CopyHandle::from(tmem_handle))?;

        monitor_service.start_monitoring(sf::ProcessId::from(bsd_client_pid))?;

        Ok(Self {
            _tmem_buffer: tmem_buffer,
            tmem_handle,
            service,
            _monitor_service: monitor_service,
            _bsd_client_pid: bsd_client_pid
        })
    }
/*
    // Implement the IPC bsd API for now.
    dispatch_bsd!(register_client(
        &mut self,
        library_config: BsdServiceConfig,
        pid: ProcessId,
        transfer_mem_size: usize,
        tmem_handle: CopyHandle
    ) -> u64);
    dispatch_bsd!(start_monitoring(&mut self, pid: ProcessId) -> ());
    dispatch_bsd!(socket(&self, domain: SocketDomain, sock_type: SocketType, protocol: IpProto) -> BsdResult<()>);
    dispatch_bsd!(socket_exempt(
        &self,
        domain: SocketDomain,
        sock_type: SocketType,
        protocol: IpProto
    ) -> BsdResult<()>);

    dispatch_bsd!(open(&self, path_cstr: InAutoSelectBuffer<u8>, flags: i32) -> BsdResult<()>);

    dispatch_bsd!(select(
        &self,
        fd_cound: u32,
        read_fds: InOutAutoSelectBuffer<FdSet>,
        write_fds: InOutAutoSelectBuffer<FdSet>,
        except_fds: InOutAutoSelectBuffer<FdSet>,
        timeout: BsdTimeout
    ) -> BsdResult<()>);

    dispatch_bsd!(poll(&self, fds: InOutAutoSelectBuffer<PollFd>, timeout: i32) -> BsdResult<()>);

    dispatch_bsd!(sysctl(
        &self,
        name: InAutoSelectBuffer<u32>,
        newp: InAutoSelectBuffer<u8>,
        oldp: OutAutoSelectBuffer<u8>
    ) -> BsdResult<u32>);

    dispatch_bsd!(recv(
        &self,
        sockfd: i32,
        flags: ReadFlags,
        out_buffer: OutAutoSelectBuffer<u8>
    ) -> BsdResult<()>);

    dispatch_bsd!(recv_from(
        &self,
        sockfd: i32,
        flags: ReadFlags,
        out_buffer: OutAutoSelectBuffer<u8>,
        from_addrs: OutAutoSelectBuffer<SocketAddrRepr>
    ) -> BsdResult<u32>);

    dispatch_bsd!(send(&self, sockfd: i32, flags: SendFlags, buffer: InAutoSelectBuffer<u8>) -> BsdResult<()>);

    dispatch_bsd!(send_to(
        &self,
        sockfd: i32,
        flags: SendFlags,
        buffer: InAutoSelectBuffer<u8>,
        to_addrs: InAutoSelectBuffer<SocketAddrRepr>
    ) -> BsdResult<()>);

    dispatch_bsd!(accept(&self, sockfd: i32, addrs: OutAutoSelectBuffer<SocketAddrRepr>) -> BsdResult<u32>);

    dispatch_bsd!(bind(&self, sockfd: i32, addrs: InAutoSelectBuffer<SocketAddrRepr>) -> BsdResult<()>);

    dispatch_bsd!(connect(&self, sockfd: i32, addrs: InAutoSelectBuffer<SocketAddrRepr>) -> BsdResult<()>);

    dispatch_bsd!(get_peer_name(
        &self,
        sockfd: i32,
        addrs: OutAutoSelectBuffer<SocketAddrRepr>
    ) -> BsdResult<u32>);

    dispatch_bsd!(get_socket_name(
        &self,
        sockfd: i32,
        addrs: OutAutoSelectBuffer<SocketAddrRepr>
    ) -> BsdResult<u32>);

    dispatch_bsd!(get_sock_opt(
        &self,
        sockfd: i32,
        level: i32,
        optname: i32,
        out_opt_buffer: OutAutoSelectBuffer<u8>
    ) -> BsdResult<u32>);

    dispatch_bsd!(listen(&self, sockfd: i32, backlog: i32) -> BsdResult<()>);

    dispatch_bsd!(fnctl(&self, socfd: i32, cmd: i32, flags: i32) -> BsdResult<()>);

    dispatch_bsd!(set_sock_opt(
        &self,
        sockfd: i32,
        level: i32,
        optname: i32,
        opt_buffer: InAutoSelectBuffer<u8>
    ) -> BsdResult<()>);

    dispatch_bsd!(shutdown(&self, sockfd: i32, how: ShutdownMode) -> BsdResult<()>);

    dispatch_bsd!(shutdown_all(&self, how: ShutdownMode) -> BsdResult<()>);

    dispatch_bsd!(write(&self, sockfd: i32, data: InAutoSelectBuffer<u8>) -> BsdResult<()>);

    dispatch_bsd!(read(&self, sockfd: i32, buffer: OutAutoSelectBuffer<u8>) -> BsdResult<()>);

    dispatch_bsd!(close(&self, sockfd: i32) -> BsdResult<()>);

    dispatch_bsd!(dup_fd(&self, fd: i32, zero: u64) -> BsdResult<()>);
    dispatch_bsd!(recv_mmesg(
        &self,
        fd: i32,
        buffer: OutMapAliasBuffer<u8>,
        vlen: i32,
        flags: ReadFlags,
        timeout: TimeSpec
    ) -> BsdResult<()>);

    dispatch_bsd!(send_mmesg(
        &self,
        fd: i32,
        buffer: OutMapAliasBuffer<u8>,
        vlen: i32,
        flags: SendFlags
    ) -> BsdResult<()>);
      */
}

impl Drop for SocketService {
    fn drop(&mut self) {
        let _ = self._monitor_service.close_session();
        let _ = self.service.close_session();
        let _ = crate::svc::close_handle(self.tmem_handle);
        let _ = wait_for_permission(self._tmem_buffer.ptr as _, MemoryPermission::Write(), None);
    }
}

static BSD_SERVICE: RwLock<Option<SocketService>> = RwLock::new(None);

pub fn initialize(
    kind: BsdSrvkind,
    config: BsdServiceConfig,
    tmem_buffer: Option<Buffer<u8>>,
) -> Result<()> {
    let mut service_handle = BSD_SERVICE.write();

    if service_handle.is_some() {
        return Ok(());
    }

    *service_handle = Some(SocketService::new(config, kind, tmem_buffer)?);

    Ok(())
}

pub(crate) fn finalize() {
    *BSD_SERVICE.write() = None;
}

pub fn read_socket_service<'a>() -> ReadGuard<'a, Option<SocketService>> {
    BSD_SERVICE.read()
}

pub fn write_socket_service<'a>() -> WriteGuard<'a, Option<SocketService>> {
    BSD_SERVICE.write()
}

pub mod net {
    use core::{mem::offset_of, net::Ipv4Addr, str::FromStr};

    use super::rc;
    use crate::{
        ipc::sf::Buffer,
        result::{Result, ResultBase, ResultCode},
        service::socket::{BsdResult, ReadFlags, SendFlags, SocketAddrRepr},
        socket::BSD_SERVICE,
    };

    pub struct TcpListener(i32);

    impl TcpListener {
        pub fn bind(ip: Ipv4Addr, port: u16) -> Result<Self> {
            let socket_server_handle = BSD_SERVICE.read();

            let socket_server = socket_server_handle
                .as_ref()
                .ok_or(rc::ResultNotInitialized::make())?;
            let ipaddr = SocketAddrRepr::from((ip, port));
            //let ipaddr = SocketAddrRepr::from_str(ipaddr).map_err(|_| rc::ResultInvalidSocketString::make())?;
            let listenfd = match socket_server.service.socket(
                super::SocketDomain::INet,
                super::SocketType::Stream,
                super::IpProto::Ip,
            )? {
                BsdResult::Ok(ret, ()) => ret,
                BsdResult::Err(errno) => {
                    return ResultCode::new_err(nx::result::pack_value(
                        rc::RESULT_MODULE,
                        1000 + errno.cast_unsigned(),
                    ));
                }
            };

            let yes = 1i32;
            if let BsdResult::Err(errno) = socket_server.service.set_sock_opt(listenfd, 0xffff /* SOL_SOCKET */, 4 /*SO_REUSEADDR */, Buffer::from_other_var(&yes))? {
                return ResultCode::new_err(nx::result::pack_value(
                    rc::RESULT_MODULE,
                    1000 + errno.cast_unsigned(),
                ));
            }
            
            if let BsdResult::Err(errno) =socket_server.service.bind(listenfd, Buffer::from_var(&ipaddr))? {
                return ResultCode::new_err(nx::result::pack_value(
                    rc::RESULT_MODULE,
                    1000 + errno.cast_unsigned(),
                ));
            };

            if let BsdResult::Err(errno) =socket_server.service.listen(listenfd, 5)? {
                return ResultCode::new_err(nx::result::pack_value(
                    rc::RESULT_MODULE,
                    1000 + errno.cast_unsigned(),
                ));
            };

            Ok(Self(listenfd))
        }

        pub fn accept(&self) -> Result<(TcpStream, SocketAddrRepr)> {
            let socket_server_handle = BSD_SERVICE.read();

            let socket_server = socket_server_handle
                .as_ref()
                .ok_or(rc::ResultNotInitialized::make())?;

            let mut out_ip: SocketAddrRepr = Default::default();

            match socket_server.service.accept(self.0, Buffer::from_mut_var(&mut out_ip))? {
                BsdResult::Ok(new_sock, written_sockaddr_size  ) => { 
                    debug_assert!(written_sockaddr_size as usize >= offset_of!(SocketAddrRepr, _zero), "Invalid write length for returned socket addr");
                    Ok((TcpStream(new_sock), out_ip))
                },
                BsdResult::Err(errno) => {
                    ResultCode::new_err(nx::result::pack_value(
                        rc::RESULT_MODULE,
                        1000 + errno.cast_unsigned(),
                    ))
                }
            }
        }

        pub fn get_socket_name(&self) -> Result<SocketAddrRepr> {
            let socket_server_handle = BSD_SERVICE.read();

            let socket_server = socket_server_handle
                .as_ref()
                .ok_or(rc::ResultNotInitialized::make())?;

                let mut out_ip: SocketAddrRepr = Default::default();
                match socket_server.service.get_socket_name(self.0, Buffer::from_mut_var(&mut out_ip))? {
                    BsdResult::Ok(_, written_sockaddr_size  ) => { 
                        debug_assert!(written_sockaddr_size as usize >= offset_of!(SocketAddrRepr, _zero), "Invalid write length for returned socket addr");
                        Ok(out_ip)
                    },
                    BsdResult::Err(errno) => {
                        ResultCode::new_err(nx::result::pack_value(
                            rc::RESULT_MODULE,
                            1000 + errno.cast_unsigned(),
                        ))
                    }
                }

        }
    }
    pub struct TcpStream(i32);

    impl TcpStream {
        pub fn recv(&mut self, data: &mut [u8]) -> Result<usize> {
            let socket_server_handle = BSD_SERVICE.read();

            let socket_server = socket_server_handle
                .as_ref()
                .unwrap();

            match socket_server.service.recv(self.0, ReadFlags::None(), Buffer::from_mut_array(data))? {
                BsdResult::Ok(ret, ()) => {
                    Ok(ret as usize)
                },
                BsdResult::Err(errno) => {
                    ResultCode::new_err(nx::result::pack_value(
                        rc::RESULT_MODULE,
                        1000 + errno.cast_unsigned(),
                    ))
                }
            }
        }

        pub fn recv_non_blocking(&mut self, data: &mut [u8]) -> Result<usize> {
            let socket_server_handle = BSD_SERVICE.read();

            let socket_server = socket_server_handle
                .as_ref()
                .unwrap();

            match socket_server.service.recv(self.0, ReadFlags::DontWait(), Buffer::from_mut_array(data))? {
                BsdResult::Ok(ret, ()) => {
                    Ok(ret as usize)
                },
                BsdResult::Err(errno) if errno == 11 /* EAGAIN */ => {
                    Ok(0)
                }
                BsdResult::Err(errno) => {
                    ResultCode::new_err(nx::result::pack_value(
                        rc::RESULT_MODULE,
                        1000 + errno.cast_unsigned(),
                    ))
                }
            }
        }

        pub fn send(&mut self, data: &[u8]) -> Result<()> {
            let socket_server_handle = BSD_SERVICE.read();

            let socket_server = socket_server_handle
                .as_ref()
                .unwrap();

            match socket_server.service.send(self.0, SendFlags::None(), Buffer::from_array(data))? {
                BsdResult::Ok(_, ()) => {
                    Ok(())
                },
                BsdResult::Err(errno) => {
                    ResultCode::new_err(nx::result::pack_value(
                        rc::RESULT_MODULE,
                        1000 + errno.cast_unsigned(),
                    ))
                }
            }
        }
    }

    pub struct UdpSocket(i32);

    impl UdpSocket {
        pub fn bind() -> Result<Self> {
            let socket_server_handle = BSD_SERVICE.read();

            let socket_server = socket_server_handle
                .as_ref()
                .ok_or(rc::ResultNotInitialized::make())?;
            let ipaddr = core::net::Ipv4Addr::new(0, 0, 0, 0).into();
            //let ipaddr = SocketAddrRepr::from_str(ipaddr).map_err(|_| rc::ResultInvalidSocketString::make())?;
            let socket = match socket_server.service.socket(
                super::SocketDomain::INet,
                super::SocketType::DataGram,
                super::IpProto::UDP,
            )? {
                BsdResult::Ok(ret, ()) => ret,
                BsdResult::Err(errno) => {
                    return ResultCode::new_err(nx::result::pack_value(
                        rc::RESULT_MODULE,
                        1000 + errno.cast_unsigned(),
                    ));
                }
            };
            
            if let BsdResult::Err(errno) =socket_server.service.bind(socket, Buffer::from_var(&ipaddr))? {
                return ResultCode::new_err(nx::result::pack_value(
                    rc::RESULT_MODULE,
                    1000 + errno.cast_unsigned(),
                ));
            };
            Err(rc::ResultNotInitialized::make())
        }
    }

    mod sys {}
}
