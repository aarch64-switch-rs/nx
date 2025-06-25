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

pub use crate::service::bsd::*;

use crate::service::new_service_object;
use crate::svc::Handle;
use crate::svc::MemoryPermission;
use crate::sync::{ReadGuard, RwLock, WriteGuard};

pub struct BsdSocketService {
    _tmem_buffer: Buffer<u8>,
    tmem_handle: Handle,
    service: Box<dyn IBsdClient + Send + 'static>,
    _monitor_service: Box<dyn IBsdClient + Send + 'static>,
    _bsd_client_pid: u64,
}

unsafe impl Sync for BsdSocketService {}
unsafe impl Send for BsdSocketService {}

impl BsdSocketService {
    fn new(
        config: BsdServiceConfig,
        kind: BsdSrvkind,
        transfer_mem_buffer: Option<Buffer<u8>>,
    ) -> Result<Self> {
        let mut service = match kind {
            BsdSrvkind::Applet => Box::new(new_service_object::<AppletBsdService>()?)
                as Box<dyn IBsdClient + Send + Sync + 'static>,
            BsdSrvkind::System => Box::new(new_service_object::<SystemBsdService>()?)
                as Box<dyn IBsdClient + Send + Sync + 'static>,
            BsdSrvkind::User => Box::new(new_service_object::<UserBsdService>()?)
                as Box<dyn IBsdClient + Send + Sync + 'static>,
        };

        let mut monitor_service = match kind {
            BsdSrvkind::Applet => Box::new(new_service_object::<AppletBsdService>()?)
                as Box<dyn IBsdClient + Send + Sync + 'static>,
            BsdSrvkind::System => Box::new(new_service_object::<SystemBsdService>()?)
                as Box<dyn IBsdClient + Send + Sync + 'static>,
            BsdSrvkind::User => Box::new(new_service_object::<UserBsdService>()?)
                as Box<dyn IBsdClient + Send + Sync + 'static>,
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

        let bsd_client_pid = service.register_client(
            config,
            sf::ProcessId::new(),
            tmem_buffer.layout.size(),
            CopyHandle::from(tmem_handle),
        )?;

        monitor_service.start_monitoring(sf::ProcessId::from(bsd_client_pid))?;

        Ok(Self {
            _tmem_buffer: tmem_buffer,
            tmem_handle,
            service,
            _monitor_service: monitor_service,
            _bsd_client_pid: bsd_client_pid,
        })
    }
}

impl Drop for BsdSocketService {
    fn drop(&mut self) {
        self._monitor_service.close_session();
        self.service.close_session();
        let _ = crate::svc::close_handle(self.tmem_handle);
        let _ = wait_for_permission(self._tmem_buffer.ptr as _, MemoryPermission::Write(), None);
    }
}

static BSD_SERVICE: RwLock<Option<BsdSocketService>> = RwLock::new(None);

pub fn initialize(
    kind: BsdSrvkind,
    config: BsdServiceConfig,
    tmem_buffer: Option<Buffer<u8>>,
) -> Result<()> {
    let mut service_handle = BSD_SERVICE.write();

    if service_handle.is_some() {
        return Ok(());
    }

    *service_handle = Some(BsdSocketService::new(config, kind, tmem_buffer)?);

    Ok(())
}

pub(crate) fn finalize() {
    *BSD_SERVICE.write() = None;
}

pub fn read_socket_service<'a>() -> ReadGuard<'a, Option<BsdSocketService>> {
    BSD_SERVICE.read()
}

pub fn write_socket_service<'a>() -> WriteGuard<'a, Option<BsdSocketService>> {
    BSD_SERVICE.write()
}

pub mod net {
    use core::time::Duration;
    use core::{mem::offset_of, net::Ipv4Addr};

    use alloc::vec::Vec;

    use super::*;
    use crate::ipc::sf::OutAutoSelectBuffer;
    use crate::service::bsd::PollFlags;
    use crate::service::bsd::{PollFd, SocketOptions};
    use crate::socket::{BsdDuration, Linger, SOL_SOCKET};
    use crate::{
        ipc::sf::Buffer,
        result::{Result, ResultBase, ResultCode},
        service::bsd::{BsdResult, ReadFlags, SendFlags, SocketAddrRepr},
    };

    mod sealed {
        use super::*;

