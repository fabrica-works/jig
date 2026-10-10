use super::{
    objects::{authority, chip, escalation, live, person, proposal, text},
    spine::{LIMITS, drive},
};
use crate::{
    Domain, Event, Request,
    action::Act,
    acts,
    confirm::Missing,
    contract::*,
    fields::Purpose,
    objects::{Body, Card, Object, Source},
};
use skein_lib::{Id, List, Queue, Time, Wall};
fn offered(act: ActKind, admission: Admission) -> List<Offered> {
    let mut rows = List::with_capacity(1);
    rows.push(Offered { act, admission }).unwrap();
    rows
}
fn install(domain: &mut Domain, body: Body) -> Id<Object> {
    let key = body.key();
    let id = domain.objects.name(key, Source::Window { generation: 0, number: 4 }).unwrap();
    assert!(domain.objects.put(key, body));
    domain.objects.get_mut(id).unwrap().project = Some(1);
    id
}
fn intend(domain: &mut Domain, id: Id<Object>, act: ActKind) -> Box<[Request]> {
    drive(domain, Time::ZERO, Event::Act { act: Act::Intend { object: id.token(), act } })
}
pub(super) fn change() -> Amendment {
    Amendment {
        addressed: None,
        spec: Some(specification(text(b"amended")).specification),
        wake: None,
        dependencies: None,
        authority: None,
        instructions: None,
    }
}
fn counts() -> PhaseCounts {
    PhaseCounts { waiting: 0, running: 0, held: 0, closing: 0, done: 0, failed: 0, cancelled: 0 }
}
pub(super) fn task() -> Task {
    Task {
        chip: chip(8),
        goal: false,
        priority: None,
        lineage_chips: List::with_capacity(0),
        inputs: List::with_capacity(0),
        project: 1,
        requester: Actor::Party(7),
        lineage: List::with_capacity(0),
        spec: specification(text(b"spec")),
        spec_revision: Revision { value: 9 },
        instructions: text(b"instructions"),
        wake: WakePolicy {
            words: true,
            results: true,
            topics: List::with_capacity(0),
            subscriptions: List::with_capacity(0),
        },
        result_contract: ResultContract::Report,
        dependencies: List::with_capacity(0),
        references: List::with_capacity(0),
        resources: List::with_capacity(0),
        budget: proposal().funding,
        authority: authority(),
        hold: Some(escalation().hold),
        escalation: None,
        proposals: List::with_capacity(0),
        activity: 0,
        deadline: None,
        changed: Wall::EPOCH,
        acts: List::with_capacity(0),
        parts: List::with_capacity(0),
        closing: None,
        spend: List::with_capacity(0),
        result: None,
        runtime: Runtime::Held,
        impact: Impact {
            tasks: counts(),
            live_runs: 0,
            proposals: 0,
            effects_pending: 0,
            effects_settling: 0,
            takeover_budget: 0,
        },
        writer_slots: List::with_capacity(0),
        delegates: List::with_capacity(0),
    }
}
fn sent(rows: &[Request]) -> &Ask {
    let Request::Send { ask, .. } = &rows[0] else {
        panic!("one request");
    };
    assert_eq!(rows.len(), 1);
    ask
}
fn edit(domain: &mut Domain, field: Purpose, bytes: &[u8]) {
    assert!(drive(domain, Time::ZERO, Event::Act { act: Act::Edit { field, text: text(bytes) } }).is_empty());
}
pub(super) fn reread(domain: &mut Domain, body: Body, what: Read) {
    let mut out = Queue::with_capacity(crate::max_out(&LIMITS));
    let read = domain.reads.ask(0, Source::Window { generation: 0, number: 4 }, what, &mut out).unwrap();
    let value = match body {
        Body::Escalation(document) => ReadValue::Escalation(document),
        Body::Task(document) => ReadValue::Task(document),
        Body::PersonTask(document) => ReadValue::PersonTask(document),
        Body::Proposal(_) | Body::Question(_) | Body::Result(_) | Body::Chip(_) | Body::Note { .. } => unreachable!(),
    };
    drive(domain, Time::ZERO, Event::Read { read: read.token(), result: ReadResult::Value(value) });
}

