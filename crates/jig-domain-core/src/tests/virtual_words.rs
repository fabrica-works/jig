//! A run woken by a word that never enters its inbox, a held delegate's escalation,
//! prepares as current (domain/engine.md, 7).
use super::*;

const REQUESTER: u64 = 1;
const HELD: u64 = 3;
const ENTRY: u64 = 1;

#[test]
fn a_held_delegates_escalation_prepares_its_requesters_next_run() {
    let (mut core, limits) = make_tree();
    let revision = core.tasks.task(HELD).unwrap().revisions.hold;
    send_task(
        &mut core,
        &limits,
        tasks::Event::RoutedEscalation {
            task: HELD,
            revision,
            holder: tasks::EscalationHolder::Task(REQUESTER),
            entry: ENTRY,
        },
    );
    let context = core.tasks.run_context(REQUESTER).unwrap();
    // The escalation is the context's newest word, never the row's.
    assert_eq!(context.last_message, ENTRY);
    assert_eq!(core.tasks.task(REQUESTER).unwrap().last_message, 0);
    assert_eq!(core.tasks.run_message(REQUESTER), Some(ENTRY));
    let _out =
        watch_outputs(&mut core, &limits, Event::PreparedAgent { context, transcript_waiter: None, busy: false });
    let _out = watch_outputs(
        &mut core,
        &limits,
        Event::Tasks(tasks::Event::Prepare { reply_to: ReplyTo::new(Token::new(REQUESTER)), task: REQUESTER }),
    );
    let row = core.tasks.task(REQUESTER).unwrap();
    assert_eq!(row.phase, tasks::Phase::Active(tasks::Active::Preparing));
    assert_eq!(row.retry_notice, None);
    assert!(core.contexts.contains_key(&REQUESTER));
    let mut notes = false;
    for request in watch_outputs(&mut core, &limits, Event::StartBrief { task: REQUESTER }) {
        if let Request::Held(held) = request
            && let Held::NotesLoad { .. } = *held
        {
            notes = true;
        }
    }
    assert!(notes, "the brief goes on to its notes");
}