        pub trait Pollable {
            fn get_poll_fd(&self) -> i32;
        }

        impl<T: SocketCommon> Pollable for T {
            fn get_poll_fd(&self) -> i32 {
                self.as_raw_fd()
            }
        }

        pub trait SocketCommon {
            fn as_raw_fd(&self) -> i32;

            fn read(&mut self, data: &mut [u8]) -> Result<usize> {
                let socket_server_handle = BSD_SERVICE.read();
                let socket_server = socket_server_handle.as_ref().unwrap();

                match socket_server.service.recv(
                    self.as_raw_fd(),
                    ReadFlags::None(),
                    Buffer::from_mut_array(data),
                )? {
                    BsdResult::Ok(ret, ()) => Ok(ret as usize),
                    BsdResult::Err(errno) => ResultCode::new_err(nx::result::pack_value(
                        rc::RESULT_MODULE,
                        1000 + errno.cast_unsigned(),
                    )),
                }
            }

            fn read_non_blocking(&mut self, data: &mut [u8]) -> Result<usize> {
                let socket_server_handle = BSD_SERVICE.read();
                let socket_server = socket_server_handle.as_ref().unwrap();

                match socket_server.service.recv(self.as_raw_fd(), ReadFlags::DontWait(), Buffer::from_mut_array(data))? {
                    BsdResult::Ok(ret, ()) => {
                        Ok(ret as usize)
                    },
                    BsdResult::Err(11) /* EAGAIN */ => {
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

            fn peek(&mut self, data: &mut [u8]) -> Result<usize> {
                let socket_server_handle = BSD_SERVICE.read();
                let socket_server = socket_server_handle.as_ref().unwrap();

                match socket_server.service.recv(
                    self.as_raw_fd(),
                    ReadFlags::Peek(),
                    Buffer::from_mut_array(data),
                )? {
                    BsdResult::Ok(ret, ()) => Ok(ret as usize),
                    BsdResult::Err(errno) => ResultCode::new_err(nx::result::pack_value(
                        rc::RESULT_MODULE,
                        1000 + errno.cast_unsigned(),
                    )),
                }
            }

            /// Sends data on the socket to the remote address to which it is connected.
            /// For TCP, all data is sent or an error is returned.
            ///
            /// Returns the length of the data written from the buffer
            /// 
            /// `Self::connect`` will connect this socket to a remote address. This method will fail if the socket is not connected.
            fn send(&mut self, data: &[u8]) -> Result<u32> {
                let socket_server_handle = BSD_SERVICE.read();
                let socket_server = socket_server_handle.as_ref().unwrap();

                match socket_server.service.send(
                    self.as_raw_fd(),
                    SendFlags::None(),
                    Buffer::from_array(data),
                )? {
                    BsdResult::Ok(len, ()) => Ok(len.cast_unsigned()),
                    BsdResult::Err(errno) => ResultCode::new_err(nx::result::pack_value(
                        rc::RESULT_MODULE,
                        1000 + errno.cast_unsigned(),
                    )),
                }
            }

            fn send_non_blocking(&mut self, data: &[u8]) -> Result<()> {
                let socket_server_handle = BSD_SERVICE.read();
                let socket_server = socket_server_handle.as_ref().unwrap();

                match socket_server.service.send(
                    self.as_raw_fd(),
                    SendFlags::DontWait(),
                    Buffer::from_array(data),
                )? {
                    BsdResult::Ok(_, ()) => Ok(()),
                    BsdResult::Err(errno) => ResultCode::new_err(nx::result::pack_value(
                        rc::RESULT_MODULE,
                        1000 + errno.cast_unsigned(),
                    )),
                }
            }

            /// Receives data on the socket from the remote address to which it is connected.
            /// On success, returns the number of bytes read or a None value if there is no data.
            ///
            ///The function must be called with valid byte array buf of sufficient size to hold the message bytes. If a message is too long to fit in the supplied buffer, excess bytes may be discarded.
            ///
            /// `UdpSocket::connect`` will connect this socket to a remote address. This method will fail if the socket is not connected.
            fn recv_non_blocking(&mut self, buffer: &mut [u8]) -> Result<Option<usize>> {
                let socket_server_handle = BSD_SERVICE.read();
                let socket_server = socket_server_handle.as_ref().unwrap();

                match socket_server.service.recv(self.as_raw_fd(), ReadFlags::DontWait(), Buffer::from_mut_array(buffer))? {
                BsdResult::Ok(ret, ()) => {
                    Ok(Some(ret as usize))
                },
                BsdResult::Err(11) /* EAGAIN */ => {
                    Ok(None)
                }
                BsdResult::Err(errno) => {
                    ResultCode::new_err(nx::result::pack_value(
                        rc::RESULT_MODULE,
                        1000 + errno.cast_unsigned(),
                    ))
                }
            }
            }

            
        fn local_addr(&self) -> Result<SocketAddrRepr> {
            let socket_server_handle = BSD_SERVICE.read();

            let socket_server = socket_server_handle.as_ref().unwrap();

            let mut out_ip: SocketAddrRepr = Default::default();
            match socket_server
                .service
                .get_socket_name(self.as_raw_fd(), Buffer::from_mut_var(&mut out_ip))?
            {
                BsdResult::Ok(_, written_sockaddr_size) => {
                    debug_assert!(
                        written_sockaddr_size as usize >= offset_of!(SocketAddrRepr, _zero),
                        "Invalid write length for returned socket addr"
                    );
                    Ok(out_ip)
                }
                BsdResult::Err(errno) => ResultCode::new_err(nx::result::pack_value(
                    rc::RESULT_MODULE,
                    1000 + errno.cast_unsigned(),
                )),
            }
        }

            fn peer_addr(&self) -> Result<SocketAddrRepr> {
                let socket_server_handle = BSD_SERVICE.read();
                let socket_server = socket_server_handle.as_ref().unwrap();

                let mut out_ip: SocketAddrRepr = Default::default();
                match socket_server
                    .service
                    .get_peer_name(self.as_raw_fd(), Buffer::from_mut_var(&mut out_ip))?
                {
                    BsdResult::Ok(_, written_sockaddr_size) => {
                        debug_assert!(
                            written_sockaddr_size as usize >= offset_of!(SocketAddrRepr, _zero),
                            "Invalid write length for returned socket addr"
                        );
                        Ok(out_ip)
                    }
                    BsdResult::Err(errno) => ResultCode::new_err(nx::result::pack_value(
                        rc::RESULT_MODULE,
                        1000 + errno.cast_unsigned(),
                    )),
                }
            }

            fn set_ttl(&self, ttl: u32) -> Result<()> {
                let socket_server_handle = BSD_SERVICE.read();
                let socket_server = socket_server_handle.as_ref().unwrap();

                if let BsdResult::Err(errno) = socket_server.service.set_sock_opt(
                    self.as_raw_fd(),
                    0, /* IPPROTO_IP */
                    4, /* IP_TTL */
                    Buffer::from_other_var(&ttl),
                )? {
                    return ResultCode::new_err(nx::result::pack_value(
                        rc::RESULT_MODULE,
                        1000 + errno.cast_unsigned(),
                    ));
                }

                Ok(())
            }

            fn ttl(&self) -> Result<u32> {
                let socket_server_handle = BSD_SERVICE.read();
                let socket_server = socket_server_handle.as_ref().unwrap();

                let mut ttl: u32 = 0;
                if let BsdResult::Err(errno) = socket_server.service.get_sock_opt(
                    self.as_raw_fd(),
                    0, /* IPPROTO_IP */
                    4, /* IP_TTL */
                    Buffer::from_other_mut_var(&mut ttl),
                )? {
                    return ResultCode::new_err(nx::result::pack_value(
                        rc::RESULT_MODULE,
                        1000 + errno.cast_unsigned(),
                    ));
                }

                Ok(ttl)
            }

            fn set_nonblocking(&self, nonblocking: bool) -> Result<()> {
                const O_NONBLOCK: i32 = 0x4000;

                let socket_server_handle = BSD_SERVICE.read();
                let socket_server = socket_server_handle.as_ref().unwrap();

                let current_flags = match socket_server.service.fnctl(
                    self.as_raw_fd(),
                    super::FnCtlCmd::GetFl,
                    0,
                )? {
                    BsdResult::Ok(flags, ()) => flags,
                    BsdResult::Err(errno) => {
                        return ResultCode::new_err(nx::result::pack_value(
                            rc::RESULT_MODULE,
                            1000 + errno.cast_unsigned(),
                        ));
                    }
                };

                let flags = if nonblocking {
                    current_flags | O_NONBLOCK
                } else {
                    current_flags & !O_NONBLOCK
                };

                if let BsdResult::Err(errno) =
                    socket_server
                        .service
                        .fnctl(self.as_raw_fd(), super::FnCtlCmd::SetFl, flags)?
                {
                    return ResultCode::new_err(nx::result::pack_value(
                        rc::RESULT_MODULE,
                        1000 + errno.cast_unsigned(),
                    ));
                }

                Ok(())
            }

            fn read_timeout(&self) -> Result<Option<Duration>> {
                let socket_server_handle = BSD_SERVICE.read();
                let socket_server = socket_server_handle.as_ref().unwrap();

                let mut timeout: BsdDuration = Default::default();
                match socket_server.service.get_sock_opt(
                    self.as_raw_fd(),
                    SOL_SOCKET,
                    SocketOptions::RCVTIMEO as _,
                    Buffer::from_other_mut_var(&mut timeout),
                )? {
                    BsdResult::Ok(_, written_len) => {
                        debug_assert_eq!(written_len as usize, size_of::<BsdDuration>())
                    }
                    BsdResult::Err(errno) => {
                        return ResultCode::new_err(nx::result::pack_value(
                            rc::RESULT_MODULE,
                            1000 + errno.cast_unsigned(),
                        ));
                    }
                }

                if timeout == Default::default() {
                    Ok(None)
                } else {
                    Ok(Some(timeout.into()))
                }
            }

            fn set_read_timeout(&self, timeout: Option<Duration>) -> Result<()> {
                let socket_server_handle = BSD_SERVICE.read();
                let socket_server = socket_server_handle.as_ref().unwrap();

                let timeout: BsdDuration = timeout.into();
                if let BsdResult::Err(errno) = socket_server.service.set_sock_opt(
                    self.as_raw_fd(),
                    SOL_SOCKET,
                    SocketOptions::RCVTIMEO as _,
                    Buffer::from_other_var(&timeout),
                )? {
                    return ResultCode::new_err(nx::result::pack_value(
                        rc::RESULT_MODULE,
                        1000 + errno.cast_unsigned(),
                    ));
                }

                Ok(())
            }

            fn write_timeout(&self) -> Result<Option<Duration>> {
                let socket_server_handle = BSD_SERVICE.read();
                let socket_server = socket_server_handle.as_ref().unwrap();

                let mut timeout: BsdDuration = Default::default();
                match socket_server.service.get_sock_opt(
                    self.as_raw_fd(),
                    SOL_SOCKET,
                    SocketOptions::SNDTIMEO as _,
                    Buffer::from_other_mut_var(&mut timeout),
                )? {
                    BsdResult::Ok(_, written_len) => {
                        debug_assert_eq!(written_len as usize, size_of::<BsdDuration>())
                    }
                    BsdResult::Err(errno) => {
                        return ResultCode::new_err(nx::result::pack_value(
                            rc::RESULT_MODULE,
                            1000 + errno.cast_unsigned(),
                        ));
                    }
                }

                if timeout == Default::default() {
                    Ok(None)
                } else {
                    Ok(Some(timeout.into()))
                }
            }

            fn set_write_timeout(&self, timeout: Option<Duration>) -> Result<()> {
                let socket_server_handle = BSD_SERVICE.read();
                let socket_server = socket_server_handle.as_ref().unwrap();

                let timeout: BsdDuration = timeout.into();
                if let BsdResult::Err(errno) = socket_server.service.set_sock_opt(
                    self.as_raw_fd(),
                    SOL_SOCKET,
                    SocketOptions::SNDTIMEO as _,
                    Buffer::from_other_var(&timeout),
                )? {
                    return ResultCode::new_err(nx::result::pack_value(
                        rc::RESULT_MODULE,
                        1000 + errno.cast_unsigned(),
                    ));
                }

                Ok(())
            }
        }
    }

    use sealed::{Pollable, SocketCommon};

    /// Takes a slice of pollable values and requested events returns an iterator over the matched index in the input list and the returned events.
    #[inline(always)]
    pub fn poll<P: sealed::Pollable>(
        pollers: &[(P, PollFlags)],
        timeout: Option<i32>,
    ) -> Result<impl Iterator<Item = (usize, PollFlags)>> {
        poll_impl(
            pollers
                .iter()
                .map(|(poll, flags)| PollFd {
                    fd: poll.get_poll_fd(),
                    events: *flags,
                    revents: Default::default(),
                })
                .collect(),
            timeout.unwrap_or(-1),
        )
    }

    #[doc(hidden)]
    fn poll_impl(
        mut fds: Vec<PollFd>,
        timeout: i32,
    ) -> Result<impl Iterator<Item = (usize, PollFlags)>> {
        let socket_server_handle = BSD_SERVICE.read();

        let socket_server = socket_server_handle
            .as_ref()
            .ok_or(rc::ResultNotInitialized::make())?;

        if let BsdResult::Err(errno) = socket_server
            .service
            .poll(Buffer::from_mut_array(fds.as_mut_slice()), timeout)?
        {
            return ResultCode::new_err(nx::result::pack_value(
                rc::RESULT_MODULE,
                1000 + errno.cast_unsigned(),
            ));
        }

        Ok(fds.into_iter().enumerate().filter_map(|(index, pollfd)| {
            if pollfd.events.intersects(pollfd.revents) {
                Some((index, pollfd.revents))
            } else {
                None
            }
        }))
    }

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
            if let BsdResult::Err(errno) = socket_server.service.set_sock_opt(
                listenfd,
                SOL_SOCKET,
                SocketOptions::ReuseAddr as i32,
                Buffer::from_other_var(&yes),
            )? {
                return ResultCode::new_err(nx::result::pack_value(
                    rc::RESULT_MODULE,
                    1000 + errno.cast_unsigned(),
                ));
            }

            if let BsdResult::Err(errno) = socket_server
                .service
                .bind(listenfd, Buffer::from_var(&ipaddr))?
            {
                return ResultCode::new_err(nx::result::pack_value(
                    rc::RESULT_MODULE,
                    1000 + errno.cast_unsigned(),
                ));
            };

            if let BsdResult::Err(errno) = socket_server.service.listen(listenfd, 5)? {
                return ResultCode::new_err(nx::result::pack_value(
                    rc::RESULT_MODULE,
                    1000 + errno.cast_unsigned(),
                ));
            };

            Ok(Self(listenfd))
        }

