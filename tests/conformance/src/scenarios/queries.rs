//! Query deliveries and ended pages checked against the durable index.
use super::{World, ask, check};
use crate::{Neighbours, Store};
use jig_conformance::{referee::Violation, scenarios::Action};
use jig_domain_core::{self as core, documents, reads};
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{ReplyTo, Token};

const WATCH: u64 = 7090;
const READ: u64 = 7091;
#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    next: Option<reads::Cursor>,
    found: Vec<u64>,
    checks: u32,
    deliveries: u32,
}
fn query() -> documents::TaskQuery {
    jig_core_world::fixtures::OWN_TASKS
}
fn indexed(store: &Store) -> Vec<u64> {
    store
        .rows
        .keys()
        .filter_map(|key| match key {
            root::Key::Core(core::Key::Tasks(tasks::Key::EndedByParty { party: 1, task, .. })) => Some(*task),
            root::Key::Core(_) | root::Key::Connector { .. } => None,
        })
        .collect()
}
fn read(world: &mut World, from: Option<reads::Cursor>) {
    world.send(root::Event::Core(core::Event::Read {
        reply: ReplyTo::new(Token::new(READ)),
        sign_in: world.peers.peers.parties[0].sign_in.expect("signed in"),
        read: reads::Read::Tasks { query: query(), live: None, ended: from },
    }));
}
pub(super) fn advance(world: &mut World, action: Action, phase: &mut u8) -> Result<bool, Violation> {
    match action {
        Action::WatchQuery => {
            if *phase < 2 && world.peers.children.is_empty() {
                let request = jig_core_world::fixtures::create_chat(
                    world.peers.peers.creating.as_ref().expect("fixture creating authority"),
                    1,
                    0,
                    b"query task".as_slice().into(),
                );
                ask(world, 110 + *phase, request);
                *phase += 1;
                return Ok(true);
            }
            if world.peers.children.is_empty() {
                world.peers.children = world
                    .store
                    .rows
                    .values()
                    .filter_map(|record| match record {
                        root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row))) => Some(row.number),
                        root::Record::Core(_) | root::Record::Connector { .. } => None,
                    })
                    .collect();
            }
            world.send(root::Event::Core(core::Event::Watch {
                watcher: Token::new(WATCH),
                sign_in: world.peers.peers.parties[0].sign_in.expect("signed in"),
                subject: documents::Watched::Query { query: query() },
                window: documents::Window { rows: 4, after: None },
            }));
        }
        Action::EndTask => {
            let task = *world.peers.children.iter().find(|task| !super::ended(world, **task)).expect("live query task");
            if *phase == 0 && !world.peers.assignments.iter().any(|(assigned, _)| *assigned == task) {
                world.advance(6_000_000_000);
                *phase = 1;
                return Ok(true);
            }
            if *phase == 1 {
                world.advance(1_000_000_000);
                *phase = 2;
                return Ok(true);
            }
            super::message(world, task);
        }
        Action::ReadEnded => {
            if *phase == 0 {
                world.peers.query.found.clear();
                *phase = 1;
                read(world, Some(reads::Cursor::Store { scope: reads::Scope::Tasks { query: query() }, after: None }));
                return Ok(true);
            }
            if let Some(next) = world.peers.query.next.take() {
                read(world, Some(next));
                return Ok(true);
            }
            check(
                world.peers.query.found == indexed(&world.store),
                "ended query differs from its durable party index",
            )?;
            world.peers.query.checks += 1;
        }
        Action::Compound
        | Action::CompoundAgain
        | Action::CancelWithGrace
        | Action::Finish
        | Action::Ready
        | Action::Late
        | Action::Effect(_)
        | Action::DeliverLate
        | Action::OtherHand
        | Action::HoldWrites
        | Action::ReleaseWrites
        | Action::CheckDeadline
        | Action::ChangeJudge
        | Action::FillPool
        | Action::ShrinkPool
        | Action::FinishHolder(_)
        | Action::Propose
        | Action::NarrowPolicy
        | Action::AcceptProposal
        | Action::StartStanding
        | Action::Period(_)
        | Action::Resume
        | Action::PartyAmend
        | Action::RunAmend
        | Action::Conversation { .. }
        | Action::Turn
        | Action::MarkTurns
        | Action::EndConversations
        | Action::MarkResults => unreachable!("query action"),
    }
    Ok(false)
}
fn line(store: &Store, line: &documents::TaskLine) {
    let task = line.chip.number;
    let record = store
        .rows
        .get(&root::Key::Core(core::Key::Tasks(tasks::Key::Live(task))))
        .or_else(|| store.rows.get(&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(task)))));
    let Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Live(owner) | tasks::Stored::Ended(owner)))) =
        record
    else {
        panic!("delivered query line has no durable owner")
    };
    assert_eq!((line.project, line.requester), (owner.project, owner.requester));
}
pub(crate) fn delivered(peers: &mut Neighbours, store: &Store, delivery: &root::Delivery) {
    match delivery {
        root::Delivery::Core(core::Held::Delivery { watcher, document }) if *watcher == Token::new(WATCH) => {
            let documents::Document::Lines(lines) = document.as_ref() else { panic!("watched query") };
            for value in &lines.lines {
                line(store, value);
            }
            peers.query.deliveries += 1;
        }
        root::Delivery::Core(core::Held::Read { reply, answer }) if *reply == ReplyTo::new(Token::new(READ)) => {
            let reads::Answer::Tasks(page) = answer else { panic!("ended query page: {answer:?}") };
            let reads::EndedGroup::Rows { rows, next, .. } = &page.ended else { panic!("ended index") };
            for value in rows {
                line(store, value);
                peers.query.found.push(value.chip.number);
            }
            peers.query.next.clone_from(next);
        }
        root::Delivery::Link { .. }
        | root::Delivery::ProjectMade { .. }
        | root::Delivery::CallBusy { .. }
        | root::Delivery::ApplicationDocument { .. }
        | root::Delivery::ApplicationRead { .. }
        | root::Delivery::Application { .. }
        | root::Delivery::Projection { .. }
        | root::Delivery::Message { .. }
        | root::Delivery::Restart(_)
        | root::Delivery::Core(_)
        | root::Delivery::CallAnswer { .. }
        | root::Delivery::Assigned { .. }
        | root::Delivery::Fleet(_)
        | root::Delivery::System { .. }
        | root::Delivery::LostRead { .. }
        | root::Delivery::Procedure { .. } => {}
    }
}
pub(super) fn check_ending(world: &World) -> Result<(), Violation> {
    check(
        world.peers.children.len() == 3
            && indexed(&world.store).len() == 3
            && world.peers.query.checks == 3
            && world.peers.query.deliveries >= 4,
        "query was not watched and read across all three task ends and restarts",
    )
}