#[test]
fn each_decision_sends_only_its_recalled_revision_and_double_confirmation_sends_once() {
    for case in 0_u8..5 {
        let (mut domain, _) = live();
        let (body, act) = match case {
            0 => {
                let mut row = escalation();
                row.acts = offered(ActKind::Release, Admission::Allowed);
                (Body::Escalation(Box::new(row)), ActKind::Release)
            }
            1 => {
                let mut row = task();
                row.acts = offered(ActKind::Amend, Admission::Allowed);
                (Body::Task(Box::new(row)), ActKind::Amend)
            }
            2 => {
                let mut row = person();
                row.acts = offered(ActKind::Answer, Admission::Allowed);
                (Body::PersonTask(Box::new(row)), ActKind::Answer)
            }
            3 => {
                let mut row = proposal();
                row.acts = offered(ActKind::Accept, Admission::Allowed);
                (Body::Proposal(Box::new(row)), ActKind::Accept)
            }
            4 => {
                let mut row = task();
                row.acts = offered(ActKind::Stop, Admission::Allowed);
                (Body::Task(Box::new(row)), ActKind::Stop)
            }
            _ => unreachable!(),
        };
        let id = install(&mut domain, body);
        assert!(intend(&mut domain, id, act).is_empty());
        assert_eq!(
            domain.objects.get(id).unwrap().card,
            Card::Deciding { request: None, progress: crate::intents::Progress::Confirming }
        );
        match case {
            1 => {
                drive(&mut domain, Time::ZERO, Event::Act { act: Act::Change { change: Box::new(change()) } });
            }
            2 => edit(&mut domain, Purpose::Answer, b"the answer"),
            0 | 3 | 4 => {}
            _ => unreachable!(),
        }
        let rows = drive(&mut domain, Time::ZERO, Event::Act { act: Act::Confirm });
        let expected = match case {
            0 => Ask::Release { project: 1, task: 8, revision: Revision { value: 3 }, again: None },
            1 => Ask::Amend { project: 1, task: 8, revision: Revision { value: 9 }, change: change(), release: None },
            2 => Ask::AnswerPerson {
                project: 1,
                task: 8,
                revision: Revision { value: 4 },
                answer: PersonAnswer::Words(text(b"the answer")),
            },
            3 => Ask::DecideProposal { project: 1, proposal: proposal().key, decision: Decision::Accepted },
            4 => Ask::Stop { project: 1, task: 8 },
            _ => unreachable!(),
        };
        assert_eq!(sent(&rows), &expected);
        assert!(drive(&mut domain, Time::ZERO, Event::Act { act: Act::Confirm }).is_empty());
        assert!(domain.confirmation().is_none());
        if case == 2 {
            assert!(domain.fields().get(Purpose::Answer).unwrap().text.bytes().is_empty());
            assert_eq!(domain.fields().get(Purpose::Answer).unwrap().writes, 1);
        }
    }
    note_decisions_name_the_revision();
}

