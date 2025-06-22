use nx_derive::{Request, Response};

use crate::ipc::sf::{
    CopyHandle, InAutoSelectBuffer, InOutAutoSelectBuffer, OutAutoSelectBuffer, OutMapAliasBuffer,
    ProcessId,
};
use crate::result::Result;
use crate::version::{self, Version, VersionInterval};

use core::net::Ipv4Addr;
use core::str::FromStr;
use core::time::Duration as TimeSpec;

pub mod rc;

#[derive(Copy, Clone, Debug, Request, Response)]
#[repr(C)]
pub struct LibraryVersion(u32);

macro_rules! version_in {
    ($version:ident, $from:expr, $to:expr) => {
        VersionInterval::from_to(
            Version::new($from.0, $from.1, $from.2),
            Version::new($to.0, $to.1, $to.2),
        )
        .contains($version)
    };
}

impl LibraryVersion {
    fn new() -> Self {
        let version = version::get_version();

        if version_in!(version, (1, 0, 0), (2, 3, 0)) {
            Self(1)
        } else if version_in!(version, (3, 0, 0), (3, 0, 2)) {
            Self(2)
        } else if version_in!(version, (4, 0, 0), (4, 1, 0)) {
            Self(3)
        } else if version_in!(version, (5, 0, 0), (5, 1, 0)) {
            Self(4)
        } else if version_in!(version, (6, 0, 0), (7, 0, 1)) {
            Self(5)
        } else if version_in!(version, (8, 0, 0), (8, 1, 1)) {
            Self(6)
        } else if version_in!(version, (9, 0, 0), (12, 1, 0)) {
            Self(7)
        } else if version_in!(version, (13, 0, 0), (15, 0, 1)) {
            Self(8)
        } else if version_in!(version, (16, 0, 0), (18, 1, 0)) {
            Self(9)
        } else
        /*if version >= Version::new(19,0, 0)*/
        {
            Self(10)
        }
    }
}

impl Default for LibraryVersion {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Copy, Clone, Debug, Request, Response)]
pub struct BsdServiceConfig {
    /// Observed 1 on 2.0 LibAppletWeb, 2 on 3.0.
    pub version: LibraryVersion,

    /// Size of the TCP transfer (send) buffer (initial or fixed).
    pub tcp_tx_buf_size: u32,
    /// Size of the TCP recieve buffer (initial or fixed).
    pub tcp_rx_buf_size: u32,
    /// Maximum size of the TCP transfer (send) buffer. If it is 0, the size of the buffer is fixed to its initial value.
    pub tcp_tx_buf_max_size: u32,
    /// Maximum size of the TCP receive buffer. If it is 0, the size of the buffer is fixed to its initial value.
    pub tcp_rx_buf_max_size: u32,

    /// Size of the UDP transfer (send) buffer (typically 0x2400 bytes).
    pub udp_tx_buf_size: u32,
    /// Size of the UDP receive buffer (typically 0xA500 bytes).
    pub udp_rx_buf_size: u32,

    /// Number of buffers for each socket (standard values range from 1 to 8).
    pub sb_efficiency: u32,
}

impl Default for BsdServiceConfig {
    fn default() -> Self {
        Self {
            version: Default::default(),
            tcp_tx_buf_size: 0x8000,
            tcp_rx_buf_size: 0x10000,
            tcp_tx_buf_max_size: 0x40000,
            tcp_rx_buf_max_size: 0x40000,
            udp_tx_buf_size: 0x2400,
            udp_rx_buf_size: 0xA500,
            sb_efficiency: 4,
        }
    }
}

impl BsdServiceConfig {
    pub fn min_transfer_mem_size(self) -> usize {
        let tx_max_size = if self.tcp_tx_buf_max_size != 0 {
            self.tcp_tx_buf_max_size
        } else {
            self.tcp_tx_buf_size
        };
        let rx_max_size = if self.tcp_rx_buf_max_size != 0 {
            self.tcp_rx_buf_max_size
        } else {
            self.tcp_rx_buf_size
        };

        let sum = tx_max_size + rx_max_size + self.udp_tx_buf_size + self.udp_rx_buf_size;

        self.sb_efficiency as usize * align_up!(sum as usize, 0x1000usize)
    }
}

