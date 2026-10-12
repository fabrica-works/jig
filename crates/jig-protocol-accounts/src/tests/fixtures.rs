use skein_lib::{Duration, Secret, Time, bytes};
use skein_oauth_accounts as driver;
pub const ACCOUNT_CHAT: u32 = 0;
pub const ACCOUNT_DELEGATE: u32 = 1;
pub const TOKEN_FIRST: &[u8] = b"token-first";
pub const TOKEN_SECOND: &[u8] = b"token-second";
pub const NOW: Time = Time::from_nanos(5);
pub fn granted(account: u32, token: &[u8], generation: u64, valid: Duration) -> driver::Event {
    driver::Event::Granted { account, token: bytes::copy_of(token), generation, valid }
}

pub fn value(token: &[u8]) -> crate::table::Value {
    crate::table::Value { token: Secret::new(bytes::copy_of(token)), account_id: Secret::new(bytes::copy_of(b"")) }
}
