use jig_client_protocol_world::{fixtures, protocol};
use jig_client_world::{Until, stories};
#[test]
fn a_person_signs_in_by_the_printed_link_and_starts_a_chat() {
    let mut w = protocol::world(fixtures::SEED);
    let c = w.client(fixtures::FIRST_OWNER, stories::conversation());
    w.person(c, fixtures::sign_in_then(stories::start_chat_steps()));
    let outcome = w.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert_eq!(w.front().chats_made(), 1);
}

#[test]
fn a_person_s_part_request_is_answered_from_the_connector_s_outcome() {
    use jig_fake_person::{Find, Step};
    use jig_view::Role;
    let mut w = fixtures::family_world(fixtures::SEED);
    let c = w.client(fixtures::FIRST_OWNER, stories::conversation());
    let parameters = Find::named(Role::TextField, b"Parameters");
    w.person(
        c,
        fixtures::sign_in_then(vec![
            Step::See { find: Find::text(b"Signed in as Person"), within: skein_lib::Duration::from_secs(1) },
            Step::Go {
                address: jig_test_domain_client::Address::First(jig_test_domain_client::first::Address {
                    project: 1,
                    item: 1,
                }),
            },
            Step::See { find: parameters.clone(), within: skein_lib::Duration::from_secs(1) },
            Step::Type { find: parameters, words: b"42".as_slice().into() },
            Step::Press { find: Find::named(Role::Button, b"Apply parameters") },
            Step::See { find: Find::text(b"Object changed"), within: skein_lib::Duration::from_secs(1) },
            Step::See { find: Find::text(b"Value: 42"), within: skein_lib::Duration::from_secs(1) },
        ]),
    );
    let outcome = w.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert_eq!(
        w.front().application_objects(),
        vec![(1, jig_test_connector::ApplicationObject { revision: 1, value: 42 })]
    );
}

#[test]
fn a_family_place_record_round_trips_through_its_owner_s_small_store_bytes() {
    use jig_test_domain_client::first::{Address, Record};
    for (label, value) in [
        ("ordinary", Record::Place { version: 1, party: 1, address: Address { project: 2, item: 3 } }),
        (
            "maximum names",
            Record::Place { version: 1, party: u64::MAX, address: Address { project: u32::MAX, item: u64::MAX } },
        ),
    ] {
        assert_eq!(
            jig_client_protocol_world::family::restored(&jig_client_protocol_world::family::saved(&value)),
            Some(value),
            "{label}"
        );
    }
}