pub type Fqdn = crate::util::ArrayString<0x100>;

//#[derive(Copy, Clone, Debug)]
pub enum BsdResult<T> {
    Ok(i32, T),
    Err(i32),
}

impl<T: super::client::ResponseCommandParameter<T>>
    super::client::ResponseCommandParameter<BsdResult<T>> for BsdResult<T>
{
    fn after_response_read(
        walker: &mut crate::ipc::DataWalker,
        ctx: &mut crate::ipc::CommandContext,
    ) -> Result<BsdResult<T>> {
        let ret: i32 = walker.advance_get();
        let errno: i32 = walker.advance_get();

        if ret < 0 {
            Ok(Self::Err(errno))
        } else {
            Ok(Self::Ok(
                ret,
                <T as super::client::ResponseCommandParameter<T>>::after_response_read(
                    walker, ctx,
                )?,
            ))
        }
    }
}

impl super::client::ResponseCommandParameter<BsdResult<()>> for BsdResult<()> {
    fn after_response_read(
        walker: &mut crate::ipc::DataWalker,
        _ctx: &mut crate::ipc::CommandContext,
    ) -> Result<BsdResult<()>> {
        let ret: i32 = walker.advance_get();
        let errno: i32 = walker.advance_get();

        if ret < 0 {
            Ok(Self::Err(errno))
        } else {
            Ok(Self::Ok(ret, ()))
        }
    }
}

impl<T: for<'a> super::server::RequestCommandParameter<'a, T>>
    super::server::ResponseCommandParameter for BsdResult<T>
{
    type CarryState = ();

    fn before_response_write(
        var: &Self,
        ctx: &mut crate::ipc::server::ServerContext,
    ) -> Result<Self::CarryState> {
        match var {
            Self::Ok(_ret, _okval) => {
                ctx.raw_data_walker.advance::<i32>();
                ctx.raw_data_walker.advance::<i32>();
                ctx.raw_data_walker.advance::<T>();
            }
            Self::Err(_errval) => {
                ctx.raw_data_walker.advance::<i32>();
                ctx.raw_data_walker.advance::<i32>();
            }
        }

        Ok(())
    }

    fn after_response_write(
        var: Self,
        _carry_state: Self::CarryState,
        ctx: &mut crate::ipc::server::ServerContext,
    ) -> Result<()> {
        match var {
            Self::Ok(ret, okval) => {
                ctx.raw_data_walker.advance_set(ret);
                ctx.raw_data_walker.advance_set(0i32);
                ctx.raw_data_walker.advance_set(okval);
            }
            Self::Err(errval) => {
                ctx.raw_data_walker.advance_set(-1i32);
                ctx.raw_data_walker.advance_set(errval);
            }
        }

        Ok(())
    }
}

impl super::server::ResponseCommandParameter for BsdResult<()> {
    type CarryState = ();

    fn before_response_write(
        var: &Self,
        ctx: &mut crate::ipc::server::ServerContext,
    ) -> Result<Self::CarryState> {
        match var {
            Self::Ok(_, _) => {
                ctx.raw_data_walker.advance::<i32>();
                ctx.raw_data_walker.advance::<i32>();
            }
            Self::Err(_) => {
                ctx.raw_data_walker.advance::<i32>();
                ctx.raw_data_walker.advance::<i32>();
            }
        }

        Ok(())
    }

    fn after_response_write(
        var: Self,
        _carry_state: Self::CarryState,
        ctx: &mut crate::ipc::server::ServerContext,
    ) -> Result<()> {
        match var {
            Self::Ok(ret, ()) => {
                ctx.raw_data_walker.advance_set(ret);
                ctx.raw_data_walker.advance_set(0i32);
            }
            Self::Err(errval) => {
                ctx.raw_data_walker.advance_set(-1i32);
                ctx.raw_data_walker.advance_set(errval);
            }
        }

        Ok(())
    }
}

