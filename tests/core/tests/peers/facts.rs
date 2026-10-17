use super::*;
const CHAT: u64 = 1;
const NEXT_RUN: Duration = Duration::from_millis(1);
const QUESTION: [u8; 16] = [83; 16];
#[test]
fn a_scripted_chat_question_waits_for_its_requester() {
    let mut world = scripted(
        831,
        vec![
            Box::new([
                Script::Call(Call::Delegate { words: b"ask the requester".as_slice().into(), budget: 10 }),
                Script::Park,
            ]),
            Box::new([
                Script::Call(Call::Question { target: CHAT, words: b"Which next?".as_slice().into() }),
                Script::Wait,
            ]),
        ],
        false,
    );
    acts(
        &mut world,
        0,
        Box::new([Act::Request { key: QUESTION, ask: Ask::Chat { words: b"ask me".as_slice().into() } }]),
    );
    world.advance(NEXT_RUN);
    let peers = world.peers.as_ref().expect("scripted peers");
    let task = peers.parties[0].last_task.expect("owner's chat");
    let row = world
        .store
        .rows
        .values()
        .find_map(|row| match row {
            root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row))) if row.number == task => Some(row),
            root::Record::Core(_) | root::Record::Connector { .. } => None,
        })
        .expect("live chat");
    assert_eq!(
        row.inbox.iter().filter(|word| word.kind == tasks::MessageKind::Question).count(),
        1,
        "answers: {:?}, assignments: {:?}, phases: {:?}",
        peers.workers[0].call_answers,
        peers.workers[0].seen,
        world
            .store
            .rows
            .values()
            .filter_map(|record| match record {
                root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row))) => Some((row.number, &row.phase)),
                root::Record::Core(_) | root::Record::Connector { .. } => None,
            })
            .collect::<Vec<_>>()
    );
}