        pub fn accept(&self) -> Result<(TcpStream, SocketAddrRepr)> {
            let socket_server_handle = BSD_SERVICE.read();

            let socket_server = socket_server_handle.as_ref().unwrap();

            let mut out_ip: SocketAddrRepr = Default::default();

            match socket_server
                .service
                .accept(self.get_poll_fd(), Buffer::from_mut_var(&mut out_ip))?
            {
                BsdResult::Ok(new_sock, written_sockaddr_size) => {
                    debug_assert!(
                        written_sockaddr_size as usize >= offset_of!(SocketAddrRepr, _zero),
                        "Invalid write length for returned socket addr"
                    );
                    Ok((TcpStream(new_sock), out_ip))
                }
                BsdResult::Err(errno) => ResultCode::new_err(nx::result::pack_value(
                    rc::RESULT_MODULE,
                    1000 + errno.cast_unsigned(),
                )),
            }
        }

        pub fn local_addr(&self) -> Result<SocketAddrRepr> {
            let socket_server_handle = BSD_SERVICE.read();

            let socket_server = socket_server_handle.as_ref().unwrap();

            let mut out_ip: SocketAddrRepr = Default::default();
            match socket_server
                .service
                .get_socket_name(self.0, Buffer::from_mut_var(&mut out_ip))?
            {
                BsdResult::Ok(_, written_sockaddr_size) => {
                    debug_assert!(
                        written_sockaddr_size as usize >= offset_of!(SocketAddrRepr, _zero),
                        "Invalid write length for returned socket addr"
                    );
                    Ok(out_ip)
                }
                BsdResult::Err(errno) => ResultCode::new_err(nx::result::pack_value(
                    rc::RESULT_MODULE,
                    1000 + errno.cast_unsigned(),
                )),
            }
        }

    }

