//! Credential values travel between protocol components only (channel.md, 8).
use alloc::boxed::Box;
use jig_domain_host as host;
use jig_protocol_accounts::table::Table;
use jig_protocol_engine_workers::GrantValue;
use skein_lib::{List, Map, Secret, Time};
#[derive(Debug)]
pub(super) struct Values {
    pub accounts: Table,
    pub application: Map<u32, Secret>,
}
impl Values {
    pub fn grants(&self, grants: &[host::Grant], now: Time) -> Box<[Option<GrantValue>]> {
        let mut values = List::with_capacity(u32::try_from(grants.len()).expect("bounded grants"));
        for grant in grants {
            let value = match grant.table {
                host::GrantTable::Accounts => account_grant(self.accounts.value(grant.account, grant.generation, now)),
                host::GrantTable::Application if grant.generation == 1 => {
                    application_grant(self.application.get(&grant.account))
                }
                host::GrantTable::Application => None,
            };
            values.push(value).expect("one value slot per grant");
        }
        values.into_boxed()
    }
}
fn account_value(value: &jig_protocol_accounts::table::Value) -> GrantValue {
    GrantValue { value: value.token.clone(), account_id: value.account_id.clone() }
}
fn application_value(value: &Secret) -> GrantValue {
    GrantValue { value: value.clone(), account_id: Secret::new(Box::default()) }
}

fn account_grant(value: Option<&jig_protocol_accounts::table::Value>) -> Option<GrantValue> {
    let value = value?;
    Some(account_value(value))
}
fn application_grant(value: Option<&Secret>) -> Option<GrantValue> {
    let value = value?;
    Some(application_value(value))
}
