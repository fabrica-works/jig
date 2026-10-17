use super::fixtures::{self, Client, TASK};
use crate::contract::{Scope, Window};
use crate::*;
use alloc::boxed::Box;
const DOCUMENT_BYTES: u64 = 32;
const TOO_MANY: usize = 33;
const RANGE: contract::VersionRange = contract::VersionRange { first: 2, last: 3 };
const SERVED: contract::VersionRange = contract::VersionRange { first: 1, last: 1 };
const RANGES: [Option<contract::VersionRange>; 2] = [None, Some(RANGE)];
const KEPT_VERSIONS: [(u16, u32); 2] = [(19, 2), (20, 1)];
fn live() -> Client {
    let mut client = Client::live();
    client.step(Event::Up { up: fixtures::me(fixtures::PARTY, skein_lib::Duration::from_secs(10)) });
    client.told();
    client
}
fn record() -> Family {
    Family { family: 19, version: 1, bytes: Box::new([255, 0, 1]) }
}
fn served(client: &mut Client, versions: Option<contract::VersionRange>) {
    client.step(Event::Up { up: Up::FamilyServed { family: 19, versions } });
}

#[test]
fn a_family_document_past_the_bytes_limit_ends_its_handle() {
    let mut limits = fixtures::limits();
    limits.document_bytes = DOCUMENT_BYTES;
    let mut client = Client::new(limits);
    let subject = Watchable::Family {
        scope: Scope::Project(1),
        window: Window { rows: 3, after: None },
        record: Family { family: 1, version: 1, bytes: Box::new([7]) },
    };
    client.step(Event::Watch { handle: TASK, subject: subject.clone() });
    let watch = client.name(subject);
    client.downs();
    let delivery =
        Up::FamilyDelivered { watch, record: Family { family: 1, version: 1, bytes: Box::new([7; TOO_MANY]) } };
    client.step(Event::Up { up: delivery.clone() });
    assert_eq!(client.told().as_ref(), &[Told::Ended { handle: TASK, why: Why::Limit }]);
    assert_eq!(client.downs().as_ref(), &[Down::Close { watch }]);
    assert_eq!(client.domain.document(TASK), Some(Document { state: State::Ended, latest: None }));
    client.step(Event::Up { up: delivery });
    assert_eq!(client.told().as_ref(), &[]);
    assert_eq!(client.downs().as_ref(), &[]);
}

#[test]
fn a_family_owner_not_served_keeps_its_requests_unsent() {
    let mut client = live();
    served(&mut client, Some(RANGE));
    client
        .step(Event::Send { key: fixtures::KEY, request: Ask::Family { scope: Scope::Project(1), record: record() } });
    let downs = client.downs();
    assert_eq!(downs.len(), 1);
    let [Down::Save { record: saved }] = downs.as_ref() else { panic!("one kept record") };
    assert_eq!(
        saved.request,
        Held { key: fixtures::KEY, request: Ask::Family { scope: Scope::Project(1), record: record() } }
    );
    assert_eq!(client.domain.pending(fixtures::KEY).expect("kept request").progress, Progress::NotServed);
    client.at(client.env.now.saturating_add(client.env.limits.requests.first));
    assert_eq!(client.downs().as_ref(), &[]);
}

#[test]
fn a_family_owner_not_served_ends_its_watches_not_served() {
    let mut client = live();
    client.step(Event::Watch {
        handle: TASK,
        subject: Watchable::Family {
            scope: Scope::Project(1),
            window: Window { rows: 3, after: None },
            record: record(),
        },
    });
    let watch = client.name(Watchable::Family {
        scope: Scope::Project(1),
        window: Window { rows: 3, after: None },
        record: record(),
    });
    client.downs();
    client.told();
    served(&mut client, Some(RANGE));
    assert_eq!(client.told().as_ref(), &[Told::Ended { handle: TASK, why: Why::NotServed }]);
    assert_eq!(client.downs().as_ref(), &[Down::Close { watch }]);
    assert_eq!(client.domain.document(TASK).expect("ended handle").state, State::Ended);
    served(&mut client, Some(RANGE));
    assert_eq!(client.told().as_ref(), &[]);
}

#[test]
fn a_family_owner_served_again_sends_what_was_kept() {
    let mut client = live();
    served(&mut client, None);
    client
        .step(Event::Send { key: fixtures::KEY, request: Ask::Family { scope: Scope::Project(1), record: record() } });
    let downs = client.downs();
    let [Down::Save { record: saved }] = downs.as_ref() else { panic!("one kept record") };
    served(&mut client, Some(SERVED));
    client.at(client.env.now.saturating_add(client.env.limits.requests.first));
    let sent = client.downs();
    let [Down::Send { key, request, .. }] = sent.as_ref() else { panic!("one resumed attempt: {sent:?}") };
    assert_eq!(*key, saved.key);
    assert_eq!(*request, Ask::Family { scope: Scope::Project(1), record: record() });
    assert_eq!(client.domain.pending(fixtures::KEY).expect("pending request").progress, Progress::InFlight);
}

#[test]
fn a_read_of_an_owner_not_served_is_answered_at_once_naming_its_range() {
    for range in RANGES {
        let mut client = live();
        served(&mut client, range);
        client.step(Event::Read {
            handle: TASK,
            read: Readable::Family {
                scope: Scope::Project(1),
                window: Window { rows: 3, after: None },
                record: record(),
            },
        });
        assert_eq!(client.downs().as_ref(), &[], "{range:?}");
        assert_eq!(
            client.told().as_ref(),
            &[Told::Page { handle: TASK, page: Readout::NotServed { family: Some(19), served: range } }],
            "{range:?}"
        );
    }
}

#[test]
fn a_read_while_out_of_date_is_answered_not_served() {
    let mut client = live();
    client.step(Event::Up { up: Up::Served { served: false, versions: Some(RANGE) } });
    client.told();
    for read in [
        fixtures::project(1),
        Readable::Family { scope: Scope::Project(1), window: Window { rows: 3, after: None }, record: record() },
    ] {
        client.step(Event::Read { handle: TASK, read });
        assert_eq!(client.downs().as_ref(), &[]);
        assert_eq!(
            client.told().as_ref(),
            &[Told::Page { handle: TASK, page: Readout::NotServed { family: None, served: Some(RANGE) } }]
        );
    }
}

#[test]
fn a_kept_family_request_of_another_version_is_dropped_and_told() {
    for (family, version) in KEPT_VERSIONS {
        let mut client = Client::new(fixtures::limits());
        let mut kept = skein_lib::List::with_capacity(1);
        let mut saved = fixtures::record(fixtures::KEY, fixtures::ENGINE_KEY, fixtures::PARTY);
        saved.request.request =
            Ask::Family { scope: Scope::Project(1), record: Family { family, version, bytes: Box::new([7]) } };
        kept.push(saved).unwrap();
        client.step(Event::Start { kept });
        assert_eq!(client.told().as_ref(), &[Told::Dropped { count: 1 }], "{family}/{version}");
        assert_eq!(client.downs().as_ref(), &[], "{family}/{version}");
        assert_eq!(client.domain.pending(fixtures::KEY), None, "{family}/{version}");
    }
}
