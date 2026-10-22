//! Credential values travel between protocol components only (channel.md, 8).
use alloc::boxed::Box;
use jig_domain_host as host;
use jig_protocol_accounts::table::{self, Table};
use jig_protocol_engine_workers::{GrantValue, Scheme};
use skein_lib::{List, Map, Secret, Time};
#[derive(Debug)]
pub(super) struct Values {
    pub accounts: Table,
    pub application: Map<(u32, u64), Secret>,
}
impl Values {
    pub fn put_application(&mut self, number: u32, generation: u64, value: Secret) {
        let key = (number, generation);
        if !self.application.contains_key(&key) {
            let mut held = self.application.range((number, 0)..=(number, u64::MAX));
            if let Some((&oldest, _)) = held.next()
                && held.next().is_some()
            {
                if generation < oldest.1 {
                    return;
                }
                drop(self.application.remove(&oldest));
            }
        }
        self.application.insert(key, value).expect("application credential capacity");
    }

    pub fn grants(&self, grants: &[host::Grant], now: Time) -> Box<[Option<GrantValue>]> {
        let mut values = List::with_capacity(u32::try_from(grants.len()).expect("bounded grants"));
        for grant in grants {
            let value = match grant.form {
                host::Form::Program(name) => Some(GrantValue::Program { name }),
                host::Form::Credential(_) => account_grant(self.accounts.value(grant.account, grant.generation, now)),
                host::Form::Application => application_grant(self.application.get(&(grant.account, grant.generation))),
            };
            values.push(value).expect("one value slot per grant");
        }
        values.into_boxed()
    }
}
fn account_value(value: &table::Value) -> GrantValue {
    match &value.scheme {
        table::Scheme::Bearer => GrantValue::Credential { scheme: Scheme::Bearer, token: value.token.clone() },
        table::Scheme::Key => GrantValue::Credential { scheme: Scheme::Key, token: value.token.clone() },
    }
}
fn application_value(value: &Secret) -> GrantValue {
    GrantValue::Application { secret: value.clone() }
}

fn account_grant(value: Option<&table::Value>) -> Option<GrantValue> {
    let value = value?;
    Some(account_value(value))
}
fn application_grant(value: Option<&Secret>) -> Option<GrantValue> {
    let value = value?;
    Some(application_value(value))
}
