//! LLM account requests (client/domain/client.md, 6; domain/accounts.md, 5).
use crate::{
    Ask, Event, Key,
    contract::{AccountChange, AccountHolder},
};
use alloc::boxed::Box;
use skein_lib::{RequestKey, Secret};

/// An account change under the caller's engine name.
#[must_use]
pub fn account_change(key: Key, name: RequestKey, holder: AccountHolder, change: AccountChange) -> Event {
    Event::Send {
        key,
        name: Some(name),
        request: Ask::Jig(Box::new(crate::contract::Ask::AccountChange { holder, change })),
    }
}

/// Add an account key, carrying its value once. Only its digest enters the typed ask.
#[must_use]
pub fn add_key(
    key: Key,
    name: RequestKey,
    holder: AccountHolder,
    provider: u32,
    join: Option<u32>,
    value: Secret,
) -> Event {
    let change = AccountChange::AddKey { provider, join, digest: skein_lib::sha256::digest(value.expose()) };
    Event::Carry {
        key,
        name,
        request: Ask::Jig(Box::new(crate::contract::Ask::AccountChange { holder, change })),
        value,
    }
}

/// Replace an account key, carrying its value once. Only its digest enters the typed ask.
#[must_use]
pub fn replace_key(key: Key, name: RequestKey, holder: AccountHolder, source: u32, value: Secret) -> Event {
    let change = AccountChange::ReplaceKey { source, digest: skein_lib::sha256::digest(value.expose()) };
    Event::Carry {
        key,
        name,
        request: Ask::Jig(Box::new(crate::contract::Ask::AccountChange { holder, change })),
        value,
    }
}

/// Declare a host-local program login under its portable directory name.
#[must_use]
pub fn declare_program(
    key: Key,
    request: RequestKey,
    holder: AccountHolder,
    provider: u32,
    join: Option<u32>,
    name: Box<[u8]>,
) -> Event {
    account_change(key, request, holder, AccountChange::DeclareProgram { provider, join, name })
}
/// Ask each host to refresh a declared program's inventory.
#[must_use]
pub fn check_program(key: Key, request: RequestKey, holder: AccountHolder, source: u32) -> Event {
    account_change(key, request, holder, AccountChange::CheckProgram { source })
}
