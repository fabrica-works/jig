use super::{fixtures, tasks};
use crate::forget::{Owner, Reading, forgets};

const TASK: u64 = 1;
const PARTY: u64 = 1;
const PARENT: u64 = 2;
const READ: Reading = Reading { results: 7, turns: Some(2), named: false };
const CASES: [(&str, tasks::Retention, tasks::Party, Reading, Option<u64>); 6] = [
    ("kept", tasks::Retention::Kept, tasks::Party::Person(PARTY), READ, None),
    ("task requester", tasks::Retention::ForgottenOnceRead, tasks::Party::Task(PARENT), READ, None),
    (
        "unread turn",
        tasks::Retention::ForgottenOnceRead,
        tasks::Party::Person(PARTY),
        Reading { turns: Some(1), ..READ },
        None,
    ),
    (
        "live reference",
        tasks::Retention::ForgottenOnceRead,
        tasks::Party::Person(PARTY),
        Reading { named: true, ..READ },
        None,
    ),
    (
        "unread result",
        tasks::Retention::ForgottenOnceRead,
        tasks::Party::Person(PARTY),
        Reading { results: 6, ..READ },
        None,
    ),
    ("released", tasks::Retention::ForgottenOnceRead, tasks::Party::Person(PARTY), READ, Some(PARTY)),
];

#[test]
fn forgets_names_the_party_only_once_its_reading_releases_the_task() {
    for (name, retention, requester, reading, expected) in CASES {
        let mut row = fixtures::ended_report(TASK, requester);
        row.flags.retention = retention;
        let party = match Owner::of_row(&row) {
            Some(owner) => forgets(&owner, reading),
            None => None,
        };
        assert_eq!(party, expected, "{name}");
    }
}

#[test]
fn an_owner_of_a_cached_result_is_the_owner_of_its_record() {
    let row = fixtures::ended_report(TASK, tasks::Party::Person(PARTY));
    let result = jig_domain_people::ResultRef {
        task: row.number,
        project: row.project,
        at: row.ended_at.expect("ended fixture"),
        position: row.result_position,
        forgettable: Some(jig_domain_people::Forgettable {
            label: row.label,
            listed: row.flags.listed,
            last_reply: Some(row.last_reply),
        }),
    };
    assert_eq!(Owner::of_result(PARTY, &result), Owner::of_row(&row));
}