    impl sealed::Pollable for TcpListener {
        fn get_poll_fd(&self) -> i32 {
            self.0
        }
    }

    pub struct TcpStream(i32);

    impl TcpStream {
        #[inline(always)]
        pub fn connect<A: Into<SocketAddrRepr>>(destination: A) -> Result<Self> {
            let destination = destination.into();
            Self::connect_impl(destination)
        }

        fn connect_impl(destination: SocketAddrRepr) -> Result<Self> {
            let socket_server_handle = BSD_SERVICE.read();

            let socket_server = socket_server_handle
                .as_ref()
                .ok_or(rc::ResultNotInitialized::make())?;

            let socket = match socket_server.service.socket(
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

            if let BsdResult::Err(errno) = socket_server
                .service
                .connect(socket, Buffer::from_var(&destination))?
            {
                return ResultCode::new_err(nx::result::pack_value(
                    rc::RESULT_MODULE,
                    1000 + errno.cast_unsigned(),
                ));
            };

            Ok(Self(socket))
        }

        
        pub fn linger(&self) -> Result<Option<Duration>> {
            let socket_server_handle = BSD_SERVICE.read();

            let socket_server = socket_server_handle.as_ref().unwrap();

            let mut linger: Linger = Default::default();
            if let BsdResult::Err(errno) = socket_server.service.get_sock_opt(
                self.0,
                SOL_SOCKET,
                SocketOptions::Linger as i32,
                OutAutoSelectBuffer::from_other_mut_var(&mut linger),
            )? {
                return ResultCode::new_err(nx::result::pack_value(
                    rc::RESULT_MODULE,
                    1000 + errno.cast_unsigned(),
                ));
            }

            Ok(linger.into())
        }

        pub fn nodelay(&self) -> Result<bool> {
            let socket_server_handle = BSD_SERVICE.read();

            let socket_server = socket_server_handle.as_ref().unwrap();

            let mut delay: i32 = 0;
            match socket_server.service.get_sock_opt(
                self.0,
                0, /* IPPROTO_IP */
                1, /* TCP_NODELAY */
                Buffer::from_other_mut_var(&mut delay),
            )? {
                BsdResult::Ok(_, written_data_len) => {
                    debug_assert_ne!(written_data_len, 0); // we're reading an i32, but we only care if it's zero or not so any sized write is valid.
                    Ok(delay != 0)
                }
                BsdResult::Err(errno) => ResultCode::new_err(nx::result::pack_value(
                    rc::RESULT_MODULE,
                    1000 + errno.cast_unsigned(),
                )),
            }
        }
        
        pub fn shutdown(&self, mode: ShutdownMode) -> Result<()> {
            let socket_server_handle = BSD_SERVICE.read();
            let socket_server = socket_server_handle.as_ref().unwrap();

            if let BsdResult::Err(errno) = socket_server.service.shutdown(
                self.0,
                mode)? {
                return ResultCode::new_err(nx::result::pack_value(
                    rc::RESULT_MODULE,
                    1000 + errno.cast_unsigned(),
                ));
            }

            Ok(())
        }
    }

