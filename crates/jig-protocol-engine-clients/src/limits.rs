//! Connection and exchange admission (protocol/client.md, section 13).
use crate::{Request, connection::Connection, sign_in::Waiting};
use skein_lib::{Duration, Map, Queue, Token};
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Limits {
    pub connections: u32,
    pub identity_bytes: u32,
    pub families: u32,
    pub providers: u32,
    pub exchanges: u32,
    pub exchanges_per_client: u32,
    pub http: skein_http::server::Limits,
    pub wire: jig_protocol_client_wire::v5::Limits,
    pub link_span: Duration,
    pub stall: Duration,
}
pub const MAX_OUT_DOWN: u32 = 4;
pub const MAX_OUT_UP: u32 = 3;
pub const MAX_OUT_FIRE: u32 = 1;
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    if limits.wire.signed_in_token < 40
        || limits.wire.link_sign_in_secret < 32
        || limits.connections == 0
        || limits.exchanges == 0
        || limits.exchanges_per_client == 0
        || limits.link_span == Duration::ZERO
        || limits.stall == Duration::ZERO
    {
        return None;
    }
    jig_protocol_client_wire::v5::check_limits(&limits.wire).ok()?;
    let body = u32::try_from(limits.http.body).ok()?;
    let each = skein_http::server::worst_case(&limits.http)?
        .checked_add(skein_lib::Intake::worst_case(body)?)?
        .checked_add(u64::from(limits.http.head))?
        .checked_add(skein_lib::List::<skein_http::Header>::worst_case(limits.http.headers)?)?
        .checked_add(u64::from(limits.wire.signed_in_token).checked_add(32)?)?
        .checked_add(Queue::<skein_http::server::Event>::worst_case(8)?)?
        .checked_add(Queue::<skein_http::server::Request>::worst_case(8)?)?
        .checked_add(Queue::<skein_lib::stream::Down>::worst_case(8)?)?;
    Map::<Token, Connection>::worst_case(limits.connections)?
        .checked_add(each.checked_mul(u64::from(limits.connections))?)?
        .checked_add(Map::<Token, Waiting>::worst_case(limits.exchanges)?)?
        .checked_add(u64::from(limits.exchanges).checked_mul(80)?)?
        .checked_add(Queue::<Request>::worst_case(MAX_OUT_DOWN)?)?
        .checked_add(skein_lib::List::<crate::Served>::worst_case(limits.families)?)?
        .checked_add(skein_lib::List::<u16>::worst_case(limits.providers)?)?
        .checked_add(u64::from(limits.identity_bytes).checked_mul(u64::from(MAX_OUT_DOWN).checked_add(1)?)?)?
        .checked_add(u64::try_from(size_of::<jig_domain_core::Event>()).ok()?.checked_mul(u64::from(MAX_OUT_DOWN))?)?
        .checked_add(256)
}
