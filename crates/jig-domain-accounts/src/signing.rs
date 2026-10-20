//! A party's sign-in and sign-out (accounts.md, 3.3 and section 5; protocol/llm.md, 5.2).
//!
//! A sign-in takes its source's number as it starts, so that the component's record is named by
//! it from the first; the counter is saved then. The component's answer decides: a new subject
//! makes an account, the holder's own renews it with this source in its signed-in source's
//! place, and anything else is refused and the new record revoked. Plan usage declined keeps
//! the sign-in and refuses the source. A sign-out removes the source at once, and its account
//! with its last source, since the store keeps no account without one (`restore::finish`); the
//! party is answered when the component says whether the provider confirmed the revocation.
use crate::checks::room_for_sign_in;
use crate::domain::{Account, Hold, Source, changed, stored};
use crate::records::within;
use crate::{
    Answer, Bound, DISPLAY, Domain, End, Ended, Failure, Holder, Key, Kind, Limits, Refusal, Request, SignIn, Signing,
    ToComponent,
};
use alloc::boxed::Box;
use skein_lib::{Env, Map, Queue, Token};

/// A sign-in under way: whose request, for which holder, at which provider.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct UnderWay {
    request: Token,
    holder: Holder,
    provider: u32,
}

/// Started, within the sign-ins' bound, at a provider offering a sign-in; answered at once.
pub(crate) fn sign_in(
    domain: &mut Domain,
    env: &Env<Limits>,
    request: Token,
    holder: Holder,
    provider: u32,
    out: &mut Queue<Request>,
) {
    if let Err(refusal) = startable(domain, &env.limits, provider) {
        out.push(Request::Answer { request, answer: Answer::Refused { refusal } });
        return;
    }
    let source = domain.take();
    domain.counted(out);
    let entry = UnderWay { request, holder, provider };
    assert!(domain.signing.insert(source, entry) == Ok(None), "a new number, within the sign-ins' bound");
    out.push(Request::Signing(Signing::SignIn { source, provider }));
    out.push(Request::Answer { request, answer: Answer::Started { source } });
}
fn startable(domain: &Domain, limits: &Limits, provider: u32) -> Result<(), Refusal> {
    if !domain.wanted() {
        return Err(Refusal::Closing);
    }
    match domain.providers.get(provider) {
        Some(offered) if offered.offers.sign_in => {}
        Some(_) | None => return Err(Refusal::Provider { provider }),
    }
    room_for_sign_in(limits, domain.signing.len())?;
    if domain.numbers_left() < 1 {
        return Err(Refusal::Numbers);
    }
    Ok(())
}

/// The component's sign-in decides what the subject does (accounts.md, 3.3).
pub(crate) fn signed_in(
    domain: &mut Domain,
    env: &Env<Limits>,
    source: u32,
    subject: Box<[u8]>,
    display: Box<[u8]>,
    declined: bool,
    out: &mut Queue<Request>,
) {
    let Some(under_way) = domain.signing.remove(&source) else { return };
    let decided = match domain.sign_in(&env.limits, under_way.holder, under_way.provider, &subject) {
        Ok(SignIn::New) => Ok(None),
        Ok(SignIn::Renew { account }) => match place(domain, account) {
            Some(place) => Ok(Some((account, place))),
            None => Err(Refusal::Limit { bound: Bound::Sources }),
        },
        Err(refusal) => Err(refusal),
    };
    let end = match decided {
        Ok(None) => {
            let made = Made { holder: under_way.holder, provider: under_way.provider, subject, display };
            End::Made { account: make(domain, env, source, made, declined, out), declined }
        }
        Ok(Some((account, place))) => {
            renew(domain, env, account, place, source, declined, out);
            End::Renewed { account, declined }
        }
        Err(refusal) => {
            out.push(Request::Signing(Signing::SignOut { source }));
            End::Refused { refusal }
        }
    };
    domain.end(Ended { request: under_way.request, source, end });
}

/// The component's sign-in failed or was cancelled: it ends, with nothing made.
pub(crate) fn failed(domain: &mut Domain, source: u32, failure: Failure) {
    let Some(under_way) = domain.signing.remove(&source) else { return };
    domain.end(Ended { request: under_way.request, source, end: End::Failed { failure } });
}

/// Where a renewal's source goes: its account's signed-in source's place, or, with none, the
/// place after its last; none when the places are spent.
fn place(domain: &Domain, account: u32) -> Option<u32> {
    let entry = domain.accounts.get(&account)?;
    for (at, number) in &entry.sources {
        if domain.sources.get(number).expect("an account's sources are kept").kind == Kind::SignIn {
            return Some(*at);
        }
    }
    match entry.sources.last() {
        Some((last, _)) => last.checked_add(1),
        None => Some(0),
    }
}

struct Made {
    holder: Holder,
    provider: u32,
    subject: Box<[u8]>,
    display: Box<[u8]>,
}

