//! Each range's intervals against its predicate (protocol/store.md, 4.3).
use super::fixtures;
use crate::{intervals, keys};
use alloc::boxed::Box;
use jig_domain_core::{CoreKey, Key, RestoreRange};
use jig_domain_notes as notes;
use skein_lib::Rng;

fn key_bytes(key: &Key) -> Box<[u8]> {
    keys::encode(key, fixtures::KEY_LIMIT).expect("within the key limit")
}

fn samples() -> core::ops::Range<u32> {
    0..500
}

#[test]
fn each_restart_range_holds_the_keys_it_admits_and_no_others() {
    let mut rng = Rng::new(0x5701_e004);
    for _ in samples() {
        let key = fixtures::key(&mut rng);
        let bytes = key_bytes(&key);
        for range in RestoreRange::LIVE {
            let inside = fixtures::within(&intervals::restore(&range), &bytes);
            assert_eq!(range.admits(&key), inside, "{range:?} and {key:?}");
        }
    }
}
#[test]
fn the_restart_reads_its_ranges_in_the_cores_order() {
    let expected = [&[1, 0, 0][..], &[1, 2], &[1, 1, 0], &[1, 0, 1]];
    for (range, start) in RestoreRange::LIVE.into_iter().zip(expected) {
        let actual = intervals::restore(&range);
        assert_eq!(actual.first().expect("one live interval").start.as_ref(), start);
    }
}
#[test]
fn a_notes_load_holds_exactly_its_scopes_lines() {
    let mut rng = Rng::new(0x5701_e005);
    let wanted = notes::Scope::Project { project: 1 };
    let range = notes::Range::Lines { scope: wanted.clone(), after: Some(1) };
    let intervals = intervals::notes(&range);
    for _ in samples() {
        let key = fixtures::key(&mut rng);
        let admitted = match &key {
            Key::Notes(notes::Key::Line { scope, name }) => scope == &wanted && *name > 1,
            Key::Notes(notes::Key::Entry { .. }) | Key::Core(_) | Key::People(_) | Key::Tasks(_) => false,
        };
        let bytes = key_bytes(&key);
        assert_eq!(fixtures::within(&intervals, &bytes), admitted, "{key:?}");
    }
}
#[test]
fn a_notes_entry_load_holds_only_its_globally_named_entry() {
    let intervals = intervals::notes(&notes::Range::Entry { name: 1 });
    for key in [
        Key::Notes(notes::Key::Entry { name: 0 }),
        Key::Notes(notes::Key::Entry { name: 1 }),
        Key::Notes(notes::Key::Entry { name: 2 }),
        Key::Notes(notes::Key::Line { scope: notes::Scope::Deployment, name: 1 }),
    ] {
        let bytes = key_bytes(&key);
        let admitted = key == Key::Notes(notes::Key::Entry { name: 1 });
        assert_eq!(fixtures::within(&intervals, &bytes), admitted, "{key:?}");
    }
}
#[test]
fn no_restart_interval_holds_a_turn_terminal_or_note() {
    for key in [
        Key::Core(CoreKey::Turn { task: 1, attempt: 1, turn: 1 }),
        Key::Core(CoreKey::Terminal { task: 1, attempt: 1 }),
        Key::Notes(notes::Key::Entry { name: 1 }),
        Key::Notes(notes::Key::Line { scope: notes::Scope::Deployment, name: 1 }),
    ] {
        let bytes = key_bytes(&key);
        for range in RestoreRange::LIVE {
            assert!(!fixtures::within(&intervals::restore(&range), &bytes), "{range:?} excludes {key:?}");
        }
    }
}

#[test]
fn each_fixed_scope_holds_the_keys_it_admits_and_no_others() {
    use jig_domain_core::reads;
    let mut rng = Rng::new(0x5701_e006);
    let mut mappings = skein_lib::List::with_capacity(26);
    for scope in fixed_scopes() {
        for party in [1, 2] {
            let intervals = intervals::scope(party, &scope).expect("fixed scope");
            mappings.push((party, scope.clone(), intervals)).expect("thirteen scopes and two parties");
        }
    }
    for _ in samples() {
        let key = fixtures::key(&mut rng);
        let bytes = key_bytes(&key);
        for (party, scope, intervals) in &mappings {
            assert_eq!(
                reads::admits(*party, scope, &key),
                fixtures::within(intervals, &bytes),
                "{party}, {scope:?}, {key:?}"
            );
        }
    }
}

fn fixed_scopes() -> [jig_domain_core::reads::Scope; 13] {
    use jig_domain_core::reads::Scope;
    [
        Scope::Proposal { proposer: jig_domain_people::Proposer::Party(1), proposal: 1 },
        Scope::Proposal { proposer: jig_domain_people::Proposer::Party(2), proposal: 1 },
        Scope::Spend { project: 1 },
        Scope::Goals { project: 1 },
        Scope::Task { task: 1 },
        Scope::PersonTask { task: 1 },
        Scope::Result { task: 1 },
        Scope::Reply { task: 1, activity: 1 },
        Scope::History { task: 1, subtree: false },
        Scope::Notes { project: 0 },
        Scope::Notes { project: 1 },
        Scope::ScopedNotes { scope: notes::Scope::Project { project: 1 }, description: None },
        Scope::ScopedNotes { scope: notes::Scope::Deployment, description: None },
    ]
}
