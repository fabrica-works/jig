//! Connection and exchange admission (protocol/client.md, section 13).
use crate::{Request, exchange::Exchange, sign_in::Waiting};
use alloc::boxed::Box;
use skein_lib::{Duration, Map, Queue, Token};
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Limits {
    pub streams: u32,
    pub watches_per_stream: u32,
    pub queued_bytes: u32,
    pub identity_bytes: u32,
    pub families: u32,
    pub providers: u32,
    pub exchanges: u32,
    pub mints: u32,
    pub exchanges_per_client: u32,
    /// A message's bytes, owned by the component (protocol/client.md, 4.2 and 13).
    pub message_bytes: u32,
    pub wire: jig_client_protocol_wire::v11::Limits,
    pub link_span: Duration,
}
pub const MAX_OUT_DOWN: u32 = 4;
pub const MAX_OUT_UP: u32 = 3;
pub const MAX_OUT_FIRE: u32 = 1;
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    if limits.wire.signed_in_token_some < 40
        || limits.wire.made_service_credential < 40
        || limits.wire.link_sign_in_secret < 32
        || limits.identity_bytes < 8
        || limits.streams == 0
        || limits.watches_per_stream == 0
        || limits.queued_bytes < 128
        || limits.mints == 0
        || limits.message_bytes == 0
        || limits.exchanges == 0
        || limits.exchanges_per_client == 0
        || limits.link_span == Duration::ZERO
    {
        return None;
    }
    jig_client_protocol_wire::v11::check_limits(&limits.wire).ok()?;
    let body = u64::from(limits.message_bytes);
    let answer = core::cmp::max(
        jig_client_protocol_wire::v11::KeyedAnswer::worst_case_bytes(&limits.wire)?,
        jig_client_protocol_wire::v11::ReadAnswer::worst_case_bytes(&limits.wire)?,
    );
    let each = u64::from(limits.wire.signed_in_token_some).checked_add(1)?.checked_add(32)?.checked_add(answer)?;
    Map::<Token, Exchange>::worst_case(limits.exchanges)?
        .checked_add(each.checked_mul(u64::from(limits.exchanges))?)?
        .checked_add(Map::<Token, Waiting>::worst_case(limits.exchanges.checked_add(limits.exchanges)?)?)?
        .checked_add(Map::<Token, Option<u64>>::worst_case(limits.exchanges.checked_add(limits.exchanges)?)?)?
        .checked_add(
            u64::from(limits.exchanges.checked_add(limits.exchanges)?)
                .checked_mul(body.checked_mul(128)?.checked_add(80)?)?,
        )?
        .checked_add(Queue::<Request>::worst_case(MAX_OUT_DOWN)?)?
        // Original names, canonicalization scratch and each routed output context.
        .checked_add(
            jig_domain_people::model_names_worst_case()?.checked_mul(u64::from(MAX_OUT_DOWN).checked_add(2)?)?,
        )?
        .checked_add(skein_lib::List::<crate::Served>::worst_case(limits.families)?)?
        .checked_add(skein_lib::List::<u16>::worst_case(limits.providers)?)?
        // The configured models, which `Component::new` holds to the core's bounds.
        .checked_add(skein_lib::List::<crate::Model>::worst_case(jig_domain_tasks::MODELS)?)?
        .checked_add(
            u64::from(jig_domain_tasks::MODELS).checked_mul(
                u64::from(jig_domain_core::MODEL_NAME_BYTES)
                    .checked_add(u64::from(jig_domain_core::MODEL_DESCRIPTION_BYTES))?
                    .checked_add(skein_lib::List::<Box<[u8]>>::worst_case(jig_domain_core::EFFORTS)?)?
                    .checked_add(
                        u64::from(jig_domain_core::EFFORTS).checked_mul(u64::from(jig_domain_core::EFFORT_BYTES))?,
                    )?,
            )?,
        )?
        .checked_add(u64::from(limits.identity_bytes).checked_mul(u64::from(MAX_OUT_DOWN).checked_add(1)?)?)?
        .checked_add(u64::try_from(size_of::<jig_domain_core::Event>()).ok()?.checked_mul(u64::from(MAX_OUT_DOWN))?)?
        // Decoded messages' enums and owned list entries are at most 64 times
        // their bounded body; reserve the current translation and its output slots.
        .checked_add(body.checked_mul(64)?.checked_mul(u64::from(MAX_OUT_DOWN).checked_add(1)?)?)?
        .checked_add(answer.checked_mul(2)?)?
        .checked_add(Map::<Token, crate::stream::Stream>::worst_case(limits.streams)?)?
        .checked_add(
            u64::from(limits.streams).checked_mul(
                Map::<Token, crate::stream::Watch>::worst_case(limits.watches_per_stream)?
                    .checked_add(u64::from(limits.queued_bytes).checked_mul(3)?)?,
            )?,
        )?
        .checked_add(Map::<Token, Token>::worst_case(limits.mints)?)?
        .checked_add(256)
}
