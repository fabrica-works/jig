//! Dense holder and source order (accounts.md, 3.4 and section 5).
use crate::{Domain, Holder, Limits, Request, Stored};
use skein_lib::{Env, List, Map, Queue};

pub(crate) fn account_record(domain: &Domain, number: u32) -> Stored {
    let account = domain.accounts.get(&number).expect("a kept account");
    Stored::Account {
        number,
        provider: account.provider,
        holder: account.holder,
        place: account.place,
        uses: account.uses.clone(),
        subject: account.subject.clone(),
        display: account.display.clone(),
    }
}

pub(crate) fn accounts(domain: &Domain, holder: Holder) -> List<u32> {
    let mut order = List::with_capacity(domain.accounts.capacity());
    let mut after = None;
    for _ in 0..domain.accounts.len() {
        let mut next = None;
        for (number, account) in &domain.accounts {
            let position = (account.place, *number);
            if account.holder != holder
                || match after {
                    Some(old) => position <= old,
                    None => false,
                }
            {
                continue;
            }
            if match next {
                Some(old) => position < old,
                None => true,
            } {
                next = Some(position);
            }
        }
        let Some((place, number)) = next else { break };
        order.push(number).expect("one per account");
        after = Some((place, number));
    }
    order
}

pub(crate) fn save_accounts(domain: &mut Domain, order: &List<u32>, out: &mut Queue<Request>) {
    for (place, number) in order.iter().enumerate() {
        let place = u32::try_from(place).expect("bounded accounts");
        let entry = domain.accounts.get_mut(number).expect("a listed account");
        if entry.place != place {
            entry.place = place;
            out.push(Request::Save { record: account_record(domain, *number) });
        }
    }
}

pub(crate) fn sources(domain: &Domain, account: u32) -> List<u32> {
    let account = domain.accounts.get(&account).expect("a kept account");
    let mut order = List::with_capacity(account.sources.capacity());
    for (_, source) in &account.sources {
        order.push(*source).expect("one per source");
    }
    order
}

pub(crate) fn save_sources(
    domain: &mut Domain,
    env: &Env<Limits>,
    account: u32,
    order: &List<u32>,
    out: &mut Queue<Request>,
) {
    let entry = domain.accounts.get_mut(&account).expect("a kept account");
    let capacity = entry.sources.capacity();
    let old = core::mem::replace(&mut entry.sources, Map::with_capacity(capacity));
    for (place, number) in order.iter().enumerate() {
        let place = u32::try_from(place).expect("bounded sources");
        let places = &mut domain.accounts.get_mut(&account).expect("a kept account").sources;
        assert!(places.insert(place, *number) == Ok(None), "dense places fit");
    }
    for (place, number) in order.iter().enumerate() {
        let place = u32::try_from(place).expect("bounded sources");
        if old.get(&place) != Some(number) {
            out.push(Request::Save { record: crate::domain::stored(domain, env, *number) });
        }
    }
}

pub(crate) fn normalize(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let mut numbers = List::with_capacity(domain.accounts.capacity());
    for (number, _) in &domain.accounts {
        numbers.push(*number).expect("one per account");
    }
    for number in &numbers {
        let holder = domain.accounts.get(number).expect("a listed account").holder;
        let order = accounts(domain, holder);
        save_accounts(domain, &order, out);
        let order = sources(domain, *number);
        save_sources(domain, env, *number, &order, out);
    }
}
