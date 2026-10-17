//! Operation fixtures shared by the scripted engine tests.
use super::*;
pub(super) fn proposed_amendment(bounded: bool, changed: bool) -> (Script, Ask, Answer) {
    let mut setup = live_script();
    let mut row = task();
    row.acts = list(vec![Offered {
        act: ActKind::Amend,
        admission: if bounded {
            Admission::Bounded {
                within: jig_client_domain::contract::Bounds::Amend {
                    budget_left: 10,
                    grants: List::with_capacity(0),
                    implications: List::with_capacity(0),
                    authority: Box::new(row.authority.clone()),
                    ceiling: Box::new(row.authority.clone()),
                },
                beyond: Beyond::Proposed { to: Recipient::Party(9) },
            }
        } else {
            Admission::Proposed { to: Recipient::Party(9) }
        },
    }]);
    let ask = Ask::Propose {
        project: 1,
        action: ProposeAction::Amendment {
            task: row.chip.task,
            revision: row.spec_revision,
            change: Box::new(Amendment {
                addressed: None,
                spec: None,
                wake: None,
                dependencies: None,
                authority: None,
                instructions: Some(text(b"new")),
                tracked: None,
            }),
        },
        reason: text(b"more work"),
    };
    let answer = Answer::Done(Outcome::Owner(if changed {
        OwnerOutcome::Amended { task: row.chip.task }
    } else {
        OwnerOutcome::Proposed { proposal: 1 }
    }));
    setup.truth.objects.insert(ObjectKey::Task(row.chip.task), ReadValue::Task(Box::new(row)));
    setup.rules.push(Rule { party: Some(1), ask: ask.clone(), answer: answer.clone(), changes: vec![] });
    (setup, ask, answer)
}
