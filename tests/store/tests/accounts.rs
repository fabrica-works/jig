//! The accounts family across crashes (store.md 4.4 item 9): the counter, an
//! account and its sources are kept whole with the commit that saves them,
//! go whole with the one that erases them, and restore at start beside the
//! installation's row, which the accounts' range never hands up.
use jig_domain_accounts::{Held, Holder, Key, Kind, Stored};
use jig_domain_core as core;
use jig_protocol_store::Write;
use jig_store_world::{World, referee::Referee, root};
use skein_lib::Wall;

fn next(number: u32) -> core::Record {
    core::Record::Accounts(Stored::Next { number })
}

fn account() -> core::Record {
    core::Record::Accounts(Stored::Account {
        number: 1,
        provider: 2,
        holder: Holder::Party(7),
        subject: Box::from(&b"subject"[..]),
        display: Box::from(&b"display"[..]),
    })
}

fn source() -> core::Record {
    core::Record::Accounts(Stored::Source {
        account: 1,
        number: 2,
        place: 0,
        kind: Kind::Key,
        name: Box::from(&b"key"[..]),
        held: Some(Held::Limited { reset: Some(Wall::from_nanos(9)) }),
    })
}

fn saves(number: u64) -> Box<[Write]> {
    Box::new([Write::Save(root::header(number)), Write::Save(next(3)), Write::Save(account()), Write::Save(source())])
}

fn erases(number: u64) -> Box<[Write]> {
    Box::new([
        Write::Save(root::header(number)),
        Write::Erase(core::Key::Accounts(Key::Source { account: 1, number: 2 })),
        Write::Erase(core::Key::Accounts(Key::Account { number: 1 })),
    ])
}

/// A commit's writes, given its number.
type Commit = fn(u64) -> Box<[Write]>;

fn rows(restored: &jig_store_world::Restored) -> Vec<core::Record> {
    restored.rows.values().filter(|record| matches!(record, core::Record::Accounts(_))).cloned().collect()
}

/// A world with `commits` after its first, then `last` committed with its
/// file operations cut at `at`, when given.
fn cut(commits: &[Commit], last: Commit, at: Option<u32>) -> (World, Option<u64>) {
    let seed = at.map_or(18, u64::from);
    let mut world = World::new(seed);
    assert_eq!(world.commit(root::first()), Some(1));
    for (index, commit) in commits.iter().enumerate() {
        let number = u64::try_from(index).expect("few commits") + 2;
        assert_eq!(world.commit(commit(number)), Some(number));
    }
    if let Some(at) = at {
        world.cut_at(at);
    }
    let number = u64::try_from(commits.len()).expect("few commits") + 2;
    let acknowledged = world.commit(last(number));
    (world, acknowledged)
}

#[test]
fn the_accounts_rows_survive_a_crash_at_each_operation_of_the_commit_that_saves_them() {
    let count = cut(&[], saves, None).0.operation_count();
    assert!(count > 0, "the save writes");
    for at in 0..=count {
        let (mut world, acknowledged) = cut(&[], saves, Some(at));
        world.crash(u64::from(at));
        let restored = world.reopen();
        Referee::check(&world.model, &restored).unwrap_or_else(|error| panic!("cut {at}: {error:?}"));
        assert!(acknowledged.is_none() || restored.last == 2, "an acknowledged save is durable, cut {at}");
        let expected = if restored.last == 2 { vec![next(3), account(), source()] } else { Vec::new() };
        assert_eq!(rows(&restored), expected, "the counter, account and source whole, cut {at}");
    }
}

#[test]
fn the_accounts_rows_go_whole_at_a_crash_at_each_operation_of_the_commit_that_erases_them() {
    let count = cut(&[saves], erases, None).0.operation_count();
    assert!(count > 0, "the erase writes");
    for at in 0..=count {
        let (mut world, acknowledged) = cut(&[saves], erases, Some(at));
        world.crash(u64::from(at));
        let restored = world.reopen();
        Referee::check(&world.model, &restored).unwrap_or_else(|error| panic!("cut {at}: {error:?}"));
        assert!(acknowledged.is_none() || restored.last == 3, "an acknowledged erase is durable, cut {at}");
        let expected = if restored.last == 3 { vec![next(3)] } else { vec![next(3), account(), source()] };
        assert_eq!(rows(&restored), expected, "the account and source go together, cut {at}");
    }
}
