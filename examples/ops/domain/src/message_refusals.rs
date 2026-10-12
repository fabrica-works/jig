//! Translation of the hub's message metadata (hosts.md, 6.4).
use alloc::boxed::Box;
use jig_domain_host::{MessageRefusal, Refused};
use skein_lib::{List, Token};
pub(crate) fn message_read(read: Option<Token>) -> Option<u64> {
    let name = read?;
    Some(name.raw())
}
pub(crate) fn refusals(refused: Box<[Refused]>) -> Box<[(u64, MessageRefusal)]> {
    let capacity = u32::try_from(refused.len()).expect("hub bounded refusals");
    let mut out = List::with_capacity(capacity);
    for refused in refused {
        out.push((refused.name.raw(), refused.reason)).expect("reserved rows");
    }
    out.into_boxed()
}
pub(crate) fn refusal_names(refused: &[(u64, MessageRefusal)]) -> Box<[u64]> {
    let capacity = u32::try_from(refused.len()).expect("hub bounded refusals");
    let mut out = List::with_capacity(capacity);
    for (name, _) in refused {
        out.push(*name).expect("reserved rows");
    }
    out.into_boxed()
}
