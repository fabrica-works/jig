//! Private values kept before their owners use them (store.md, section 9).
use crate::{Below, Event, Limits, Phase, SecretOwner, SecretRow, Secrets, Store, directory, keys, open};
use alloc::boxed::Box;
use skein_lib::{Env, List, Map, Queue, Secret, Token};

#[derive(Debug)]
enum Pending {
    Keep(Token),
    Erase(Token),
    Sweep,
}
#[derive(Debug)]
pub(crate) struct State {
    pending: Map<u64, Pending>,
    next: u64,
    sweeping: bool,
    swept: bool,
    after: Option<Box<[u8]>>,
    waiting: bool,
}
impl State {
    pub fn new(capacity: u32) -> State {
        State {
            pending: Map::with_capacity(capacity),
            next: 1,
            sweeping: false,
            swept: false,
            after: None,
            waiting: false,
        }
    }
}
pub(crate) fn worst_case(limits: &Limits) -> Option<u64> {
    Map::<u64, Pending>::worst_case(limits.secrets.queued)?
        .checked_add(u64::from(limits.secrets.key))?
        .checked_add(List::<SecretRow>::worst_case(limits.secrets.page.rows)?)?
        .checked_add(List::<skein_kv::Row>::worst_case(limits.secrets.page.rows)?)?
        .checked_add(List::<skein_kv::Op>::worst_case(limits.secrets.ops)?)?
        .checked_add(u64::from(limits.secrets.value).checked_mul(2)?)?
        .checked_add(32)?
        .checked_add(u64::from(limits.secrets.page.bytes).checked_mul(2)?)
}
fn prefix(owner: SecretOwner) -> [u8; 2] {
    match owner {
        SecretOwner::SignIn => [1, 0],
        SecretOwner::Service => [1, 1],
        SecretOwner::Application => [2, 0],
    }
}
fn key(owner: SecretOwner, number: u64, limit: u32) -> Option<Box<[u8]>> {
    let [tag, owner] = prefix(owner);
    let mut key = skein_kv::KeyWriter::new(limit);
    key.tag(tag).tag(owner).u64(number);
    key.finish()
}
fn read_key(key: &[u8]) -> Option<(SecretOwner, u64)> {
    let mut reader = skein_kv::KeyReader::new(key);
    let tag = reader.tag()?;
    let kind = reader.tag()?;
    let owner = match tag {
        1 => match kind {
            0 => SecretOwner::SignIn,
            1 => SecretOwner::Service,
            _ => return None,
        },
        2 => {
            if kind != 0 {
                return None;
            }
            SecretOwner::Application
        }
        _ => return None,
    };
    let number = reader.u64()?;
    if !reader.is_empty() {
        return None;
    }
    Some((owner, number))
}
struct Loaded {
    rows: Box<[skein_kv::Row]>,
    next: Option<Box<[u8]>>,
}
fn load(store: &mut Store, env: &Env<Limits>, range: skein_kv::Range, max: skein_kv::Page) -> Option<Loaded> {
    let kv_env = directory::env(store, env, true);
    let mut above = Queue::with_capacity(skein_kv::MAX_OUT.above);
    let mut below = Queue::with_capacity(skein_kv::MAX_OUT.below);
    skein_kv::down(
        &mut store.stores.secrets,
        &kv_env,
        skein_kv::Request::Load { owner: Token::new(0), range, max },
        &mut above,
        &mut below,
    );
    assert!(below.is_empty(), "a secrets page issues no files");
    match above.pop()? {
        skein_kv::Event::Loaded { rows, next, .. } => Some(Loaded { rows, next }),
        skein_kv::Event::Opened { .. }
        | skein_kv::Event::Committed { .. }
        | skein_kv::Event::Refused { .. }
        | skein_kv::Event::Failed { .. }
        | skein_kv::Event::Got { .. }
        | skein_kv::Event::Closed { .. }
        | skein_kv::Event::OpenFailed { .. }
        | skein_kv::Event::Recovered { .. }
        | skein_kv::Event::RecoveryFailed { .. } => None,
    }
}
fn submit(
    store: &mut Store,
    env: &Env<Limits>,
    ops: Box<[skein_kv::Op]>,
    pending: Pending,
    above: &mut Queue<Event>,
    below: &mut Below,
) {
    let number = store.secret_state.next;
    let Some(next) = number.checked_add(1) else {
        store.phase = Phase::Stopped;
        return;
    };
    if store.secret_state.pending.insert(number, pending).is_err() {
        store.phase = Phase::Stopped;
        return;
    }
    store.secret_state.next = next;
    let kv_env = directory::env(store, env, true);
    let mut events = Queue::with_capacity(skein_kv::MAX_OUT.above);
    skein_kv::down(
        &mut store.stores.secrets,
        &kv_env,
        skein_kv::Request::Commit { owner: Token::new(number), ops },
        &mut events,
        &mut below.secrets,
    );
    directory::events_up(store, true, &mut events, above, below);
}
pub(crate) fn committed(store: &mut Store, owner: Token, above: &mut Queue<Event>) {
    let Some(pending) = store.secret_state.pending.remove(&owner.raw()) else {
        store.phase = Phase::Stopped;
        return;
    };
    match pending {
        Pending::Keep(owner) => above.push(Event::Kept { owner }),
        Pending::Erase(owner) => above.push(Event::Erased { owner }),
        Pending::Sweep => store.secret_state.waiting = false,
    }
}
pub(crate) fn secrets(
    store: &mut Store,
    env: &Env<Limits>,
    owner: Token,
    request: Secrets,
    above: &mut Queue<Event>,
    below: &mut Below,
) {
    match store.phase {
        Phase::Ready { .. } => {}
        Phase::Closed | Phase::Stopped => return,
    }
    match request {
        Secrets::Keep { owner: kind, number, value } => {
            if value.len() > usize::try_from(store.limits.secrets.value).expect("secret limit fits usize") {
                store.phase = Phase::Stopped;
                return;
            }
            let Some(key) = key(kind, number, store.limits.secrets.key) else {
                store.phase = Phase::Stopped;
                return;
            };
            let ops = Box::new([skein_kv::Op::Put { key, value: Box::from(value.expose()) }]);
            submit(store, env, ops, Pending::Keep(owner), above, below);
        }
        Secrets::Erase { owner: kind, number } => {
            let Some(key) = key(kind, number, store.limits.secrets.key) else {
                store.phase = Phase::Stopped;
                return;
            };
            submit(store, env, Box::new([skein_kv::Op::Erase { key }]), Pending::Erase(owner), above, below);
        }
        Secrets::Get { owner: kind, number } => {
            let Some(key) = key(kind, number, store.limits.secrets.key) else {
                store.phase = Phase::Stopped;
                return;
            };
            let kv_env = directory::env(store, env, true);
            let mut events = Queue::with_capacity(skein_kv::MAX_OUT.above);
            skein_kv::down(
                &mut store.stores.secrets,
                &kv_env,
                skein_kv::Request::Get { owner, key },
                &mut events,
                &mut below.secrets,
            );
            match events.pop().expect("a secret get answers in its step") {
                skein_kv::Event::Got { value, .. } => match value {
                    Some(value) => above.push(Event::Secret { owner, value: Some(Secret::new(value)) }),
                    None => above.push(Event::Secret { owner, value: None }),
                },
                skein_kv::Event::Opened { .. }
                | skein_kv::Event::Committed { .. }
                | skein_kv::Event::Refused { .. }
                | skein_kv::Event::Failed { .. }
                | skein_kv::Event::Loaded { .. }
                | skein_kv::Event::Closed { .. }
                | skein_kv::Event::OpenFailed { .. }
                | skein_kv::Event::Recovered { .. }
                | skein_kv::Event::RecoveryFailed { .. } => store.phase = Phase::Stopped,
            }
        }
        Secrets::Load { owner: kind, after } => {
            let mut range = skein_kv::Range::prefix(&prefix(kind));
            if let Some(number) = after {
                let Some(number) = number.checked_add(1) else {
                    above.push(Event::SecretsLoaded { owner, rows: Box::new([]), next: None });
                    return;
                };
                let Some(key) = key(kind, number, store.limits.secrets.key) else {
                    store.phase = Phase::Stopped;
                    return;
                };
                range.start = key;
            }
            let Some(Loaded { rows, next }) = load(store, env, range, store.limits.secrets.page) else {
                store.phase = Phase::Stopped;
                return;
            };
            let mut found = List::with_capacity(store.limits.secrets.page.rows);
            let mut last = None;
            for row in rows {
                let Some((kind, number)) = read_key(&row.key) else {
                    store.phase = Phase::Stopped;
                    return;
                };
                last = Some(number);
                found
                    .push(SecretRow { owner: kind, number, value: Secret::new(row.value) })
                    .expect("bounded secret page");
            }
            above.push(Event::SecretsLoaded {
                owner,
                rows: found.into_boxed(),
                next: if next.is_some() { last } else { None },
            });
        }
    }
}
pub(crate) fn start_sweep(store: &mut Store) {
    if !store.secret_state.swept {
        store.secret_state.sweeping = true;
    }
}
pub(crate) fn sweep(store: &mut Store, env: &Env<Limits>, above: &mut Queue<Event>, below: &mut Below) {
    if !store.secret_state.sweeping || store.secret_state.waiting || store.phase == Phase::Stopped {
        return;
    }
    let mut range = skein_kv::Range::prefix(&[1]);
    if let Some(after) = store.secret_state.after.take() {
        range.start = after;
    }
    let max = skein_kv::Page {
        rows: store.limits.secrets.page.rows.min(store.limits.secrets.ops),
        bytes: store.limits.secrets.page.bytes,
    };
    let Some(Loaded { rows, next }) = load(store, env, range, max) else {
        store.phase = Phase::Stopped;
        return;
    };
    let mut ops = List::with_capacity(store.limits.secrets.ops);
    for row in rows {
        let Some((kind, number)) = read_key(&row.key) else {
            store.phase = Phase::Stopped;
            return;
        };
        let people = match kind {
            SecretOwner::SignIn => jig_domain_people::Key::SignIn(number),
            SecretOwner::Service => jig_domain_people::Key::Person(number),
            SecretOwner::Application => {
                store.phase = Phase::Stopped;
                return;
            }
        };
        let Some(key) = keys::encode(&jig_domain_core::Key::People(people), store.limits.store.key) else {
            store.phase = Phase::Stopped;
            return;
        };
        let Ok(value) = open::get(store, Token::new(0), key) else {
            store.phase = Phase::Stopped;
            return;
        };
        if value.is_none() {
            ops.push(skein_kv::Op::Erase { key: row.key }).expect("one op per swept row");
        }
    }
    store.secret_state.after = next;
    if store.secret_state.after.is_none() {
        store.secret_state.sweeping = false;
        store.secret_state.swept = true;
    }
    if !ops.is_empty() {
        store.secret_state.waiting = true;
        submit(store, env, ops.into_boxed(), Pending::Sweep, above, below);
    }
}
