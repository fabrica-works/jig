#![expect(
    clippy::wildcard_enum_match_arm,
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "ordinary entry-point proofs select requests and inspect complete output"
)]
//! Native fixtures cover received authority; source 02 separately proves its producer.
use super::spine::{LIMITS, delivered, drive, frame, open_token, start};
use crate::tests::fixtures::{Ended, Fixture, ReadPage, Watched};
use crate::{
    Event, Request,
    address::{Address, Place},
    page::Page,
};
use jig_client::contract::*;
use skein_lib::{Duration, List, Time};

fn document() -> OwnerDeployment {
    OwnerDeployment {
        providers: List::with_capacity(0),
        providers_cut: false,
        key_bytes: 256,
        readiness: List::with_capacity(0),
        readiness_cut: false,
        store: StoreFill { bytes: 0, mark: 0 },
        hosts: List::with_capacity(2),
        attempts: List::with_capacity(2),
        accounts: List::with_capacity(2),
        tasks: OwnerCapacity { used: 1, limit: 2 },
        workers: OwnerCapacity { used: 3, limit: 4 },
        attempts_capacity: OwnerCapacity { used: 5, limit: 6 },
        waiting_attempts: 7,
        proposals: OwnerCapacity { used: 8, limit: 9 },
        projects: List::with_capacity(2),
        grace_bound: Duration::from_nanos(u64::MAX),
        hello_refusals: List::with_capacity(2),
        hosts_forgotten: u64::MAX,
        hello_refusals_forgotten: 11,
        person_room_refusals: List::with_capacity(2),
        person_room_refusals_forgotten: 13,
        person_answer_retention: Duration::from_nanos(17),
        task_room_refusals: List::with_capacity(2),
        task_room_refusals_forgotten: 19,
        cut: true,
        connectors: List::with_capacity(2),
        connectors_cut: false,
    }
}
fn deploy(domain: &mut Fixture) -> Box<[Request]> {
    drive(domain, Time::ZERO, Event::Went { place: Place::Jig(Address::Deployment) })
}
fn owner(domain: &mut Fixture, watch: jig_client::Handle, value: OwnerDeployment) {
    drive(
        domain,
        Time::ZERO,
        Watched { watch, event: WatchEvent::Delivered(Delivery::OwnerDeployment(Box::new(value))) },
    );
}
#[test]
fn native_deployment_authority_none_builtins_and_admitted_policy_numbers_are_independent_of_project_rows() {
    for role in [
        None,
        Some(HeldRole::Owner),
        Some(HeldRole::Maintainer),
        Some(HeldRole::Member),
        Some(HeldRole::Observer),
        Some(HeldRole::Policy { role: 4 }),
        Some(HeldRole::Policy { role: u32::MAX }),
    ] {
        let mut domain = Fixture::new(&LIMITS);
        let watch = start(&mut domain, List::with_capacity(0));
        let mut value = frame(7);
        value.deployment_role = role;
        value.cut = true;
        assert!(value.projects.is_empty());
        delivered(&mut domain, Time::ZERO, watch, value);
        let out = deploy(&mut domain);
        if role.is_none() {
            assert_eq!(domain.page(), &Page::Missing);
            let mut opened = false;
            for request in &out {
                if let Request::Client {
                    asked: jig_client::Event::Watch { subject: jig_client::Watchable::Jig(Subject::Deployment), .. },
                } = request
                {
                    opened = true;
                    break;
                }
            }
            assert!(!opened);
        } else {
            let token = out
                .iter()
                .find_map(|r| match r {
                    Request::Client {
                        asked:
                            jig_client::Event::Watch {
                                handle: watch,
                                subject: jig_client::Watchable::Jig(Subject::Deployment),
                            },
                    } => Some(*watch),
                    _ => None,
                })
                .unwrap();
            let whole = document();
            owner(&mut domain, token, whole.clone());
            assert_eq!(domain.deployment_page().owner, Some(&whole));
            assert!(domain.deployment_page().document.is_none());
        }
    }
}
#[test]
fn native_deployment_wrong_subject_wrong_read_left_generation_and_ended_watch_never_restore_a_page() {
    let mut domain = Fixture::new(&LIMITS);
    let frame_watch = start(&mut domain, List::with_capacity(0));
    let initial = delivered(&mut domain, Time::ZERO, frame_watch, frame(7));
    let inbox = open_token(&initial);
    owner(&mut domain, inbox, document());
    assert!(domain.deployment_page().owner.is_none());
    let read = initial
        .iter()
        .find_map(|r| match r {
            Request::Client {
                asked: jig_client::Event::Read { handle: read, read: jig_client::Readable::Jig(Read::Inbox { .. }) },
            } => Some(*read),
            _ => None,
        })
        .unwrap();
    drive(
        &mut domain,
        Time::ZERO,
        ReadPage { read, result: ReadResult::Value(ReadValue::OwnerDeployment(Box::new(document()))) },
    );
    assert!(domain.deployment_page().owner.is_none());
    let opened = deploy(&mut domain);
    let watch = open_token(&opened);
    let whole = document();
    owner(&mut domain, watch, whole.clone());
    assert_eq!(domain.deployment_page().owner, Some(&whole));
    drive(&mut domain, Time::ZERO, Event::Went { place: Place::Jig(Address::Conversations { project: None }) });
    assert!(domain.deployment_page().owner.is_none());
    owner(&mut domain, watch, whole.clone());
    assert!(domain.deployment_page().owner.is_none());
    drive(&mut domain, Time::ZERO, Ended { watch, end: WatchEnd::Closed });
    owner(&mut domain, watch, whole);
    assert!(domain.deployment_page().owner.is_none());
}
#[test]
fn native_deployment_genuine_and_synthetic_documents_replace_each_other_without_inventing_connector_parts() {
    let mut domain = Fixture::new(&LIMITS);
    let frame_watch = start(&mut domain, List::with_capacity(0));
    delivered(&mut domain, Time::ZERO, frame_watch, frame(7));
    let opened = deploy(&mut domain);
    let watch = open_token(&opened);
    let old = Deployment {
        hosts: List::with_capacity(0),
        accounts: List::with_capacity(0),
        capacity: List::with_capacity(0),
        proposals: 0,
        refusals: List::with_capacity(0),
        parts: List::with_capacity(0),
    };
    drive(
        &mut domain,
        Time::ZERO,
        Watched { watch, event: WatchEvent::Delivered(Delivery::Deployment(Box::new(old.clone()))) },
    );
    assert_eq!(domain.deployment_page().document, Some(&old));
    let whole = document();
    owner(&mut domain, watch, whole.clone());
    assert!(domain.deployment_page().document.is_none());
    assert_eq!(domain.deployment_page().owner, Some(&whole));
    drive(
        &mut domain,
        Time::ZERO,
        Watched { watch, event: WatchEvent::Delivered(Delivery::Deployment(Box::new(old.clone()))) },
    );
    assert!(domain.deployment_page().owner.is_none());
    assert_eq!(domain.deployment_page().document, Some(&old));
}
#[test]
fn native_deployment_only_supplied_configured_references_open_ordered_parts_in_the_current_generation() {
    let mut domain = Fixture::new(&LIMITS);
    let frame_watch = start(&mut domain, List::with_capacity(0));
    delivered(&mut domain, Time::ZERO, frame_watch, frame(7));
    let opened = deploy(&mut domain);
    let watch = open_token(&opened);
    let mut whole = document();
    whole.connectors.push(u16::MAX).unwrap();
    whole.connectors.push(0).unwrap();
    whole.connectors_cut = true;
    let out = drive(
        &mut domain,
        Time::ZERO,
        Watched { watch, event: WatchEvent::Delivered(Delivery::OwnerDeployment(Box::new(whole.clone()))) },
    );
    let parts: Vec<_> = out
        .iter()
        .filter_map(|r| match r {
            Request::ShowPart { part, scope } => Some((part.clone(), *scope)),
            _ => None,
        })
        .collect();
    assert_eq!(
        parts,
        vec![
            (Part::Deployment { connector: u16::MAX }, PartScope::Deployment),
            (Part::Deployment { connector: 0 }, PartScope::Deployment)
        ]
    );
    assert_eq!(domain.deployment_page().owner, Some(&whole));
    let left = drive(&mut domain, Time::ZERO, Event::Went { place: Place::Elsewhere });
    assert!(left.contains(&Request::PartsLeft));
    let late = drive(
        &mut domain,
        Time::ZERO,
        Watched { watch, event: WatchEvent::Delivered(Delivery::OwnerDeployment(Box::new(whole))) },
    );
    let mut shown = false;
    for request in &late {
        if let Request::ShowPart { .. } = request {
            shown = true;
            break;
        }
    }
    assert!(!shown);
}