/// A new account, with the provider's display name where it fits, and the source in its first
/// place; the account's number is taken now, and the counter saved.
fn make(
    domain: &mut Domain,
    env: &Env<Limits>,
    source: u32,
    made: Made,
    declined: bool,
    out: &mut Queue<Request>,
) -> u32 {
    let account = domain.take();
    domain.counted(out);
    let display = if within(&made.display, DISPLAY) { made.display } else { Box::new([]) };
    out.push(Request::Save {
        record: crate::Stored::Account {
            number: account,
            provider: made.provider,
            holder: made.holder,
            subject: made.subject.clone(),
            display: display.clone(),
        },
    });
    let entry = Account {
        provider: Some(made.provider),
        holder: made.holder,
        subject: made.subject,
        display,
        sources: Map::with_capacity(env.limits.sources),
    };
    let fresh = match domain.accounts.insert(account, entry) {
        Ok(old) => old.is_none(),
        Err(_) => false,
    };
    assert!(fresh, "a new number, with room checked");
    join(domain, env, account, 0, source, declined, out);
    account
}

/// The holder's account of the subject renewed: this source takes its signed-in source's
/// place, which is erased and replaced at the component.
fn renew(
    domain: &mut Domain,
    env: &Env<Limits>,
    account: u32,
    place: u32,
    source: u32,
    declined: bool,
    out: &mut Queue<Request>,
) {
    let entry = domain.accounts.get(&account).expect("a renewed account is kept");
    if let Some(old) = entry.sources.get(&place).copied() {
        drop(domain.sources.remove(&old).expect("an account's sources are kept"));
        domain.alarms.cancel(old);
        out.push(Request::Erase { key: Key::Source { account, number: old } });
        out.push(Request::Signing(Signing::Replaced { source: old, by: source }));
    }
    join(domain, env, account, place, source, declined, out);
}

/// The source joins its account at its place, is saved, and is wanted; refused when plan usage
/// was declined, until its holder acts (accounts.md, section 5).
fn join(
    domain: &mut Domain,
    env: &Env<Limits>,
    account: u32,
    place: u32,
    source: u32,
    declined: bool,
    out: &mut Queue<Request>,
) {
    let mut entry = Source::new(account, Kind::SignIn, Box::new([]), &env.limits);
    if declined {
        entry.hold = Some(Hold::Refused);
        entry.hold_attention = true;
    }
    let hold = entry.hold;
    assert!(domain.sources.insert(source, entry) == Ok(None), "a new number, within the sources' bound");
    let places = &mut domain.accounts.get_mut(&account).expect("a joined account is kept").sources;
    assert!(places.insert(place, source).is_ok(), "a place checked within the sources' bound");
    out.push(Request::Save { record: stored(domain, env, source) });
    out.push(Request::Component(ToComponent::Want { account: source }));
    changed(domain, env, source, hold, out);
}

/// A signed-in source of the holder's is removed, and its account with its last source; the
/// component revokes its record, and the party is answered when it says so.
pub(crate) fn sign_out(domain: &mut Domain, request: Token, holder: Holder, source: u32, out: &mut Queue<Request>) {
    let account = match removable(domain, holder, source) {
        Ok(account) => account,
        Err(refusal) => {
            out.push(Request::Answer { request, answer: Answer::Refused { refusal } });
            return;
        }
    };
    drop(domain.sources.remove(&source).expect("checked above"));
    domain.alarms.cancel(source);
    let entry = domain.accounts.get_mut(&account).expect("checked above");
    let mut at = None;
    for (place, number) in &entry.sources {
        if *number == source {
            at = Some(*place);
        }
    }
    entry.sources.remove(&at.expect("a source has its place")).expect("its place is kept");
    out.push(Request::Erase { key: Key::Source { account, number: source } });
    if entry.sources.is_empty() {
        drop(domain.accounts.remove(&account).expect("checked above"));
        out.push(Request::Erase { key: Key::Account { number: account } });
    }
    assert!(domain.signing_out.insert(source, request) == Ok(None), "room checked");
    out.push(Request::Signing(Signing::SignOut { source }));
}
fn removable(domain: &Domain, holder: Holder, source: u32) -> Result<u32, Refusal> {
    if !domain.wanted() {
        return Err(Refusal::Closing);
    }
    let Some(entry) = domain.sources.get(&source) else { return Err(Refusal::Unknown) };
    let account = domain.accounts.get(&entry.account).expect("a source's account is kept");
    if account.holder != holder {
        return Err(Refusal::Unknown);
    }
    if entry.kind != Kind::SignIn {
        return Err(Refusal::Kind { kind: entry.kind });
    }
    if domain.signing_out.len() >= domain.signing_out.capacity() {
        return Err(Refusal::Limit { bound: Bound::Sources });
    }
    Ok(entry.account)
}

/// The component revoked and forgot a source's record: a party's sign-out is answered.
pub(crate) fn signed_out(domain: &mut Domain, source: u32, confirmed: bool, out: &mut Queue<Request>) {
    if let Some(request) = domain.signing_out.remove(&source) {
        out.push(Request::Answer { request, answer: Answer::SignedOut { confirmed } });
    }
}