fn note_decisions_name_the_revision() {
    let mut note = Note {
        number: 6,
        revision: Revision { value: 11 },
        scope: NoteScope::Project(1),
        description: text(b"before"),
        body: text(b"body"),
        by: Actor::Party(7),
        tasks: List::with_capacity(0),
        changed: Wall::EPOCH,
        acts: List::with_capacity(2),
    };
    note.acts.push(Offered { act: ActKind::EditNote, admission: Admission::Allowed }).unwrap();
    note.acts.push(Offered { act: ActKind::DeleteNote, admission: Admission::Allowed }).unwrap();
    assert_eq!(
        acts::note_ask(1, &note, ActKind::EditNote, text(b"after"), text(b"new")),
        Some(Ask::EditNote {
            project: 1,
            note: 6,
            revision: Revision { value: 11 },
            description: text(b"after"),
            body: text(b"new")
        })
    );
    assert_eq!(
        acts::note_ask(1, &note, ActKind::DeleteNote, text(b""), text(b"")),
        Some(Ask::DeleteNote { project: 1, note: 6, revision: Revision { value: 11 } })
    );
    for act in [ActKind::EditNote, ActKind::DeleteNote] {
        let (mut domain, _) = live();
        let id = domain.objects.note(1, Source::Window { generation: 0, number: 4 }, note.clone()).unwrap();
        assert!(intend(&mut domain, id, act).is_empty());
        assert_eq!(domain.confirmation().unwrap().revision, Some(Revision { value: 11 }));
        let expected = if act == ActKind::EditNote {
            edit(&mut domain, Purpose::NoteDescription, b"after");
            edit(&mut domain, Purpose::NoteBody, b"new");
            Ask::EditNote {
                project: 1,
                note: 6,
                revision: Revision { value: 11 },
                description: text(b"after"),
                body: text(b"new"),
            }
        } else {
            Ask::DeleteNote { project: 1, note: 6, revision: Revision { value: 11 } }
        };
        assert_eq!(sent(&drive(&mut domain, Time::ZERO, Event::Act { act: Act::Confirm })), &expected);
        assert!(domain.confirmation().is_none());
        if act == ActKind::EditNote {
            assert!(domain.fields().get(Purpose::NoteDescription).unwrap().text.bytes().is_empty());
            assert!(domain.fields().get(Purpose::NoteBody).unwrap().text.bytes().is_empty());
        }
    }
}

#[test]
fn a_changed_revision_refuses_in_the_confirmation_while_spend_changes_only_its_words() {
    let (mut domain, _) = live();
    let mut row = escalation();
    row.acts = offered(ActKind::Release, Admission::Allowed);
    let id = install(&mut domain, Body::Escalation(Box::new(row.clone())));
    intend(&mut domain, id, ActKind::Release);
    row.hold.revision = Revision { value: 8 };
    reread(&mut domain, Body::Escalation(Box::new(row)), Read::Escalation { task: 8 });
    assert!(domain.confirmation().unwrap().changed);
    assert_eq!(domain.confirmation().unwrap().missing, Some(Missing::Changed { now: Some(Revision { value: 8 }) }));
    assert!(drive(&mut domain, Time::ZERO, Event::Act { act: Act::Confirm }).is_empty());
    drive(&mut domain, Time::ZERO, Event::Act { act: Act::Dismiss });
    domain.objects.clear();
    domain.objects.reclaim();
    domain.reads.reclaim();
    let mut row = task();
    row.acts = offered(ActKind::Amend, Admission::Proposed { to: Recipient::Party(9) });
    let id = install(&mut domain, Body::Task(Box::new(row.clone())));
    intend(&mut domain, id, ActKind::Amend);
    drive(&mut domain, Time::ZERO, Event::Act { act: Act::Change { change: Box::new(change()) } });
    row.budget.own = 5;
    reread(&mut domain, Body::Task(Box::new(row)), Read::Task { task: 8 });
    let open = domain.confirmation().unwrap();
    assert!(!open.changed);
    assert_eq!(open.proposed, Some(Recipient::Party(9)));
    assert!(open.missing.is_none());
    assert_eq!(
        sent(&drive(&mut domain, Time::ZERO, Event::Act { act: Act::Confirm })),
        &Ask::Amend { project: 1, task: 8, revision: Revision { value: 9 }, change: change(), release: None }
    );
}

