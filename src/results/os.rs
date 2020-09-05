pub const RESULT_MODULE: u32 = 1;

result_define_group!(RESULT_MODULE => {
    InvalidSize: 101,
    InvalidAddress: 102,
    InvalidHandle: 114,
    Timeout: 117,
    OperationCanceled: 118,
    SessionClosed: 123,
    UnhandledException: 124,
    FatalException: 128
});