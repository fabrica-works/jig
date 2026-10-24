use super::*;

#[test]
fn an_unread_closed_tail_continues_the_brief_without_counting() {
    let (mut core, limits) = crate::tests::core();
    crate::tests::make_task(&mut core, &limits, 1);
    let mut proof = RunProof::claimed(1, 1, 1, None, jig_domain_fleet::HostKind::Worker);
    proof.window = Some(crate::CurrentWindow { attempt: 1, turn: 1, closed: true });
    core.proofs.insert(1, proof).unwrap();
    core.begin_transcript(1, 1);
    core.contexts.insert(1, core.tasks.run_context(1).unwrap()).unwrap();
    core.transcripts.get_mut(&1).unwrap().append_text(Some(b"partial text".as_slice().into()), 128);
    let mut work = Queue::with_capacity(2);
    let _requests = transcript_unread(&mut core, &mut work, 1);
    let Some(Event::StartBrief { task }) = work.pop() else { panic!("closed tail continues the brief") };
    assert_eq!(task, 1);
    assert!(work.is_empty(), "no PreparationUnread is counted");
    assert!(core.contexts.contains_key(&1));
    let transcript = core.transcripts.get(&1).unwrap();
    assert!(transcript.closed);
    assert!(transcript.last.is_none());
    assert!(transcript.texts.is_empty());
}
