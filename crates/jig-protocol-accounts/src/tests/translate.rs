use super::fixtures::{self, ACCOUNT_CHAT, ACCOUNT_DELEGATE, TOKEN_FIRST, TOKEN_SECOND};
use crate::translate::{self, Up};
use jig_domain_accounts as accounts;
use skein_lib::{Duration, Secret, Time, bytes};
use skein_oauth as oauth;
use skein_oauth_accounts as driver;

fn failure(failure: driver::Failure) -> Up {
    translate::up(
        driver::Event::Failed { account: ACCOUNT_CHAT, ends: driver::Ends::Grant, failure },
        fixtures::NOW,
        b"",
    )
}
fn child_failure(failure: accounts::Failure) -> Up {
    Up::Child(accounts::FromComponent::Failed { account: ACCOUNT_CHAT, failure })
}
fn refused(why: driver::Refusal) -> Up {
    translate::up(
        driver::Event::Refused { account: ACCOUNT_CHAT, asked: driver::Asked::Grant, why },
        fixtures::NOW,
        b"",
    )
}

#[test]
fn a_rate_limited_exchange_fails_later_with_the_drivers_delay() {
    assert_eq!(
        failure(driver::Failure::Exchange(oauth::Failure::RateLimited { retry_after: Duration::from_secs(30) })),
        child_failure(accounts::Failure::Later { after: Some(Duration::from_secs(30)) })
    );
}
#[test]
fn want_becomes_a_held_grant() {
    assert!(
        translate::down(accounts::ToComponent::Want { account: ACCOUNT_DELEGATE })
            == driver::Request::Grant { account: ACCOUNT_DELEGATE }
    );
}
#[test]
fn release_and_close_pass_through() {
    assert!(
        translate::down(accounts::ToComponent::Release { account: ACCOUNT_CHAT })
            == driver::Request::Release { account: ACCOUNT_CHAT }
    );
    assert!(translate::down(accounts::ToComponent::Close) == driver::Request::Close);
    assert_eq!(translate::up(driver::Event::Closed, fixtures::NOW, b""), Up::Child(accounts::FromComponent::Closed));
}
#[test]
fn a_rejected_generation_reaches_the_driver_unchanged() {
    assert!(
        translate::down(accounts::ToComponent::Rejected { account: ACCOUNT_CHAT, generation: 2 })
            == driver::Request::Rejected { account: ACCOUNT_CHAT, generation: 2 }
    );
}
#[test]
fn a_granted_token_is_handed_out_as_a_bearer_secret() {
    for (token, generation) in [(TOKEN_FIRST, 1_u64), (TOKEN_SECOND, 2_u64)] {
        let up = translate::up(
            fixtures::granted(ACCOUNT_CHAT, token, generation, Duration::from_nanos(10)),
            fixtures::NOW,
            b"",
        );
        assert_eq!(
            up,
            Up::Granted {
                event: accounts::FromComponent::Granted {
                    form: accounts::Form::Credential(accounts::Scheme::Bearer),
                    account: ACCOUNT_CHAT,
                    generation,
                    valid: Duration::from_nanos(10)
                },
                account: ACCOUNT_CHAT,
                generation,
                value: fixtures::value(token),
                expires: Time::from_nanos(15),
            }
        );
    }
}
#[test]
fn a_timed_out_exchange_fails_later_after_the_childs_backoff() {
    for cause in [oauth::Failure::TimedOut, oauth::Failure::Unavailable] {
        assert_eq!(failure(driver::Failure::Exchange(cause)), child_failure(accounts::Failure::Later { after: None }));
    }
}
#[test]
fn malformed_limit_and_not_kept_fail_later() {
    for cause in [
        driver::Failure::NotKept,
        driver::Failure::Exchange(oauth::Failure::Malformed),
        driver::Failure::Exchange(oauth::Failure::Limit),
    ] {
        assert_eq!(failure(cause), child_failure(accounts::Failure::Later { after: None }));
    }
}
#[test]
fn refused_and_expired_fail_as_sign_in_again() {
    for cause in [driver::Failure::Exchange(oauth::Failure::Refused), driver::Failure::Expired] {
        assert_eq!(failure(cause), child_failure(accounts::Failure::SignInAgain));
    }
}
#[test]
fn an_expiring_record_reaches_the_child_as_expiring() {
    assert_eq!(
        translate::up(driver::Event::Expiring { account: ACCOUNT_CHAT, generation: 1 }, fixtures::NOW, b""),
        Up::Child(accounts::FromComponent::Expiring { account: ACCOUNT_CHAT, generation: 1 })
    );
}
#[test]
fn a_client_rejected_fails_as_misconfigured() {
    assert_eq!(
        failure(driver::Failure::Exchange(oauth::Failure::ClientRejected)),
        child_failure(accounts::Failure::Misconfigured)
    );
}
#[test]
fn an_unloaded_file_keeps_its_place_and_cause() {
    assert_eq!(
        failure(driver::Failure::Unloaded { at: driver::Place::File, why: driver::Unloaded::Unreadable }),
        child_failure(accounts::Failure::Unloaded { at: accounts::Place::File, why: accounts::Unread::Unreadable })
    );
}
#[test]
fn a_grant_refused_as_full_fails_later_with_no_delay() {
    assert_eq!(refused(driver::Refusal::Full { bound: 2 }), child_failure(accounts::Failure::Later { after: None }));
}
#[test]
fn a_grant_refused_as_closed_is_cancelled() {
    assert_eq!(refused(driver::Refusal::Closed), child_failure(accounts::Failure::Cancelled));
}
#[test]
fn a_record_to_keep_goes_to_the_store_not_the_child() {
    let record = crate::handed_in::record(
        ACCOUNT_CHAT,
        Secret::new(bytes::copy_of(TOKEN_FIRST)),
        skein_lib::Wall::EPOCH,
        None,
        &fixtures::driver_limits().client.document,
    )
    .expect("record fits");
    let up = translate::up(driver::Event::Keep { account: ACCOUNT_CHAT, record: record.clone() }, fixtures::NOW, b"");
    assert_eq!(up, Up::Keep { account: ACCOUNT_CHAT, record: translate::Candidate(record) });
}

#[test]
fn each_scheme_names_the_form_it_is_granted_as() {
    use crate::table;
    use jig_domain_accounts::{Form, Scheme};
    for (scheme, form) in [
        (table::Scheme::Bearer, Form::Credential(Scheme::Bearer)),
        (table::Scheme::Key, Form::Credential(Scheme::Key)),
        (table::Scheme::Withdrawn { id: Secret::new(Box::from(&b"id"[..])) }, Form::Withdrawn),
    ] {
        assert_eq!(translate::form(&scheme), form);
    }
}