#[test]
fn only_engine_offers_open_and_new_confirmations_or_dismissals_clear_the_old_fields() {
    let (mut domain, _) = live();
    let mut row = proposal();
    row.acts = offered(ActKind::Reject, Admission::Allowed);
    let id = install(&mut domain, Body::Proposal(Box::new(row.clone())));
    assert!(intend(&mut domain, id, ActKind::Accept).is_empty());
    assert!(domain.confirmation().is_none());
    assert_eq!(domain.objects.get(id).unwrap().card, Card::Refused { refusal: Refusal::NotOffered });
    intend(&mut domain, id, ActKind::Reject);
    assert!(domain.confirmation().unwrap().dangerous);
    assert_eq!(domain.confirmation().unwrap().missing, Some(Missing::Words { field: Purpose::Reason }));
    edit(&mut domain, Purpose::Reason, b"reason");
    drive(&mut domain, Time::ZERO, Event::Act { act: Act::Dismiss });
    assert!(domain.fields().get(Purpose::Reason).unwrap().text.bytes().is_empty());
    assert_eq!(domain.fields().get(Purpose::Reason).unwrap().writes, 1);
    intend(&mut domain, id, ActKind::Reject);
    edit(&mut domain, Purpose::Reason, b"new reason");
    // A new act closes the old confirmation even when the new act is not offered.
    intend(&mut domain, id, ActKind::Accept);
    assert!(domain.confirmation().is_none());
    assert!(domain.fields().get(Purpose::Reason).unwrap().text.bytes().is_empty());
    assert_eq!(domain.fields().get(Purpose::Reason).unwrap().writes, 2);
}

#[test]
fn bounded_amendment_confirms_supplied_budget_and_grants_and_refreshes_a_moved_offer() {
    bounded_amendment_dimensions();
    for act in [ActKind::Amend, ActKind::AmendRelease] {
        let (mut domain, _) = live();
        let mut row = task();
        let grant = Grant { connector: 1, kind: 2, pattern: text(b"listed") };
        let mut grants = List::with_capacity(1);
        grants.push(grant.clone()).unwrap();
        row.acts = offered(
            act,
            Admission::Bounded {
                within: Bounds::Amend {
                    budget_left: 5,
                    grants: grants.clone(),
                    authority: Box::new({
                        let mut allowed = row.authority.clone();
                        allowed.budget = 15;
                        allowed.grants = grants.clone();
                        allowed
                    }),
                    ceiling: Box::new({
                        let mut maximum = row.authority.clone();
                        maximum.budget = 100;
                        maximum.grants = grants;
                        maximum
                    }),
                },
                beyond: Beyond::Proposed { to: Recipient::Party(9) },
            },
        );
        let id = install(&mut domain, Body::Task(Box::new(row.clone())));
        intend(&mut domain, id, act);
        assert_eq!(domain.confirmation().unwrap().missing, Some(Missing::Change));
        let mut draft = change();
        let mut after = row.authority.clone();
        after.budget = 15;
        after.grants = List::with_capacity(1);
        after.grants.push(grant).unwrap();
        draft.authority = Some(after);
        drive(&mut domain, Time::ZERO, Event::Act { act: Act::Change { change: Box::new(draft.clone()) } });
        assert!(domain.confirmation().unwrap().missing.is_none());
        assert_eq!(domain.confirmation().unwrap().proposed, None);
        draft.authority.as_mut().unwrap().budget = 16;
        drive(&mut domain, Time::ZERO, Event::Act { act: Act::Change { change: Box::new(draft.clone()) } });
        assert_eq!(domain.confirmation().unwrap().proposed, Some(Recipient::Party(9)));
        row.acts = offered(
            act,
            Admission::Bounded {
                within: Bounds::Amend {
                    budget_left: 5,
                    grants: List::with_capacity(0),
                    authority: Box::new(row.authority.clone()),
                    ceiling: Box::new({
                        let mut maximum = row.authority.clone();
                        maximum.budget = 100;
                        maximum.grants = draft.authority.as_ref().unwrap().grants.clone();
                        maximum
                    }),
                },
                beyond: Beyond::Refused(Refusal::Role { held: Some(HeldRole::Policy { role: 2 }) }),
            },
        );
        reread(&mut domain, Body::Task(Box::new(row.clone())), Read::Task { task: 8 });
        assert_eq!(
            domain.confirmation().unwrap().missing,
            Some(Missing::Refused { why: Refusal::Role { held: Some(HeldRole::Policy { role: 2 }) } })
        );
        assert!(drive(&mut domain, Time::ZERO, Event::Act { act: Act::Confirm }).is_empty());
        draft.authority = None;
        drive(&mut domain, Time::ZERO, Event::Act { act: Act::Change { change: Box::new(draft.clone()) } });
        assert_eq!(domain.confirmation().unwrap().proposed, None);
        assert_eq!(
            sent(&drive(&mut domain, Time::ZERO, Event::Act { act: Act::Confirm })),
            &Ask::Amend {
                project: 1,
                task: 8,
                revision: row.spec_revision,
                change: draft,
                release: if act == ActKind::AmendRelease { Some(row.hold.as_ref().unwrap().revision) } else { None }
            }
        );
    }
}

