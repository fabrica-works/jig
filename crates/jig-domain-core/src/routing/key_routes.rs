//! LLM keys and moved records travel by carried number only (domain/accounts.md, section 5).
use crate::{Carried, Core, Event, Held, Limits, Request, Requests};
use alloc::boxed::Box;
use jig_domain_accounts as accounts;
use jig_domain_people as people;
use skein_lib::{Env, Queue, ReplyTo, Token};

#[expect(clippy::too_many_arguments, reason = "the carried party envelope is passed unchanged")]
pub(super) fn start(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    reply_to: ReplyTo,
    sign_in: u64,
    key: [u8; 16],
    ask: people::Ask,
    carried: Carried,
) -> Requests {
    let digest = if let people::Ask::AccountChange {
        change:
            people::AccountChange::AddKey { digest, .. }
            | people::AccountChange::ReplaceKey { digest, .. }
            | people::AccountChange::FinishSignIn { digest, .. }
            | people::AccountChange::MoveIn { digest },
        ..
    } = &ask
    {
        Some(*digest)
    } else {
        None
    };
    if digest != Some(carried.digest) {
        let mut out = Queue::with_capacity(2);
        out.push(Request::Held(Box::new(Held::PeopleReply {
            to: reply_to,
            reply: people::Reply::Refused(people::Refusal::NotOffered),
        })));
        out.push(Request::Decided);
        return super::application_routes::first(Some(carried.number), Requests::Out(out));
    }
    assert!(core.key_input.is_none(), "one LLM key admission at a time");
    core.key_input = Some(carried);
    let routed = super::people_routes::route_people(
        core,
        env,
        work,
        people::Event::Ask { models: Box::new([]), reply_to, sign_in, key, ask },
    );
    let unused = super::application_routes::number(core.key_input.take());
    super::application_routes::first(unused, routed)
}

/// A carried account change's cleanup is fixed at admission (store.md, section 9).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Flight {
    pub number: u64,
    pub always_unused: bool,
}

pub(super) fn admitted(
    core: &mut Core,
    request: Token,
    holder: accounts::Holder,
    by: Option<u64>,
    change: people::AccountChange,
) -> Result<accounts::Change, people::Refusal> {
    let change = match change {
        people::AccountChange::AddKey { provider, join, .. } => {
            let carried = take(core, request, false)?;
            if let Some(account) = join {
                held(core, holder, account)?;
            }
            accounts::Change::AddKey { provider, join, carried }
        }
        people::AccountChange::FinishSignIn { source, .. } => accounts::Change::FinishSignIn {
            source,
            carried: take(core, request, true)?,
            by: by.ok_or(people::Refusal::NotOffered)?,
        },
        people::AccountChange::MoveIn { .. } => accounts::Change::MoveIn { carried: take(core, request, false)? },
        people::AccountChange::ReplaceKey { source, .. } => {
            let carried = take(core, request, false)?;
            if let Some(row) = core.accounts.source(source) {
                held(core, holder, row.account)?;
            }
            accounts::Change::ReplaceKey { source, carried }
        }
        other @ (people::AccountChange::DeclareProgram { .. }
        | people::AccountChange::CheckProgram { .. }
        | people::AccountChange::RemoveKey { .. }
        | people::AccountChange::Resume { .. }
        | people::AccountChange::Remove { .. }
        | people::AccountChange::MoveAccount { .. }
        | people::AccountChange::MoveSource { .. }
        | people::AccountChange::SetUses { .. }
        | people::AccountChange::MoveOut { .. }
        | people::AccountChange::Written { .. }) => return super::account_requests::change(core, holder, other),
    };
    Ok(change)
}
fn take(core: &mut Core, request: Token, always_unused: bool) -> Result<u64, people::Refusal> {
    let carried = core.key_input.take().ok_or(people::Refusal::NotOffered)?;
    let flight = Flight { number: carried.number, always_unused };
    assert!(core.keys.insert(request, flight) == Ok(None), "one LLM key per people flight");
    Ok(carried.number)
}

/// A finish is always unused. Only a kept key or adopted moved record consumes its value.
pub(super) fn finished(core: &mut Core, request: Token, outcome: &people::Outcome) -> Option<u64> {
    let carried = core.keys.remove(&request)?;
    if carried.always_unused {
        return Some(carried.number);
    }
    if let people::Outcome::AccountKeyKept { .. } | people::Outcome::AccountMovedIn { .. } = outcome {
        None
    } else {
        Some(carried.number)
    }
}

fn held(core: &Core, holder: accounts::Holder, account: u32) -> Result<(), people::Refusal> {
    if let Some(row) = core.accounts.account(account)
        && row.holder() != holder
    {
        return Err(people::Refusal::NotOffered);
    }
    Ok(())
}
