//! Holder-owned account changes (accounts.md, section 5).
use crate::{Answer, Bound, Change, Domain, Holder, Key, Kind, Limits, Refusal, Request, Signing};
use skein_lib::{Env, List, Queue, Token};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum Leaving {
    Source(Token),
    Account(u32),
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct Removing {
    pub(crate) request: Token,
    pub(crate) left: u32,
    pub(crate) confirmed: bool,
}

pub(crate) fn account(domain: &Domain, holder: Holder, number: u32) -> Result<(), Refusal> {
    if !domain.wanted() {
        return Err(Refusal::Closing);
    }
    let Some(account) = domain.accounts.get(&number) else { return Err(Refusal::Unknown) };
    if account.holder != holder {
        return Err(Refusal::Unknown);
    }
    Ok(())
}
pub(crate) fn source(domain: &Domain, holder: Holder, number: u32) -> Result<u32, Refusal> {
    if !domain.wanted() {
        return Err(Refusal::Closing);
    }
    let Some(source) = domain.sources.get(&number) else { return Err(Refusal::Unknown) };
    account(domain, holder, source.account)?;
    Ok(source.account)
}

pub(crate) fn change(
    domain: &mut Domain,
    env: &Env<Limits>,
    request: Token,
    holder: Holder,
    change: Change,
    out: &mut Queue<Request>,
) {
    // Every request refuses Closing before inspecting any other field.
    if !domain.wanted() {
        refuse(request, Refusal::Closing, out);
        return;
    }
    let result = match change {
        Change::AddKey { provider, join, carried } => {
            crate::keys::add(domain, env, request, holder, provider, join, carried, out);
            return;
        }
        Change::ReplaceKey { source, carried } => {
            crate::keys::replace(domain, env, request, holder, source, carried, out);
            return;
        }
        Change::RemoveKey { source } => {
            crate::keys::remove(domain, env, request, holder, source, out);
            return;
        }
        Change::Resume { source } => resume(domain, env, holder, source, out),
        Change::Remove { account } => {
            remove(domain, request, holder, account, out);
            return;
        }
        Change::MoveAccount { account: number, place } => move_account(domain, holder, number, place, out),
        Change::MoveSource { account: number, source, place } => {
            move_source(domain, env, holder, number, source, place, out)
        }
        Change::SetUses { account: number, uses } => match account(domain, holder, number) {
            Err(refusal) => Err(refusal),
            Ok(()) => {
                if crate::records::valid_uses(uses.as_deref()) {
                    domain.accounts.get_mut(&number).expect("checked account").uses = uses;
                    out.push(Request::Save { record: crate::ordering::account_record(domain, number) });
                    Ok(())
                } else {
                    Err(Refusal::Uses)
                }
            }
        },
    };
    match result {
        Ok(()) => out.push(Request::Answer { request, answer: Answer::Done }),
        Err(refusal) => refuse(request, refusal, out),
    }
}

fn resume(
    domain: &mut Domain,
    env: &Env<Limits>,
    holder: Holder,
    number: u32,
    out: &mut Queue<Request>,
) -> Result<(), Refusal> {
    let Some(entry) = domain.sources.get(&number) else { return Err(Refusal::Unknown) };
    if domain.accounts.get(&entry.account).expect("a kept account").holder != holder {
        return Err(Refusal::Unknown);
    }
    let entry = domain.sources.get_mut(&number).expect("checked source");
    let before = entry.hold;
    match before {
        Some(crate::domain::Hold::Limited { .. }) => {
            entry.hold = None;
            entry.hold_attention = false;
        }
        Some(crate::domain::Hold::Refused) | None => return Err(Refusal::NotLimited),
    }
    crate::domain::changed(domain, env, number, before, out);
    Ok(())
}

/// Remove one source, erasing its account with the last source and saving dense shifts.
pub(crate) fn detach(domain: &mut Domain, env: &Env<Limits>, account: u32, number: u32, out: &mut Queue<Request>) {
    drop(domain.sources.remove(&number).expect("a kept source"));
    domain.alarms.cancel(number);
    let entry = domain.accounts.get_mut(&account).expect("a source's account");
    let mut position = None;
    for (place, source) in &entry.sources {
        if *source == number {
            position = Some(*place);
        }
    }
    entry.sources.remove(&position.expect("a source's place")).expect("a kept place");
    out.push(Request::Erase { key: Key::Source { account, number } });
    if entry.sources.is_empty() {
        let holder = entry.holder;
        drop(domain.accounts.remove(&account).expect("a kept account"));
        out.push(Request::Erase { key: Key::Account { number: account } });
        let order = crate::ordering::accounts(domain, holder);
        crate::ordering::save_accounts(domain, &order, out);
    } else {
        let mut plan = false;
        for (_, source) in &entry.sources {
            let kind = domain.sources.get(source).expect("a kept source").kind;
            plan |= kind == Kind::SignIn || kind == Kind::Program;
        }
        entry.plan = plan;
        let order = crate::ordering::sources(domain, account);
        crate::ordering::save_sources(domain, env, account, &order, out);
    }
}

fn removable(domain: &Domain, holder: Holder, number: u32) -> Result<u32, Refusal> {
    account(domain, holder, number)?;
    if domain.removing.len() == domain.removing.capacity() {
        return Err(Refusal::Limit { bound: Bound::Accounts });
    }
    for (_, keeping) in &domain.keeping {
        if keeping.join == Some(number) {
            return Err(Refusal::Limit { bound: Bound::SignIns });
        }
    }
    let entry = domain.accounts.get(&number).expect("checked account");
    let mut leaving: u32 = 0;
    for (_, source) in &entry.sources {
        match domain.sources.get(source).expect("a kept source").kind {
            Kind::SignIn => leaving = leaving.saturating_add(1),
            Kind::Key => {}
            Kind::Program | Kind::None => {
                return Err(Refusal::Kind { kind: domain.sources.get(source).expect("a kept source").kind });
            }
        }
    }
    if leaving > domain.signing_out.capacity().saturating_sub(domain.signing_out.len()) {
        return Err(Refusal::Limit { bound: Bound::Sources });
    }
    Ok(leaving)
}
fn remove(domain: &mut Domain, request: Token, holder: Holder, number: u32, out: &mut Queue<Request>) {
    let left = match removable(domain, holder, number) {
        Ok(left) => left,
        Err(refusal) => {
            refuse(request, refusal, out);
            return;
        }
    };
    let sources = crate::ordering::sources(domain, number);
    // Remove the account once, then its rows; no transient renumbering of sources being erased.
    drop(domain.accounts.remove(&number).expect("checked account"));
    out.push(Request::Erase { key: Key::Account { number } });
    for source in &sources {
        let entry = domain.sources.remove(source).expect("an account's source");
        domain.alarms.cancel(*source);
        out.push(Request::Erase { key: Key::Source { account: number, number: *source } });
        match entry.kind {
            Kind::SignIn => {
                assert!(
                    domain.signing_out.insert(*source, Leaving::Account(number)) == Ok(None),
                    "sign-out room checked"
                );
                out.push(Request::Signing(Signing::SignOut { source: *source }));
            }
            Kind::Key => out.push(Request::Signing(Signing::Forget { source: *source })),
            Kind::Program | Kind::None => unreachable!("removal checked each source's kind"),
        }
    }
    let order = crate::ordering::accounts(domain, holder);
    crate::ordering::save_accounts(domain, &order, out);
    if left == 0 {
        out.push(Request::Answer { request, answer: Answer::Removed { confirmed: true } });
    } else {
        assert!(
            domain.removing.insert(number, Removing { request, left, confirmed: true }) == Ok(None),
            "removal room checked"
        );
    }
    // The counter stays kept; an erased account's number is never given again.
    domain.counted(out);
}

pub(crate) fn signed_out(domain: &mut Domain, account: u32, confirmed: bool, out: &mut Queue<Request>) {
    let entry = domain.removing.get_mut(&account).expect("an account removal names its pending request");
    entry.confirmed &= confirmed;
    entry.left = entry.left.checked_sub(1).expect("one answer per signed-in source");
    if entry.left == 0 {
        let removing = domain.removing.remove(&account).expect("a completed removal");
        out.push(Request::Answer {
            request: removing.request,
            answer: Answer::Removed { confirmed: removing.confirmed },
        });
    }
}
fn moved(order: &List<u32>, number: u32, place: u32) -> List<u32> {
    let mut result = List::with_capacity(order.capacity());
    let place = place.min(order.len().saturating_sub(1));
    for other in order {
        if *other == number {
            continue;
        }
        if result.len() == place {
            result.push(number).expect("one entry moved");
        }
        result.push(*other).expect("same entries");
    }
    if result.len() == place {
        result.push(number).expect("one entry moved");
    }
    result
}
fn move_account(
    domain: &mut Domain,
    holder: Holder,
    number: u32,
    place: u32,
    out: &mut Queue<Request>,
) -> Result<(), Refusal> {
    account(domain, holder, number)?;
    let order = crate::ordering::accounts(domain, holder);
    let order = moved(&order, number, place);
    crate::ordering::save_accounts(domain, &order, out);
    Ok(())
}
fn move_source(
    domain: &mut Domain,
    env: &Env<Limits>,
    holder: Holder,
    number: u32,
    source: u32,
    place: u32,
    out: &mut Queue<Request>,
) -> Result<(), Refusal> {
    account(domain, holder, number)?;
    if self::source(domain, holder, source)? != number {
        return Err(Refusal::Unknown);
    }
    let order = crate::ordering::sources(domain, number);
    let order = moved(&order, source, place);
    crate::ordering::save_sources(domain, env, number, &order, out);
    Ok(())
}
pub(crate) fn refuse(request: Token, refusal: Refusal, out: &mut Queue<Request>) {
    out.push(Request::Answer { request, answer: Answer::Refused { refusal } });
}