fn bounded_amendment_dimensions() {
    let before = authority();
    let mut facts = before.clone();
    facts.budget = 20;
    facts.depth = 3;
    facts.tasks = 5;
    facts.deadline = Some(Wall::from_nanos(100));
    facts.tools = List::with_capacity(1);
    facts.tools.push(7).unwrap();
    facts.task_kinds = List::with_capacity(1);
    facts.task_kinds.push(Executor::Party { party: 4 }).unwrap();
    facts.note_scopes = List::with_capacity(1);
    facts.note_scopes.push(NoteScope::Project(1)).unwrap();
    let mut maximum = facts.clone();
    maximum.tasks = 6;
    maximum.depth = 4;
    maximum.budget = 100;
    maximum.deadline = None;
    let offer = Admission::Bounded {
        within: Bounds::Amend {
            budget_left: 10,
            grants: List::with_capacity(0),
            authority: Box::new(facts.clone()),
            ceiling: Box::new(maximum),
        },
        beyond: Beyond::Proposed { to: Recipient::Party(9) },
    };
    let mut draft = change();
    draft.authority = Some(facts.clone());
    assert_eq!(admission::amendment(&offer, &draft, &before), Admission::Allowed);
    for dimension in 0_u8..7 {
        let mut outside = facts.clone();
        match dimension {
            0 => outside.budget = 21,
            1 => outside.depth = 4,
            2 => outside.tasks = 6,
            3 => outside.deadline = Some(Wall::from_nanos(101)),
            4 => *outside.tools.get_mut(0).unwrap() = 8,
            5 => *outside.task_kinds.get_mut(0).unwrap() = Executor::Party { party: 5 },
            6 => *outside.note_scopes.get_mut(0).unwrap() = NoteScope::Project(2),
            _ => unreachable!(),
        }
        draft.authority = Some(outside);
        let prediction = admission::amendment(&offer, &draft, &before);
        assert_eq!(
            prediction,
            if dimension < 4 {
                Admission::Proposed { to: Recipient::Party(9) }
            } else {
                Admission::Refused(Refusal::NotOffered)
            }
        );
    }
    // Current task facts may already be wider than the role; narrowing remains direct.
    let mut existing = facts;
    existing.depth = 9;
    let mut narrower = existing.clone();
    narrower.budget = 19;
    draft.authority = Some(narrower);
    assert_eq!(admission::amendment(&offer, &draft, &existing), Admission::Allowed);
}