    impl sealed::SocketCommon for TcpStream {
        fn as_raw_fd(&self) -> i32 {
            self.0
        }
    }
    pub struct UdpSocket(i32);

    impl UdpSocket {
        pub fn bind<A: Into<Ipv4Addr>>(addr: A, port: Option<u16>) -> Result<Self> {
            let socket_server_handle = BSD_SERVICE.read();

            let socket_server = socket_server_handle
                .as_ref()
                .ok_or(rc::ResultNotInitialized::make())?;
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

            let ipaddr: Ipv4Addr = addr.into();
            let socketaddr = SocketAddrRepr::from((ipaddr, port.unwrap_or(0)));
            if let BsdResult::Err(errno) = socket_server
                .service
                .bind(socket, Buffer::from_var(&socketaddr))?
            {
                return ResultCode::new_err(nx::result::pack_value(
                    rc::RESULT_MODULE,
                    1000 + errno.cast_unsigned(),
                ));
            };
            Err(rc::ResultNotInitialized::make())
        }

        #[inline(always)]
        pub fn connect<A: Into<SocketAddrRepr>>(destination: A) -> Result<Self> {
            let destination = destination.into();
            Self::connect_impl(destination)
        }

        fn connect_impl(destination: SocketAddrRepr) -> Result<Self> {
            let socket_server_handle = BSD_SERVICE.read();

            let socket_server = socket_server_handle
                .as_ref()
                .ok_or(rc::ResultNotInitialized::make())?;

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

            if let BsdResult::Err(errno) = socket_server
                .service
                .connect(socket, Buffer::from_var(&destination))?
            {
                return ResultCode::new_err(nx::result::pack_value(
                    rc::RESULT_MODULE,
                    1000 + errno.cast_unsigned(),
                ));
            };

            Ok(Self(socket))
        }

