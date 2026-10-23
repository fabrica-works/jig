//! Independent literals for complete accounts watch records.
use super::live_values::{encoded, pieces, rows};
use super::{limits, w, wire_text};
use skein_lib::Duration;

#[must_use]
pub fn account_sources_record() -> Box<[u8]> {
    let l = limits();
    let sources = [
        w::OwnerAccountSource::new(
            &l,
            w::OwnerAccountSourceParts {
                name: None,
                place: 2,
                number: 11,
                kind: w::OwnerAccountKind::SignIn,
                state: w::OwnerAccountState::Granted,
                usable: true,
                expiring: false,
                attention: true,
                spent_until: Some(101),
            },
        )
        .expect("literal"),
        w::OwnerAccountSource::new(
            &l,
            w::OwnerAccountSourceParts {
                name: None,
                place: 5,
                number: 12,
                kind: w::OwnerAccountKind::Key,
                state: w::OwnerAccountState::Later(
                    w::OwnerAccountStateLater::new(&l, w::OwnerAccountStateLaterParts { until: None })
                        .expect("literal"),
                ),
                usable: false,
                expiring: true,
                attention: false,
                spent_until: None,
            },
        )
        .expect("literal"),
    ];
    pieces(sources.iter().map(|s| encoded(s.measure(), |w| s.encode(w))))
}
#[must_use]
pub fn account_line_record() -> w::OwnerAccountLine {
    let l = limits();
    w::OwnerAccountLine::new(
        &l,
        w::OwnerAccountLineParts {
            provider_name: None,
            manage_usage: None,
            number: 7,
            place: 3,
            uses: Some(rows([
                w::OwnerAccountUse::Party(
                    w::OwnerAccountUseParty::new(&l, w::OwnerAccountUsePartyParts { party: 23 }).expect("literal"),
                ),
                w::OwnerAccountUse::Project(
                    w::OwnerAccountUseProject::new(&l, w::OwnerAccountUseProjectParts { project: 29 })
                        .expect("literal"),
                ),
            ])),
            plan: true,
            provider: Some(3),
            display: w::DocumentText::new(
                &l,
                w::DocumentTextParts { bytes: Box::from(b"Personal".as_slice()), cut: true },
            )
            .expect("literal"),
            holder: w::OwnerAccountHolder::Party(
                w::OwnerAccountHolderParty::new(&l, w::OwnerAccountHolderPartyParts { party: 23 }).expect("literal"),
            ),
            sources: account_sources_record(),
            cut: true,
        },
    )
    .expect("literal")
}
#[must_use]
pub fn accounts_record(end: Option<w::OwnerAccountEnd>) -> w::OwnerAccounts {
    let l = limits();
    let line = account_line_record();
    let other = w::OwnerAccountLine::new(
        &l,
        w::OwnerAccountLineParts {
            provider_name: None,
            manage_usage: None,
            number: 9,
            place: 4,
            uses: None,
            plan: false,
            provider: None,
            display: wire_text(b""),
            holder: w::OwnerAccountHolder::Deployment,
            sources: Box::from([]),
            cut: false,
        },
    )
    .expect("literal");
    let signing = w::OwnerAccountSignIn::new(
        &l,
        w::OwnerAccountSignInParts {
            source: 19,
            provider: 3,
            holder: w::OwnerAccountHolder::Party(
                w::OwnerAccountHolderParty::new(&l, w::OwnerAccountHolderPartyParts { party: 23 }).expect("literal"),
            ),
        },
    )
    .expect("literal");
    w::OwnerAccounts::new(
        &l,
        w::OwnerAccountsParts {
            skein_bools: [true, false, true, false],
            readiness: Box::new([]),
            program_hosts: Box::new([]),
            accounts: pieces([
                encoded(line.measure(), |w| line.encode(w)),
                encoded(other.measure(), |w| other.encode(w)),
            ]),
            signing: pieces([encoded(signing.measure(), |w| signing.encode(w))]),
            last_end: end.map(|end| {
                w::OwnerAccountLastEnd::new(&l, w::OwnerAccountLastEndParts { source: 11, end }).expect("literal")
            }),
        },
    )
    .expect("literal")
}
#[must_use]
pub fn account_end_records() -> Vec<w::OwnerAccountEnd> {
    let l = limits();
    let mut ends = vec![
        w::OwnerAccountEnd::Made(
            w::OwnerAccountEndMade::new(&l, w::OwnerAccountEndMadeParts { account: 7, declined: false })
                .expect("literal"),
        ),
        w::OwnerAccountEnd::Renewed(
            w::OwnerAccountEndRenewed::new(&l, w::OwnerAccountEndRenewedParts { account: 7, declined: true })
                .expect("literal"),
        ),
    ];
    let refusals = [
        w::OwnerAccountRefusal::NotLimited,
        w::OwnerAccountRefusal::Uses,
        w::OwnerAccountRefusal::NotKept(
            w::OwnerAccountRefusalNotKept::new(
                &l,
                w::OwnerAccountRefusalNotKeptParts { why: w::OwnerAccountKeyFailure::NotStored },
            )
            .expect("literal"),
        ),
        w::OwnerAccountRefusal::Limit(
            w::OwnerAccountRefusalLimit::new(
                &l,
                w::OwnerAccountRefusalLimitParts { bound: w::OwnerAccountBound::HolderAccounts },
            )
            .expect("literal"),
        ),
        w::OwnerAccountRefusal::Held(
            w::OwnerAccountRefusalHeld::new(&l, w::OwnerAccountRefusalHeldParts { provider: 3 }).expect("literal"),
        ),
        w::OwnerAccountRefusal::Provider(
            w::OwnerAccountRefusalProvider::new(&l, w::OwnerAccountRefusalProviderParts { provider: 3 })
                .expect("literal"),
        ),
        w::OwnerAccountRefusal::Subject,
        w::OwnerAccountRefusal::Unknown,
        w::OwnerAccountRefusal::Numbers,
        w::OwnerAccountRefusal::Kind(
            w::OwnerAccountRefusalKind::new(&l, w::OwnerAccountRefusalKindParts { kind: w::OwnerAccountKind::Program })
                .expect("literal"),
        ),
        w::OwnerAccountRefusal::Closing,
        w::OwnerAccountRefusal::NotStarted,
        w::OwnerAccountRefusal::NotSignedIn,
        w::OwnerAccountRefusal::NotOurs,
        w::OwnerAccountRefusal::Cancelled,
        w::OwnerAccountRefusal::Moving,
    ];
    for refusal in refusals {
        ends.push(w::OwnerAccountEnd::Refused(
            w::OwnerAccountEndRefused::new(&l, w::OwnerAccountEndRefusedParts { refusal }).expect("literal"),
        ));
    }
    let failures = [
        w::OwnerAccountFailure::Later(
            w::OwnerAccountFailureLater::new(
                &l,
                w::OwnerAccountFailureLaterParts { after: Some(Duration::from_nanos(103)) },
            )
            .expect("literal"),
        ),
        w::OwnerAccountFailure::Later(
            w::OwnerAccountFailureLater::new(&l, w::OwnerAccountFailureLaterParts { after: None }).expect("literal"),
        ),
        w::OwnerAccountFailure::SignInAgain,
        w::OwnerAccountFailure::MissingScope(
            w::OwnerAccountFailureMissingScope::new(&l, w::OwnerAccountFailureMissingScopeParts { scope: 31 })
                .expect("literal"),
        ),
        w::OwnerAccountFailure::Misconfigured,
        w::OwnerAccountFailure::Unloaded(
            w::OwnerAccountFailureUnloaded::new(
                &l,
                w::OwnerAccountFailureUnloadedParts {
                    at: w::OwnerAccountPlace::File,
                    why: w::OwnerAccountUnread::Stalled,
                },
            )
            .expect("literal"),
        ),
        w::OwnerAccountFailure::Cancelled,
    ];
    for failure in failures {
        ends.push(w::OwnerAccountEnd::Failed(
            w::OwnerAccountEndFailed::new(&l, w::OwnerAccountEndFailedParts { failure }).expect("literal"),
        ));
    }
    ends
}