#[test]
fn bounded_amendment_keeps_every_supplied_policy_recipient_in_order() {
    let (mut domain, _) = live();
    let mut row = task();
    let mut roles = List::with_capacity(2);
    roles.push(7).unwrap();
    roles.push(2).unwrap();
    let recipient = Recipient::Roles { project: 1, roles };
    let mut direct = row.authority.clone();
    direct.budget = 15;
    let mut ceiling = row.authority.clone();
    ceiling.budget = 100;
    row.acts = offered(
        ActKind::Amend,
        Admission::Bounded {
            within: Bounds::Amend {
                budget_left: 5,
                grants: List::with_capacity(0),
                authority: Box::new(direct),
                ceiling: Box::new(ceiling),
            },
            beyond: Beyond::Proposed { to: recipient.clone() },
        },
    );
    let id = install(&mut domain, Body::Task(Box::new(row.clone())));
    intend(&mut domain, id, ActKind::Amend);
    let mut draft = change();
    let mut after = row.authority.clone();
    after.budget = 16;
    draft.authority = Some(after);
    drive(&mut domain, Time::ZERO, Event::Act { act: Act::Change { change: Box::new(draft.clone()) } });
    assert_eq!(domain.confirmation().unwrap().proposed, Some(recipient));
    assert!(domain.confirmation().unwrap().missing.is_none());
    assert_eq!(
        sent(&drive(&mut domain, Time::ZERO, Event::Act { act: Act::Confirm })),
        &Ask::Amend { project: 1, task: 8, revision: row.spec_revision, change: draft, release: None }
    );
}

#[test]
fn person_verdict_choices_preserve_full_codes_and_refuse_missing_machine_fields() {
    for required in [false, true] {
        let (mut domain, _) = live();
        let mut row = person();
        let mut options = List::with_capacity(1);
        let mut fields = List::with_capacity(1);
        if required {
            fields.push(FieldRule { name: b"state".as_slice().into(), max: 8 }).unwrap();
        }
        options
            .push(OptionLine { number: u32::MAX, words: text(b"choice"), words_max: 32, followups: 0, fields })
            .unwrap();
        row.contract = PersonContract::Choice { options };
        row.acts = offered(ActKind::Answer, Admission::Allowed);
        let id = install(&mut domain, Body::PersonTask(Box::new(row)));
        assert!(intend(&mut domain, id, ActKind::Answer).is_empty());
        assert!(drive(&mut domain, Time::ZERO, Event::Act { act: Act::Choose { option: u32::MAX } }).is_empty());
        let rows = drive(&mut domain, Time::ZERO, Event::Act { act: Act::Confirm });
        if required {
            assert!(rows.is_empty(), "machine field choice cannot send an incomplete answer");
            assert_eq!(domain.confirmation().unwrap().missing, Some(Missing::Choice));
        } else {
            assert_eq!(
                sent(&rows),
                &Ask::AnswerPerson {
                    project: 1,
                    task: 8,
                    revision: Revision { value: 4 },
                    answer: PersonAnswer::Choice { option: u32::MAX, reason: text(b"") },
                }
            );
        }
    }
}

#[test]
fn approval_confirmation_keeps_the_whole_state_and_actual_optional_explanation() {
    for (reason, explanation) in [
        (None, None),
        (Some(b"".as_slice()), Some(text(b""))),
        (Some(b"explanation".as_slice()), Some(text(b"explanation"))),
    ] {
        let (mut domain, _) = live();
        let mut row = person();
        row.contract = PersonContract::Approval {
            state: text(&[0, 255, 1]),
            part: Part::Resource { connector: 1, path: text(b"state") },
        };
        row.acts = offered(ActKind::Answer, Admission::Allowed);
        let id = install(&mut domain, Body::PersonTask(Box::new(row)));
        assert!(intend(&mut domain, id, ActKind::Answer).is_empty());
        if let Some(reason) = reason {
            edit(&mut domain, Purpose::Reason, reason);
        }
        assert_eq!(
            sent(&drive(&mut domain, Time::ZERO, Event::Act { act: Act::Confirm })),
            &Ask::AnswerPerson {
                project: 1,
                task: 8,
                revision: Revision { value: 4 },
                answer: PersonAnswer::Approved { state: Box::new([0, 255, 1]), words: explanation },
            }
        );
    }
}

fn specification(words: Text) -> SpecificationDocument {
    SpecificationDocument {
        specification: Specification { words, parameters: List::with_capacity(0), inputs: List::with_capacity(0) },
        cut: false,
    }
}
