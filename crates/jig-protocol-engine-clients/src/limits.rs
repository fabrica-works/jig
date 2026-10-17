//! Connection and exchange admission (protocol/client.md, section 13).
use crate::{Request, connection::Connection, sign_in::Waiting};
use skein_lib::{Duration, Map, Queue, Token};
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Limits {
    pub connections: u32,
    pub streams: u32,
    pub watches_per_stream: u32,
    pub queued_bytes: u32,
    pub identity_bytes: u32,
    pub families: u32,
    pub providers: u32,
    pub exchanges: u32,
    pub exchanges_per_client: u32,
    pub http: skein_http::server::Limits,
    pub wire: jig_client_protocol_wire::v8::Limits,
    pub link_span: Duration,
    pub stall: Duration,
}
pub const MAX_OUT_DOWN: u32 = 4;
pub const MAX_OUT_UP: u32 = 3;
pub const MAX_OUT_FIRE: u32 = 1;
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    if limits.wire.signed_in_token < 40
        || limits.wire.made_service_credential < 40
        || limits.wire.link_sign_in_secret < 32
        || limits.identity_bytes < 8
        || limits.connections == 0
        || limits.streams == 0
        || limits.watches_per_stream == 0
        || limits.queued_bytes < 128
        || limits.queued_bytes.checked_add(32).is_none()
        || limits.exchanges == 0
        || limits.exchanges_per_client == 0
        || limits.link_span == Duration::ZERO
        || limits.stall == Duration::ZERO
    {
        return None;
    }
    jig_client_protocol_wire::v8::check_limits(&limits.wire).ok()?;
    let body = u32::try_from(limits.http.body).ok()?;
    let answer = core::cmp::max(
        jig_client_protocol_wire::v8::Answer::worst_case_bytes(&limits.wire)?,
        jig_client_protocol_wire::v8::ReadAnswer::worst_case_bytes(&limits.wire)?,
    );
    let each = skein_http::server::worst_case(&limits.http)?
        .checked_add(u64::from(limits.http.read))?
        .checked_add(skein_lib::Intake::worst_case(body)?)?
        .checked_add(u64::from(limits.http.head))?
        .checked_add(skein_lib::List::<skein_http::Header>::worst_case(limits.http.headers)?)?
        .checked_add(u64::from(limits.wire.signed_in_token).checked_add(32)?)?
        .checked_add(answer)?
        .checked_add(Queue::<skein_http::server::Event>::worst_case(8)?)?
        .checked_add(Queue::<skein_http::server::Request>::worst_case(8)?)?
        .checked_add(Queue::<skein_lib::stream::Down>::worst_case(8)?)?;
    Map::<Token, Connection>::worst_case(limits.connections)?
        .checked_add(each.checked_mul(u64::from(limits.connections))?)?
        .checked_add(Map::<Token, Waiting>::worst_case(limits.exchanges.checked_add(limits.connections)?)?)?
        .checked_add(
            u64::from(limits.exchanges.checked_add(limits.connections)?)
                .checked_mul(u64::from(body).checked_mul(128)?.checked_add(80)?)?,
        )?
        .checked_add(Queue::<Request>::worst_case(MAX_OUT_DOWN)?)?
        .checked_add(skein_lib::List::<crate::Served>::worst_case(limits.families)?)?
        .checked_add(skein_lib::List::<u16>::worst_case(limits.providers)?)?
        .checked_add(u64::from(limits.identity_bytes).checked_mul(u64::from(MAX_OUT_DOWN).checked_add(1)?)?)?
        .checked_add(u64::try_from(size_of::<jig_domain_core::Event>()).ok()?.checked_mul(u64::from(MAX_OUT_DOWN))?)?
        // A decoded ask's enums and owned list entries are at most 64 times
        // their wire body; reserve the current translation and its output slots.
        .checked_add(u64::from(body).checked_mul(64)?.checked_mul(u64::from(MAX_OUT_DOWN).checked_add(1)?)?)?
        .checked_add(answer.checked_mul(2)?)?
        .checked_add(Map::<Token, crate::stream::Stream>::worst_case(limits.streams)?)?
        .checked_add(
            u64::from(limits.streams).checked_mul(
                Map::<Token, crate::stream::Watch>::worst_case(limits.watches_per_stream)?
                    .checked_add(u64::from(limits.queued_bytes).checked_mul(3)?)?
                    .checked_add(Queue::<skein_http::sse::writer::Event>::worst_case(2)?)?
                    .checked_add(Queue::<skein_lib::stream::Down>::worst_case(2)?)?,
            )?,
        )?
        .checked_add(256)
}
