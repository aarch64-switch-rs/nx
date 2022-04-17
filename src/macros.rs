pub mod result;

pub mod util;

pub mod ipc;

pub mod diag;

pub mod rrt0;

// Note: since macros are resolved after the entire module has been resolved, to easily use them anywhere we have separate modules for them