        /// Receives data on the socket from the remote address to which it is connected.
        /// On success, returns the number of bytes read and the origin.
        ///
        ///The function must be called with valid byte array buf of sufficient size to hold the message bytes. If a message is too long to fit in the supplied buffer, excess bytes may be discarded.
        pub fn read_from(&mut self, buffer: &mut [u8]) -> Result<(usize, Ipv4Addr, u16)> {
            let socket_server_handle = BSD_SERVICE.read();

            let socket_server = socket_server_handle.as_ref().unwrap();

            let mut out_addr: SocketAddrRepr = Default::default();
            match socket_server.service.recv_from(
                self.0,
                ReadFlags::None(),
                Buffer::from_mut_array(buffer),
                Buffer::from_mut_var(&mut out_addr),
            )? {
                BsdResult::Ok(ret, ()) => Ok((
                    ret as usize,
                    Ipv4Addr::from_bits(u32::from_be_bytes(out_addr.addr)),
                    u16::from_be(out_addr.port),
                )),
                BsdResult::Err(errno) => ResultCode::new_err(nx::result::pack_value(
                    rc::RESULT_MODULE,
                    1000 + errno.cast_unsigned(),
                )),
            }
        }

        /// Receives data on the socket from the remote address to which it is connected.
        /// On success, returns the number of bytes read and the origin or a None value if there is no data.
        ///
        ///The function must be called with valid byte array buf of sufficient size to hold the message bytes. If a message is too long to fit in the supplied buffer, excess bytes may be discarded.
        pub fn recv_from_non_blocking(
            &mut self,
            buffer: &mut [u8],
        ) -> Result<Option<(usize, Ipv4Addr, u16)>> {
            let socket_server_handle = BSD_SERVICE.read();

            let socket_server = socket_server_handle.as_ref().unwrap();

            let mut out_addr: SocketAddrRepr = Default::default();
            match socket_server.service.recv_from(self.0, ReadFlags::DontWait(), Buffer::from_mut_array(buffer), Buffer::from_mut_var(&mut out_addr))? {
                BsdResult::Ok(ret, ()) => {
                    Ok(Some((ret as usize, Ipv4Addr::from_bits(u32::from_be_bytes(out_addr.addr)), u16::from_be(out_addr.port))))
                },
                BsdResult::Err(11) /* EAGAIN */ => {
                    Ok(None)
                }
                BsdResult::Err(errno) => {
                    ResultCode::new_err(nx::result::pack_value(
                        rc::RESULT_MODULE,
                        1000 + errno.cast_unsigned(),
                    ))
                }
            }
        }

