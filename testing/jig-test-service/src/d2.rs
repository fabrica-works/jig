//! D2: removed by JP13. Explicit setup for the retained handed-in live fixture only.
use alloc::boxed::Box;
use jig_domain_accounts::{Holder, Kind, Stored};
use jig_protocol_accounts::handed_in;
use skein_lib::{Map, Secret, Wall};
use skein_oauth as oauth;

/// D2: removed by JP13. Not configuration, and never inferred from stored records.
pub struct D2 {
    pub(crate) sources: Map<u32, oauth::SavedToken>,
}
impl core::fmt::Debug for D2 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("D2").field("sources", &self.sources.len()).finish()
    }
}
impl D2 {
    #[must_use]
    pub fn empty() -> Self {
        Self { sources: Map::with_capacity(0) }
    }
    pub(crate) fn worst_case(&self, limits: &oauth::Limits) -> Option<u64> {
        Map::<u32, oauth::SavedToken>::worst_case(self.sources.capacity())?
            .checked_add(u64::from(self.sources.capacity()).checked_mul(u64::from(limits.record_bytes))?)
    }
}
/// D2: removed by JP13. The helper's explicit rows and their secret, to preseed before startup.
#[derive(Debug)]
pub struct D2Preseed {
    pub rows: Box<[Stored]>,
    pub value: Secret,
    pub input: D2,
}
/// D2: removed by JP13. The only builder of nonempty D2 input; no import or startup fallback.
#[expect(clippy::too_many_arguments, reason = "the explicit fixture names every identity and credential")]
pub fn d2_preseed(
    account: u32,
    source: u32,
    provider: u32,
    holder: Holder,
    token: Secret,
    lapses: Wall,
    account_id: Option<Box<[u8]>>,
    limits: &oauth::Limits,
) -> Result<D2Preseed, handed_in::Unfit> {
    if account == source {
        return Err(handed_in::Unfit);
    }
    let next = account.max(source).checked_add(1).ok_or(handed_in::Unfit)?;
    let record = handed_in::record(source, token, lapses, account_id, limits)?;
    let value = oauth::encode_record(&record, limits).or(Err(handed_in::Unfit))?;
    let mut sources = Map::with_capacity(1);
    assert!(sources.insert(source, record).is_ok(), "one explicit D2 source");
    Ok(D2Preseed {
        rows: Box::new([
            Stored::Account {
                number: account,
                provider,
                holder,
                place: 0,
                uses: None,
                subject: Box::new([]),
                display: Box::new([]),
            },
            Stored::Source { account, number: source, place: 0, kind: Kind::SignIn, name: Box::new([]), held: None },
            Stored::Next { number: next },
        ]),
        value: Secret::new(value),
        input: D2 { sources },
    })
}
