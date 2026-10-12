//! Single-read positions remain bounded through lifecycle and restart.
use super::*;

#[test]
fn a_held_task_names_the_milestone_of_its_hold() {
    let mut h = Harness::new();
    h.make(vec![task(1, &[])]);
    assert_eq!(live(&h, 1).hold, 0);
    h.send(Event::Hold { task: 1, why: Hold::Stopped });
    assert_eq!(live(&h, 1).hold, 3);
    h.route(1, EscalationHolder::Person(9), 1);
    h.restart();
    assert_eq!(live(&h, 1).hold, 3);
    h.send(Event::Control {
        reply_to: to(),
        by: Party::Person(9),
        task: 1,
        control: Control::Release { again: None, message: None },
    });
    assert_eq!(live(&h, 1).hold, 3);
    h.send(Event::Hold { task: 1, why: Hold::Budget });
    assert_eq!(live(&h, 1).hold, 5);
    let Stored::Milestone(row) = &h.stored[&Key::Milestone { task: 1, position: 5 }] else { panic!() };
    let crate::Phase::Held { why, .. } = &row.phase else { panic!("held milestone") };
    assert_eq!(*why, Hold::Budget);
}

#[test]
fn question_lines_keep_the_newest_within_inbox_messages() {
    let mut h = question_cohort(5);
    h.env.limits.inbox_messages = 2;
    h.env.limits.run_inbox = 2;
    h.send(Event::Prepare { reply_to: to(), task: 2 });
    h.send(Event::Claim { reply_to: to(), task: 2, attempt: 1, resumed: false, budget: 10, writes: Box::new([]) });
    h.send(Event::Started { task: 2, attempt: 1 });
    question_cycles(&mut h);
    assert_eq!(
        live(&h, 2).question_lines.as_ref(),
        [
            crate::QuestionLine { message: 5, line: 7, answer: None },
            crate::QuestionLine { message: 3, line: 5, answer: None },
        ]
    );
    h.env.limits = LIMITS;
    h.restart();
    assert_eq!(live(&h, 2).question_lines.len(), 2);
}

fn question_cycles(h: &mut Harness) {
    for (turn, source) in (3..=5).enumerate() {
        let message = u64::try_from(turn).unwrap().checked_mul(2).unwrap().checked_add(1).unwrap();
        let out = word(h, 2, Party::Task(source), message, MessageKind::Question);
        assert!(
            out.iter().any(|request| match request {
                Request::Sent { .. } => true,
                _ => false,
            }),
            "{out:?}"
        );
        h.send(Event::Turn {
            reply_to: to(),
            task: 2,
            attempt: 1,
            turn: u32::try_from(turn).unwrap().checked_add(1).unwrap(),
            read: Some(message),
            offered: Some(message),
            cumulative: 0,
        });
    }
}

fn question_cohort(last: u64) -> Harness {
    let mut h = Harness::new();
    let mut parent = task(1, &[]);
    parent.numbers.budget = 1000;
    parent.authority.budget.spend = 1000;
    h.make(vec![parent]);
    let children = (2..=last)
        .map(|number| {
            let mut child = task(number, &[]);
            child.funder = Funder::Task(1);
            child
        })
        .collect::<Vec<_>>();
    let out = h.send(Event::Make { reply_to: to(), creator: Party::Task(1), batch: children.into_boxed_slice() });
    assert!(
        out.iter().any(|request| match request {
            Request::Made { .. } => true,
            _ => false,
        }),
        "{out:?}"
    );
    for right in 3..=last {
        let out = h.send(Event::Introduce { reply_to: to(), by: 1, left: 2, right });
        assert!(
            out.iter().any(|request| match request {
                Request::Done { .. } => true,
                _ => false,
            }),
            "{out:?}"
        );
    }
    h
}

fn live(h: &Harness, number: u64) -> &crate::TaskRecord {
    h.domain.task(number).expect("live task")
}

fn word(h: &mut Harness, task: u64, from: Party, number: u64, kind: MessageKind) -> Vec<Request> {
    h.send(Event::Message {
        reply_to: to(),
        project: 1,
        task,
        release: false,
        word: crate::Word { number, from, kind, words: Box::new([1]), at: Wall::EPOCH, hits: 1, eligible: true },
    })
}

#[test]
fn a_tasks_answer_names_the_line_in_the_delegates_activity() {
    let mut h = question_cohort(2);
    let _question = word(&mut h, 1, Party::Task(2), 1, MessageKind::Question);
    let _answer = word(&mut h, 2, Party::Task(1), 2, MessageKind::Answer { question: 1 });
    let expected = Some(crate::AnswerLine { task: 2, position: 2 });
    assert_eq!(live(&h, 1).question_lines[0].answer, expected);
    assert_eq!(live(&h, 1).activity, 2, "the answering task gains no line");
    let Stored::Live(saved) = &h.stored[&Key::Live(1)] else { panic!("answer pointer saved") };
    assert_eq!(saved.question_lines[0].answer, expected);
    let Stored::Activity(line) = &h.stored[&Key::Activity { task: 2, position: 2 }] else { panic!("answer line") };
    let crate::Line::Words { kind, .. } = &line.line else { panic!("answer words") };
    assert_eq!(*kind, MessageKind::Answer { question: 1 });
    let original = live(&h, 1).clone();
    let _again = word(&mut h, 2, Party::Task(1), 3, MessageKind::Answer { question: 1 });
    assert_eq!(live(&h, 1), &original, "the first answer stays");
    h.restart();
    assert_eq!(live(&h, 1).question_lines[0].answer, expected);
}