        /// Sends data on the socket to the remote address provided (no call to `UdpSocket::connect` is necessary).
        /// Unlike `std::net::UdpSocket`, this method does not return length of the written data.
        /// All data is sent or an error is returned.
        pub fn send_to<A: Into<SocketAddrRepr>>(
            &mut self,
            data: &[u8],
            destinaation: A,
        ) -> Result<()> {
            let socket_server_handle = BSD_SERVICE.read();

            let socket_server = socket_server_handle.as_ref().unwrap();

            let destination = destinaation.into();
            match socket_server.service.send_to(
                self.0,
                SendFlags::None(),
                Buffer::from_array(data),
                Buffer::from_var(&destination),
            )? {
                BsdResult::Ok(_, ()) => Ok(()),
                BsdResult::Err(errno) => ResultCode::new_err(nx::result::pack_value(
                    rc::RESULT_MODULE,
                    1000 + errno.cast_unsigned(),
                )),
            }
        }
    }

    impl core::fmt::Write for UdpSocket {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            match self.send(s.as_bytes()) {
                Ok(_) => Ok(()),
                Err(_) => Err(core::fmt::Error)
            }
        }
    }

    impl sealed::SocketCommon for UdpSocket {
        fn as_raw_fd(&self) -> i32 {
            self.0
        }
    }

    mod sys {}
}
