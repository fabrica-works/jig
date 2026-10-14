//! Committed inbox facts for the renderer's page goldens.
use super::{held_chat, proposal_world, task, text};
use crate::{Scripted, Test, World, test};
use jig_domain_client::contract::{
    self as c, Actor, ConversationLine, Count, DocumentText, Ending, Executor, InboxEntry, InboxGroup, InboxReadGroup,
    Needs, ObjectKey, OwnerEscalation, OwnerRequester, PersonContract, PersonTask, Phase, Question, Read, ReadValue,
    Recipient, ResultValue, Revision, Runtime,
};
use jig_scripted_engine::{Change, Rows, Source};
use skein_lib::{Time, Wall};
/// Enough distinct objects for an inbox containing every entry kind.
#[must_use]
pub fn limits() -> jig_test_domain_client::Limits {
    let mut limits = test::limits();
    limits.jig.objects = 8;
    limits.jig.watches = 16;
    limits.jig.rows = 4;
    limits.jig.document_bytes = 8192;
    limits
}
#[must_use]
pub fn client(seed: u64) -> (jig_test_domain_client::Domain, jig_test_view::View<'static>) {
    test::new(&limits(), seed)
}
/// One row of each inbox kind, with its real card document.
#[must_use]
pub fn inbox() -> World<Test, Scripted> {
    let mut world = proposal_world(91);
    let reply = cards(&mut world);
    rows(&mut world, &reply);
    world
}
fn cards(world: &mut World<Test, Scripted>) -> ConversationLine {
    let ReadValue::Task(held) = held_chat(91).front().engine.truth().objects[&ObjectKey::Task(9)].clone() else {
        panic!("held fixture is a task")
    };
    let mut frame = super::frame(7);
    frame.needs = Needs { questions: 1, proposals: 1, escalations: 1, person_tasks: 1 };
    frame.needs_total = 4;
    frame.updates.value = 2;
    world.front().engine.act(Time::ZERO, Wall::EPOCH, Change::Frame(frame)).expect("golden frame fits");
    let task = task();
    let question = Question {
        task: task.chip.clone(),
        from: Some(OwnerRequester::Person(7)),
        message: 1,
        words: text(b"Which option?"),
        since: Wall::EPOCH,
        acts: test::list([]),
        ending: None,
    };
    let mut person_chip = task.chip.clone();
    person_chip.task = 10;
    person_chip.executor = Executor::Party { party: 7 };
    let person = PersonTask {
        task: person_chip,
        revision: Revision { value: 1 },
        requester: Actor::Party(7),
        spec: task.spec.clone(),
        contract: PersonContract::Words,
        inputs: test::list([]),
        addressed: Recipient::Party(7),
        taken: None,
        since: Some(Wall::EPOCH),
        stall: None,
        deadline: None,
        acts: test::list([]),
        ending: None,
    };
    let mut result_chip = task.chip.clone();
    result_chip.task = 11;
    result_chip.phase = Phase::Done;
    let result = c::Result {
        task: result_chip,
        ending: Ending::Done,
        value: Some(ResultValue::Report(DocumentText::complete(text(b"Finished")))),
        at: Some(Wall::EPOCH),
        details: None,
        acts: test::list([]),
    };
    for (key, value) in [
        (ObjectKey::Question { task: 9, message: 1 }, ReadValue::Question(Box::new(question))),
        (
            ObjectKey::Escalation(9),
            ReadValue::OwnerEscalation(Box::new(OwnerEscalation { acts: held.acts.clone(), task: held })),
        ),
        (ObjectKey::PersonTask(10), ReadValue::PersonTask(Box::new(person))),
        (ObjectKey::Result(11), ReadValue::Result(Box::new(result))),
    ] {
        world
            .front()
            .engine
            .act(Time::ZERO, Wall::EPOCH, Change::Object { project: 1, key, value })
            .expect("golden object fits");
    }
    ConversationLine {
        runtime: Some(Runtime::Idle),
        delegates: 0,
        budget: task.budget,
        chip: task.chip,
        project: 1,
        first_words: DocumentText::complete(text(b"Hello")),
        unread: Some(Count { value: 1, limited: false }),
        cached_replies: 1,
        seen: 0,
        last_reply: 1,
        changed: Some(Wall::EPOCH),
    }
}
fn rows(world: &mut World<Test, Scripted>, reply: &ConversationLine) {
    for group in [InboxReadGroup::Needs, InboxReadGroup::Updates] {
        let rows = [
            (InboxGroup::Questions, ObjectKey::Question { task: 9, message: 1 }),
            (InboxGroup::Proposals, ObjectKey::Proposal(super::proposal().key)),
            (InboxGroup::Escalations, ObjectKey::Escalation(9)),
            (InboxGroup::PersonTasks, ObjectKey::PersonTask(10)),
            (InboxGroup::Results, ObjectKey::Result(11)),
            (InboxGroup::Replies, ObjectKey::Task(9)),
        ]
        .into_iter()
        .enumerate()
        .filter(|(_, (kind, _))| InboxReadGroup::of(*kind) == group)
        .map(|(n, (kind, object))| InboxEntry {
            reply: (kind == InboxGroup::Replies).then(|| Box::new(reply.clone())),
            lineage: test::list([]),
            goal: None,
            recipient: Some(Recipient::Party(7)),
            stall: None,
            object,
            project: 1,
            group: kind,
            position: u64::try_from(n).expect("six rows") + 1,
            at: Wall::EPOCH,
        })
        .collect();
        world
            .front()
            .engine
            .act(
                Time::ZERO,
                Wall::EPOCH,
                Change::Source(Source {
                    party: 7,
                    read: Read::Inbox { group, project: None, kind: None, goal: None, after: None },
                    rows: Rows::Inbox(rows),
                }),
            )
            .expect("golden rows fit");
    }
}
