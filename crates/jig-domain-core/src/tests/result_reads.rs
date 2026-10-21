use super::*;

const PREPARING: u64 = 1;
const DEPENDENCY: u64 = 12;
const INPUT: u64 = 13;
const CHARTER: u32 = 1;
const TEXT_BYTES: u32 = 256; // Both fixture results and their added line fit whole.
const TEXT_LIMITS: BriefTextLimits = BriefTextLimits { parts: 1, read_bytes: TEXT_BYTES, brief_bytes: TEXT_BYTES };

fn preparing() -> (Core, Limits) {
    let (mut core, limits) = make_tree();
    send_task(
        &mut core,
        &limits,
        tasks::Event::Prepare { reply_to: ReplyTo::new(Token::new(PREPARING)), task: PREPARING },
    );
    let mut context = core.tasks.run_context(PREPARING).unwrap();
    context.dependencies = Box::new([DEPENDENCY]);
    context.spec.inputs = Box::new([INPUT]);
    core.contexts.insert(PREPARING, context).unwrap();
    (core, limits)
}

pub(super) fn requested(out: &[Request], wanted: u64) -> Token {
    for request in out {
        if let Request::Held(held) = request
            && let Held::Load(now) = held.as_ref()
            && let Now::EndedTask { request, task } = now.as_ref()
        {
            assert_eq!(*task, wanted);
            return *request;
        }
    }
    panic!("ended result requested");
}

fn start(core: &mut Core, limits: &Limits) -> Token {
    requested(&watch_outputs(core, limits, Event::StartBrief { task: PREPARING }), DEPENDENCY)
}

fn ended(number: u64) -> Box<tasks::TaskRecord> {
    Box::new(fixtures::ended_report(number, tasks::Party::Person(1)))
}

fn answer(core: &mut Core, limits: &Limits, request: Token, row: Option<Box<tasks::TaskRecord>>) -> Box<[Request]> {
    watch_outputs(core, limits, Event::EndedTaskLoaded { request, row, busy: false })
}

fn only_decided(out: &[Request]) {
    for request in out {
        let Request::Decided = request else { panic!("unexpected output: {request:?}") };
    }
}

#[test]
fn dependencies_and_inputs_load_in_order_before_the_notes() {
    let (mut core, limits) = preparing();
    let request = start(&mut core, &limits);
    assert!(core.note_routes.is_empty());
    assert!(core.dependency_results.is_empty());
    let out = answer(&mut core, &limits, request, Some(ended(DEPENDENCY)));
    assert_eq!(requested(&out, INPUT), request);
    assert!(core.note_routes.is_empty());
    assert!(core.dependency_results.is_empty());
    let out = answer(&mut core, &limits, request, Some(ended(INPUT)));
    assert!(core.result_reads.is_empty());
    assert_eq!(
        read_brief_part(&core, TEXT_LIMITS, PREPARING, BriefPart::Dependencies).unwrap().as_ref(),
        b"task 12: report\ndone\n\ntask 13: report\ndone\n\n",
    );
    let mut notes_loaded = false;
    for request in &out {
        if let Request::Held(held) = request
            && let Held::NotesLoad { .. } = held.as_ref()
        {
            notes_loaded = true;
        }
    }
    assert!(notes_loaded);
}

#[test]
fn abandoning_preparation_drops_its_read_and_ignores_late_answers() {
    let (mut core, limits) = preparing();
    let request = start(&mut core, &limits);
    watch_outputs(&mut core, &limits, Event::Tasks(tasks::Event::Hold { task: PREPARING, why: tasks::Hold::Budget }));
    assert!(core.result_reads.is_empty());
    let phase = core.tasks.task(PREPARING).unwrap().phase.clone();
    let out = answer(&mut core, &limits, request, None);
    only_decided(&out);
    assert_eq!(core.tasks.task(PREPARING).unwrap().phase, phase);
    assert!(core.result_reads.is_empty());
    assert!(core.dependency_results.is_empty());
    assert!(core.contexts.is_empty());
    assert!(core.note_routes.is_empty());
}

#[test]
fn a_reply_for_a_changed_preparation_is_silently_dropped() {
    let (mut core, limits) = preparing();
    let request = start(&mut core, &limits);
    core.contexts.get_mut(&PREPARING).unwrap().last_message = 1;
    let out = answer(&mut core, &limits, request, None);
    only_decided(&out);
    assert_eq!(core.tasks.task(PREPARING).unwrap().phase, tasks::Phase::Active(tasks::Active::Preparing));
    assert!(core.result_reads.is_empty());
    assert!(core.dependency_results.is_empty());
    assert!(core.note_routes.is_empty());
}

#[test]
fn unavailable_or_rejected_results_back_off_without_spending_attempts() {
    let mut wrong_project = ended(DEPENDENCY);
    wrong_project.project = 2;
    let mut live = ended(DEPENDENCY);
    live.phase = tasks::Phase::Active(tasks::Active::Due);
    for (label, row, busy) in [
        ("missing", None, false),
        ("busy", Some(ended(DEPENDENCY)), true),
        ("wrong task", Some(ended(INPUT)), false),
        ("wrong project", Some(wrong_project), false),
        ("live task", Some(live), false),
    ] {
        let (mut core, limits) = preparing();
        let request = start(&mut core, &limits);
        let out = watch_outputs(&mut core, &limits, Event::EndedTaskLoaded { request, row, busy });
        let row = core.tasks.task(PREPARING).unwrap();
        let tasks::Phase::Active(tasks::Active::BackingOff { .. }) = row.phase else {
            panic!("{label}: preparation did not back off");
        };
        assert_eq!(row.attempt, 0, "{label}");
        assert_eq!(row.tries, tasks::Tries::NONE, "{label}");
        assert!(core.result_reads.is_empty(), "{label}");
        assert!(core.contexts.is_empty(), "{label}");
        assert!(core.dependency_results.is_empty(), "{label}");
        assert!(core.note_routes.is_empty(), "{label}");
        for request in &out {
            if let Request::Now(_) = request {
                panic!("{label}: unexpected immediate output: {request:?}");
            }
        }
    }
}

#[test]
fn added_dependency_lines_name_the_time_party_and_outcome() {
    let (mut core, _) = make_tree();
    for (party, kind, expected) in [
        (
            tasks::Party::Task(4),
            tasks::ResultKind::Report,
            b"waited for task 12, added 99 by task 4: done\ntask 12: report\nresult\n\n".as_slice(),
        ),
        (
            tasks::Party::Person(5),
            tasks::ResultKind::Failed,
            b"waited for task 12, added 99 by person 5: failed\ntask 12: failed\nresult\n\n".as_slice(),
        ),
        (
            tasks::Party::Deployment { project: 6 },
            tasks::ResultKind::Cancelled,
            b"waited for task 12, added 99 by deployment for project 6: cancelled\ntask 12: cancelled\nresult\n\n"
                .as_slice(),
        ),
    ] {
        let mut context = charters::context(PREPARING, CHARTER);
        context.added = Box::new([tasks::Added { task: DEPENDENCY, at: Wall::from_nanos(99), by: party }]);
        core.contexts.insert(PREPARING, context).unwrap();
        core.dependency_results
            .insert(
                PREPARING,
                Box::new([HistoricalResult { task: DEPENDENCY, kind, words: b"result".as_slice().into() }]),
            )
            .unwrap();
        assert_eq!(read_brief_part(&core, TEXT_LIMITS, PREPARING, BriefPart::Dependencies).unwrap().as_ref(), expected);
    }
}
