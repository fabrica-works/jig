//! Access-only records supplied at startup (protocol/llm.md, 5.4).
use alloc::boxed::Box;
use skein_lib::{Secret, Wall, bytes};
use skein_oauth as oauth;
use skein_oauth_accounts as driver;

/// A record that cannot fit or be encoded under the driver's limits.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Unfit;

/// Checks a parsed access-only record before the owner's first Want.
pub fn hand_in(
    account: u32,
    token: Secret,
    lapses: Wall,
    account_id: Option<Box<[u8]>>,
    limits: &oauth::Limits,
) -> Result<driver::Request, Unfit> {
    let record = oauth::SavedToken {
        key: account,
        generation: 1,
        access_token: bytes::copy_of(token.expose()),
        refresh_token: None,
        metadata: account_id,
        expires_at: lapses,
    };
    match oauth::encode_record(&record, limits) {
        Ok(encoded) => drop(encoded),
        Err(_) => return Err(Unfit),
    }
    Ok(driver::Request::HandIn { account, record })
}
