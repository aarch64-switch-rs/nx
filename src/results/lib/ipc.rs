pub const RESULT_SUBMODULE: u32 = 600;

result_define_subgroup!(super::RESULT_MODULE, RESULT_SUBMODULE => {
    ObjectIdAlreadyAllocated: 1,
    DomainNotFound: 2,
    InvalidCommandType: 3,
    InvalidDomainCommandType: 4,
    SignaledServerNotFound: 5,
    CopyHandlesFull: 6,
    MoveHandlesFull: 7,
    DomainObjectsFull: 8,
    InvalidDomainObject: 9,
    PointerSizesFull: 10,
    SendStaticsFull: 11,
    ReceiveStaticsFull: 12,
    SendBuffersFull: 13,
    ReceiveBuffersFull: 14,
    ExchangeBuffersFull: 15,
    InvalidSendStaticCount: 16,
    InvalidReceiveStaticCount: 17,
    InvalidSendBufferCount: 18,
    InvalidReceiveBufferCount: 19,
    InvalidExchangeBufferCount: 20,
    InvalidBufferAttributes: 21
});