/*
impl BsdResult {
    pub fn to_result(self) -> Result<i32> {
        if self.0 < 0 {
            Err(ResultCode::new(self.1 as u32))
        } else {
            Ok(self.0)
        }
    }

    //fn convert_errno(_errno: i32) {}
}*/

#[derive(Copy, Clone, Debug, Default, Request, Response)]
#[repr(C)]
pub struct SocketAddrRepr {
    /// The actual size in bytes of the Socket.
    ///
    /// This gets sent over BSD API where the size of the socket is at least the size of `Self` here
    /// but it could be longer (e.g. IPV6 addresses don't fit).
    len: u8,
    /// The address family
    family: SocketDomain,
    // TCP/UDP port
    pub port: u16,
    // IPv4 Address
    pub addr: [u8;4],
    /// The min size we are working with for the true types. The real size is based on `self.actual_length` and `self.family`.
    pub(crate) _zero: [u8; 8],
}

const_assert!( core::mem::size_of::<SocketAddrRepr>() == 16);

impl FromStr for SocketAddrRepr {
    type Err = core::net::AddrParseError;
    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        let ipv4_addr = Ipv4Addr::from_str(s)?;

        Ok(SocketAddrRepr::from(ipv4_addr))
    }
}

impl From<core::net::Ipv4Addr> for SocketAddrRepr {
    fn from(value: core::net::Ipv4Addr) -> Self {
        Self {
            len: 6,
            family: SocketDomain::INet,
            port: 0,
            addr: value.octets(),
            _zero: [0; 8],
        }
    }
}

impl From<(core::net::Ipv4Addr, u16)> for SocketAddrRepr {
    fn from(value: (core::net::Ipv4Addr, u16)) -> Self {
        Self {
            len: 6,
            family: SocketDomain::INet,
            port: value.1.to_be(),
            addr: value.0.octets(),
            _zero: [0; 8],
        }
    }
}

#[derive(Copy, Clone, Debug, Default, Request, Response)]
#[repr(C)]
struct BsdDuration {
    seconds: u64,
    microseconds: u64,
}

#[derive(Copy, Clone, Debug, Request, Response)]
#[repr(C)]
#[deprecated]
/// This is newly added but immediately deprecated.
/// See `ISocketClient::select` for details.
pub struct BsdTimeout {
    timeout: BsdDuration,
    no_timeout: bool,
}

impl BsdTimeout {
    pub const fn new() -> Self {
        Self {
            timeout: BsdDuration { seconds: 0, microseconds: 0},
            no_timeout: true
        }
    }

    pub const fn timeout(timout: TimeSpec) ->  Self {
        Self {
            timeout: BsdDuration {
                seconds: timout.as_secs(),
                microseconds: timout.subsec_micros() as u64,
            },
            no_timeout: false,
        }
    }
}

impl Default for BsdTimeout {
    fn default() -> Self {
        Self::new()
    }
}


