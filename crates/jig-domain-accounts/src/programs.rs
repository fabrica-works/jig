//! Holder declarations and lifetime names (accounts.md, 3.4 and section 5).
use crate::domain::{Account, Source};
use crate::{Domain, Holder, Kind, Limits, Refusal, Request, Stored};
use alloc::boxed::Box;
use skein_lib::{Env, List, Map, Queue};

/// A reservation outlives its source: an old directory keeps its original holder.
#[derive(PartialEq, Eq, Debug)]
pub(crate) struct Issued {
    pub(crate) holder: Holder,
    pub(crate) name: Box<[u8]>,
}

/// A declaration refuses before allocating either a number or a name.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DeclarationRefusal {
    Account(Refusal),
    Name,
    Issued,
    Names,
}

/// Declare one source, saving the account, source, name reservation and counter together.
/// The caller validates a joined account's existing payers against its new plan terms.
///
/// # Errors
/// An invalid or issued name, the lifetime-name bound, or account/provider admission.
pub fn declare(
    domain: &mut Domain,
    env: &Env<Limits>,
    holder: Holder,
    provider: u32,
    join: Option<u32>,
    name: Box<[u8]>,
    out: &mut Queue<Request>,
) -> Result<u32, DeclarationRefusal> {
    if !domain.wanted() {
        return Err(DeclarationRefusal::Account(Refusal::Closing));
    }
    if !crate::records::program_name(&name) {
        return Err(DeclarationRefusal::Name);
    }
    for (_, issued) in &domain.issued {
        if issued.name == name {
            return Err(DeclarationRefusal::Issued);
        }
    }
    if domain.issued.len() >= env.limits.program_names.min(domain.issued.capacity()) {
        return Err(DeclarationRefusal::Names);
    }
    match domain.providers.get(provider) {
        Some(offered) if !offered.offers.programs.is_empty() => {}
        Some(_) | None => return Err(DeclarationRefusal::Account(Refusal::Provider { provider })),
    }
    let admission = match join {
        Some(account) => {
            if let Err(refusal) = crate::changes::account(domain, holder, account) {
                return Err(DeclarationRefusal::Account(refusal));
            }
            let Some(entry) = domain.accounts.get(&account) else {
                return Err(DeclarationRefusal::Account(Refusal::Unknown));
            };
            if entry.provider != provider {
                return Err(DeclarationRefusal::Account(Refusal::Unknown));
            }
            domain.room_for_source(&env.limits, account)
        }
        None => domain.room_for_account(&env.limits, holder),
    };
    if let Err(refusal) = admission {
        return Err(DeclarationRefusal::Account(refusal));
    }
    let account = match join {
        Some(number) => number,
        None => {
            let number = domain.take();
            let entry = Account {
                provider,
                holder,
                subject: Box::new([]),
                display: Box::new([]),
                sources: Map::with_capacity(env.limits.sources),
                place: domain.last_place(holder),
                uses: None,
                plan: true,
            };
            assert!(domain.accounts.insert(number, entry).is_ok(), "admitted account");
            out.push(Request::Save { record: crate::ordering::account_record(domain, number) });
            number
        }
    };
    let number = domain.take();
    let entry = domain.accounts.get_mut(&account).expect("admitted account remains kept");
    let place = entry.sources.len();
    assert!(entry.sources.insert(place, number) == Ok(None), "admitted source at the last place");
    entry.plan = true;
    let issued = Issued { holder, name: name.clone() };
    assert!(domain.issued.insert(number, issued).is_ok(), "admitted lifetime name");
    out.push(Request::Save { record: Stored::Issued { number, holder, name: name.clone() } });
    let source = Source::new(account, Kind::Program, name, &env.limits);
    assert!(domain.sources.insert(number, source) == Ok(None), "admitted source");
    out.push(Request::Save { record: crate::domain::stored(domain, env, number) });
    domain.counted(out);
    Ok(number)
}

pub(crate) fn restore(domain: &mut Domain, number: u32, holder: Holder, name: Box<[u8]>) -> bool {
    if number == u32::MAX || domain.accounts.contains_key(&number) || !crate::records::program_name(&name) {
        return false;
    }
    for (other_number, issued) in &domain.issued {
        if *other_number == number || issued.name == name {
            return false;
        }
    }
    domain.issued.insert(number, Issued { holder, name }).is_ok()
}

pub(crate) fn finish_restore(domain: &mut Domain, out: &mut Queue<Request>) {
    // A live source predating the name registry still owns its directory. Reserve it
    // before admitting changes, rather than making its old name available to a new holder.
    let mut missing = List::with_capacity(domain.sources.capacity());
    for (number, source) in &domain.sources {
        if source.kind == Kind::Program && !domain.issued.contains_key(number) {
            missing.push(*number).expect("one reservation per live source");
        }
    }
    for number in missing.into_boxed() {
        let Some(source) = domain.sources.get(&number) else { continue };
        let Some(account) = domain.accounts.get(&source.account) else { continue };
        let holder = account.holder;
        let name = source.name.clone();
        if restore(domain, number, holder, name.clone()) {
            out.push(Request::Save { record: Stored::Issued { number, holder, name } });
        }
    }
    let mut rejected = List::with_capacity(domain.sources.capacity());
    for (number, source) in &domain.sources {
        let valid = match source.kind {
            Kind::Program => match domain.issued.get(number) {
                Some(issued) => match domain.accounts.get(&source.account) {
                    Some(account) => issued.holder == account.holder && issued.name == source.name,
                    None => false,
                },
                None => false,
            },
            Kind::SignIn | Kind::Key | Kind::None => !domain.issued.contains_key(number),
        };
        if !valid {
            rejected.push(*number).expect("at most one rejected row per source");
        }
    }
    for number in &rejected {
        let Some(source) = domain.sources.remove(number) else { continue };
        if let Some(account) = domain.accounts.get_mut(&source.account) {
            let mut at = None;
            for (place, kept) in &account.sources {
                if kept == number {
                    at = Some(*place);
                }
            }
            if let Some(place) = at {
                account.sources.remove(&place);
            }
        }
        domain.alarms.cancel(*number);
        domain.unrestored = domain.unrestored.saturating_add(1);
    }
    let mut numbers = List::with_capacity(domain.accounts.capacity());
    for (number, _) in &domain.accounts {
        numbers.push(*number).expect("one number per account");
    }
    for number in &numbers {
        let Some(account) = domain.accounts.get_mut(number) else { continue };
        let mut plan = false;
        for (_, number) in &account.sources {
            if let Some(source) = domain.sources.get(number)
                && let Some(provider) = domain.providers.get(account.provider)
            {
                plan |= provider.plan(source.kind);
            }
        }
        account.plan = plan;
    }
}
