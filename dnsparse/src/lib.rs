#![no_std]

mod frame;
pub use frame::DnsFrame;

mod header;
pub use header::{DnsHeader, HeaderKind, ResponseCode, OpCode, QueryKind, QueryClass};

mod question;
pub use question::{Question, Questions};