impl From<Option<core::time::Duration>> for BsdTimeout {
    fn from(value: Option<core::time::Duration>) -> Self {
        if let Some(duration) = value {
            Self {
                timeout: BsdDuration {
                    seconds: duration.as_secs(),
                    microseconds: duration.subsec_micros() as u64,
                },
                no_timeout: false,
            }
        } else {
            Self {
                no_timeout: true,
                timeout: Default::default(),
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Request, Response)]
#[repr(C)]
pub enum ShutdownMode {
    Receive = 0,
    Send = 1,
    Bidirectional = 2,
}

pub type FdSet = [u64; 1024 / (8 * core::mem::size_of::<u64>())];

#[derive(Copy, Clone, Debug, Default, Request, Response)]
#[repr(C)]
pub struct PollFd {
    pub fd: i32,
    pub events: u16,
    pub revents: u16,
}

#[derive(Copy, Clone, Debug, Default, Request, Response)]
#[repr(u8)]
pub enum SocketDomain {
    // IPV4
    #[default]
    INet = 2,
    ///Internal Routing Protocol
    Route = 17,
    // IPV6
    //INet6 = 28, - not supported?
}

#[derive(Copy, Clone, Debug, Request, Response)]
#[repr(C)]
pub enum SocketType {
    Stream = 1,
    DataGram = 2,
    Raw = 3,
    SequencePacket = 5,
}

#[derive(Copy, Clone, Debug, Request, Response)]
#[repr(C)]
pub enum IpProto {
    Ip = 0,
    ICMP = 1,
    TCP = 6,
    UDP = 17,
}

define_bit_enum! {
    /// Represents the valid flags when receiving data from a network socket.
    ReadFlags (u32) {
        None = 0,
        /// process out-of-band data
        Oob = 0x00000001,
        /// peek at incoming message
        Peek = 0x00000002,
        /// data discarded before delivery
        Trun = 0x00000010,
        /// control data lost before delivery
        CTrunc = 0x00000020,
        /// wait for full request or error
        WaitAll = 0x00000040,
        /// this message should be nonblocking
        DontWait = 0x00000080,
        /// make received fds close-on-exec
        CMsg_CloExec = 0x00040000,
        /// do not block after receiving the first message (only for `recv_mmsg()`)
        WaitForOne = 0x00080000
    }
}

define_bit_enum! {
    SendFlags (u32) {
        None = 0,
        /// process out-of-band data
        Oob = 0x00001,
        /// bypass routing, use direct interface
        DontRoute = 0x00004,
        /// data completes record
        Eor = 0x00008,
        /// do not block
        DontWait = 0x00080,
        /// data completes transaction
        Eof = 0x00100,
        /// do not generate SIGPIPE on EOF
        NoSignal = 0x20000
    }
}

#[nx_derive::ipc_trait]
pub trait Socket {
    #[ipc_rid(0)]
    fn register_client(
        &mut self,
        library_config: BsdServiceConfig,
        pid: ProcessId,
        transfer_mem_size: usize,
        tmem_handle: CopyHandle,
    ) -> u64;

    #[ipc_rid(1)]
    fn start_monitoring(&mut self, pid: ProcessId) -> ();

    #[ipc_rid(2)]
    /// See [read(2)](https://man.openbsd.org/read.2)
    fn socket(
        &self,
        domain: SocketDomain,
        sock_type: SocketType,
        protocol: IpProto,
    ) -> BsdResult<()>;

    #[ipc_rid(3)]
    fn socket_exempt(
        &self,
        domain: SocketDomain,
        sock_type: SocketType,
        protocol: IpProto,
    ) -> BsdResult<()>;

    #[ipc_rid(4)]
    fn open(&self, path_cstr: InAutoSelectBuffer<u8>, flags: i32) -> BsdResult<()>;

    #[ipc_rid(5)]
    fn select(
        &self,
        fd_cound: u32,
        read_fds: InOutAutoSelectBuffer<FdSet>,
        write_fds: InOutAutoSelectBuffer<FdSet>,
        except_fds: InOutAutoSelectBuffer<FdSet>,
        timeout: BsdTimeout,
    ) -> BsdResult<()>;

    #[ipc_rid(6)]
    fn poll(&self, fds: InOutAutoSelectBuffer<PollFd>, timeout: i32) -> BsdResult<()>;

    #[ipc_rid(7)]
    fn sysctl(
        &self,
        name: InAutoSelectBuffer<u32>,
        newp: InAutoSelectBuffer<u8>,
        oldp: OutAutoSelectBuffer<u8>,
    ) -> BsdResult<u32>;

    #[ipc_rid(8)]
    fn recv(
        &self,
        sockfd: i32,
        flags: ReadFlags,
        out_buffer: OutAutoSelectBuffer<u8>,
    ) -> BsdResult<()>;

    #[ipc_rid(9)]
    fn recv_from(
        &self,
        sockfd: i32,
        flags: ReadFlags,
        out_buffer: OutAutoSelectBuffer<u8>,
        from_addrs: OutAutoSelectBuffer<SocketAddrRepr>,
    ) -> BsdResult<()>;

    #[ipc_rid(10)]
    fn send(&self, sockfd: i32, flags: SendFlags, buffer: InAutoSelectBuffer<u8>) -> BsdResult<()>;

    #[ipc_rid(11)]
    fn send_to(
        &self,
        sockfd: i32,
        flags: SendFlags,
        buffer: InAutoSelectBuffer<u8>,
        to_addrs: InAutoSelectBuffer<SocketAddrRepr>,
    ) -> BsdResult<()>;

    #[ipc_rid(12)]
    fn accept(&self, sockfd: i32, addrs: OutAutoSelectBuffer<SocketAddrRepr>) -> BsdResult<u32>;

    #[ipc_rid(13)]
    fn bind(&self, sockfd: i32, addrs: InAutoSelectBuffer<SocketAddrRepr>) -> BsdResult<()>;

    #[ipc_rid(14)]
    fn connect(&self, sockfd: i32, addrs: InAutoSelectBuffer<SocketAddrRepr>) -> BsdResult<()>;

    #[ipc_rid(15)]
    fn get_peer_name(
        &self,
        sockfd: i32,
        addrs: OutAutoSelectBuffer<SocketAddrRepr>,
    ) -> BsdResult<u32>;

    #[ipc_rid(16)]
    fn get_socket_name(
        &self,
        sockfd: i32,
        addrs: OutAutoSelectBuffer<SocketAddrRepr>,
    ) -> BsdResult<u32>;

    #[ipc_rid(17)]
    fn get_sock_opt(
        &self,
        sockfd: i32,
        level: i32,
        optname: i32,
        out_opt_buffer: OutAutoSelectBuffer<u8>,
    ) -> BsdResult<u32>;

    #[ipc_rid(18)]
    fn listen(&self, sockfd: i32, backlog: i32) -> BsdResult<()>;

    #[ipc_rid(20)]
    fn fnctl(&self, socfd: i32, cmd: i32, flags: i32) -> BsdResult<()>;

    #[ipc_rid(21)]
    fn set_sock_opt(
        &self,
        sockfd: i32,
        level: i32,
        optname: i32,
        opt_buffer: InAutoSelectBuffer<u8>,
    ) -> BsdResult<()>;

    #[ipc_rid(22)]
    fn shutdown(&self, sockfd: i32, how: ShutdownMode) -> BsdResult<()>;

    #[ipc_rid(23)]
    fn shutdown_all(&self, how: ShutdownMode) -> BsdResult<()>;

    #[ipc_rid(24)]
    fn write(&self, sockfd: i32, data: InAutoSelectBuffer<u8>) -> BsdResult<()>;

    #[ipc_rid(25)]
    fn read(&self, sockfd: i32, buffer: OutAutoSelectBuffer<u8>) -> BsdResult<()>;

    #[ipc_rid(26)]
    fn close(&self, sockfd: i32) -> BsdResult<()>;

    #[ipc_rid(27)]
    fn dup_fd(&self, fd: i32, zero: u64) -> BsdResult<()>;

    #[ipc_rid(29)]
    #[version(VersionInterval::from(Version::new(7, 0, 0)))]
    fn recv_mmesg(
        &self,
        fd: i32,
        buffer: OutMapAliasBuffer<u8>,
        vlen: i32,
        flags: ReadFlags,
        timeout: TimeSpec,
    ) -> BsdResult<()>;

    #[ipc_rid(30)]
    #[version(VersionInterval::from(Version::new(7, 0, 0)))]
    fn send_mmesg(
        &self,
        fd: i32,
        buffer: OutMapAliasBuffer<u8>,
        vlen: i32,
        flags: SendFlags,
    ) -> BsdResult<()>;
}
