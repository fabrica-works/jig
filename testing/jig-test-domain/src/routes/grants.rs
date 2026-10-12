//! Translate the owned grants of a prepared assignment to the fleet.
use alloc::boxed::Box;
use jig_domain_accounts as accounts;
use jig_domain_fleet as fleet;
use skein_lib::List;

pub(super) fn fleet(grants: &[accounts::Grant]) -> Box<[fleet::Grant]> {
    let mut translated = List::with_capacity(u32::try_from(grants.len()).expect("bounded grants"));
    for grant in grants {
        translated
            .push(fleet::Grant {
                table: match grant.table {
                    accounts::GrantTable::Accounts => fleet::GrantTable::Accounts,
                    accounts::GrantTable::Application => fleet::GrantTable::Application,
                },
                account: grant.account,
                generation: grant.generation,
                valid: grant.valid,
            })
            .expect("grant room");
    }
    translated.into_boxed()
}
