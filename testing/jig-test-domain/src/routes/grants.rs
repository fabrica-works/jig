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
                form: fleet_form(grant.form),
                account: grant.account,
                generation: grant.generation,
                valid: grant.valid,
            })
            .expect("grant room");
    }
    translated.into_boxed()
}

fn fleet_form(form: accounts::Form) -> fleet::Form {
    match form {
        accounts::Form::Credential(scheme) => fleet::Form::Credential(match scheme {
            accounts::Scheme::Bearer => fleet::Scheme::Bearer,
            accounts::Scheme::Key => fleet::Scheme::Key,
        }),
        accounts::Form::Application => fleet::Form::Application,
        accounts::Form::Program(name) => fleet::Form::Program(name),
    }
}
