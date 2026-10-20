use jig_client::{Asker, Key, contract::*};
use jig_client_test_kit::check::{Check, Delivered, Fault, Limits, Object};
use skein_lib::{List, Queue};
use skein_ui::{NodeId, Role, Run, View};
const KEY: Key = Key { asker: Asker::JIG, number: 7 };
const NOTE: u64 = 9;
const PROJECT: u32 = 1;
const TASK: u64 = 12;
const OTHER_TASK: u64 = 13;
const SPEC: Revision = Revision { value: 2 };
const PERSON: Revision = Revision { value: 3 };
const REVISION: Revision = Revision { value: 4 };
const WRONG: Revision = Revision { value: 99 };
const AUTHOR: u64 = 10;
fn check() -> Check {
    Check::new(&Limits { objects: 2, revisions: 2 })
}
fn text(bytes: &[u8]) -> Text {
    Text::new(bytes.into(), 32).expect("fixture text fits")
}
fn note(revision: Revision) -> Note {
    Note {
        number: NOTE,
        revision,
        scope: NoteScope::Project(PROJECT),
        description: DocumentText::complete(text(b"Note")),
        body: DocumentText::complete(text(b"Body")),
        writer: NoteAuthor::Party { party: AUTHOR },
        tasks: List::with_capacity(0),
        changed: None,
        acts: List::with_capacity(0),
        cut: false,
    }
}
fn delete(revision: Revision) -> Ask {
    Ask::DeleteNote { project: PROJECT, note: NOTE, scope: NoteScope::Project(PROJECT), revision }
}
fn undelivered(revision: Revision) -> Fault {
    Fault::Undelivered { key: KEY, object: Object::Note { scope: NoteScope::Project(PROJECT), note: NOTE }, revision }
}
#[test]
fn a_decision_naming_a_delivered_revision_passes() {
    let mut check = check();
    check.delivered(Delivered::Note(&note(REVISION)));
    assert_eq!(check.sent(KEY, &delete(REVISION)), None);
}
#[test]
fn a_decision_naming_a_revision_never_delivered_is_a_fault() {
    let mut check = check();
    check.delivered(Delivered::Note(&note(REVISION)));
    assert_eq!(check.sent(KEY, &delete(WRONG)), Some(undelivered(WRONG)));
}
#[test]
fn a_revision_delivered_after_the_decision_is_still_a_fault() {
    let mut check = check();
    let fault = check.sent(KEY, &delete(REVISION));
    check.delivered(Delivered::Note(&note(REVISION)));
    assert_eq!(fault, Some(undelivered(REVISION)));
}
#[test]
fn revision_history_evicts_only_its_oldest_revision() {
    let mut check = check();
    for revision in [SPEC, PERSON, REVISION] {
        check.delivered(Delivered::Note(&note(revision)));
    }
    assert_eq!(check.sent(KEY, &delete(SPEC)), Some(undelivered(SPEC)));
    assert_eq!(check.sent(KEY, &delete(PERSON)), None);
    assert_eq!(check.sent(KEY, &delete(REVISION)), None);
}
#[test]
fn the_same_note_number_in_another_scope_is_not_delivered() {
    let mut check = check();
    let mut document = note(REVISION);
    document.scope = NoteScope::Deployment;
    check.delivered(Delivered::Note(&document));
    assert_eq!(check.sent(KEY, &delete(REVISION)), Some(undelivered(REVISION)));
}
fn unchanged() -> Box<Amendment> {
    Box::new(Amendment {
        llm: None,
        addressed: None,
        spec: None,
        wake: None,
        dependencies: None,
        authority: None,
        instructions: None,
        tracked: None,
    })
}
#[test]
fn an_amendment_names_its_spec_and_person_revisions_separately() {
    let change = Box::new(Amendment {
        addressed: Some(AddressChange { revision: PERSON, address: PersonAddress::Party(AUTHOR) }),
        ..*unchanged()
    });
    let ask = Ask::Amend { project: PROJECT, task: TASK, revision: Some(SPEC), change, reason: text(b"change") };
    assert_eq!(
        jig_client_test_kit::check::named(&ask),
        vec![(Object::Task(TASK), SPEC), (Object::PersonTask(TASK), PERSON)]
    );
}
#[test]
fn a_priority_only_amendment_names_no_revision() {
    let ask =
        Ask::Amend { project: PROJECT, task: TASK, revision: None, change: unchanged(), reason: text(b"priority") };
    assert_eq!(jig_client_test_kit::check::named(&ask), vec![]);
}
#[test]
fn a_compound_names_each_operations_own_revision() {
    let mut operations = List::with_capacity(2);
    operations
        .push(Operation::Release { project: PROJECT, task: TASK, revision: REVISION, again: None })
        .expect("capacity");
    operations
        .push(Operation::Amend {
            project: PROJECT,
            task: OTHER_TASK,
            revision: Some(SPEC),
            change: unchanged(),
            reason: text(b"change"),
        })
        .expect("capacity");
    assert_eq!(
        jig_client_test_kit::check::named(&Ask::Compound { project: PROJECT, operations }),
        vec![(Object::Escalation(TASK), REVISION), (Object::Task(OTHER_TASK), SPEC)]
    );
}
fn scene(container: Role, effect: &[u8], sibling: bool) -> (View, NodeId) {
    let limits = skein_ui::Limits {
        nodes: 12,
        depth: 5,
        text_per_node: 64,
        text_bytes: 256,
        drawing_bytes: 0,
        styles_per_node: 0,
        patches: 64,
        bindings: 0,
        slots: 0,
        markdown_depth: 1,
        scene_marks: 0,
        scene_bytes: 0,
    };
    let mut view = View::new(&limits);
    let mut b = view.builder();
    b.open(Role::Page);
    b.open(Role::Region);
    b.name(b"Other");
    if sibling {
        b.open(Role::Text);
        b.text(Run::Plain, b"2 live runs");
        b.close();
    }
    b.close();
    b.open(container);
    b.name(b"Confirm cancellation");
    if !effect.is_empty() {
        b.open(Role::Text);
        b.text(Run::Plain, effect);
        b.close();
    }
    b.open(Role::Button);
    b.name(b"Confirm");
    b.close();
    b.close();
    b.close();
    let mut patches = Queue::with_capacity(64);
    view.finish(b, 1, &mut patches);
    let node = view.tree().nodes().iter().find(|n| n.role == Role::Button).expect("built button").id;
    (view, node)
}
#[test]
fn a_cancel_acted_on_without_its_effect_shown_is_a_fault() {
    let (view, node) = scene(Role::Dialog, b"", false);
    assert_eq!(
        check().acted(KEY, ActKind::Cancel, view.tree(), node, &[b"2 live runs"]),
        Some(Fault::Unseen { key: KEY, act: ActKind::Cancel })
    );
}
#[test]
fn a_cancel_whose_effect_the_confirmation_shows_passes() {
    let (view, node) = scene(Role::Dialog, b"2 live runs", false);
    assert_eq!(check().acted(KEY, ActKind::Cancel, view.tree(), node, &[b"2 live runs"]), None);
}
#[test]
fn facts_in_an_unrelated_region_do_not_justify_a_decision() {
    let (view, node) = scene(Role::Dialog, b"", true);
    assert_eq!(
        check().acted(KEY, ActKind::Cancel, view.tree(), node, &[b"2 live runs"]),
        Some(Fault::Unseen { key: KEY, act: ActKind::Cancel })
    );
}
#[test]
fn an_absent_acted_node_cannot_pass_silently() {
    let (view, _) = scene(Role::Dialog, b"2 live runs", false);
    assert_eq!(
        check().acted(KEY, ActKind::Cancel, view.tree(), NodeId(0), &[b"2 live runs"]),
        Some(Fault::Unseen { key: KEY, act: ActKind::Cancel })
    );
}

#[test]
fn a_cancel_whose_effect_the_nearest_named_region_shows_passes() {
    let (view, node) = scene(Role::Region, b"2 live runs", false);
    assert_eq!(check().acted(KEY, ActKind::Cancel, view.tree(), node, &[b"2 live runs"]), None);
}
#[test]
fn a_different_count_cannot_supply_a_decisions_fact() {
    let (view, node) = scene(Role::Dialog, b"12 live runs", false);
    assert_eq!(
        check().acted(KEY, ActKind::Cancel, view.tree(), node, &[b"2 live runs"]),
        Some(Fault::Unseen { key: KEY, act: ActKind::Cancel })
    );
}
#[test]
fn bounded_object_history_evicts_the_oldest_object() {
    let mut check = Check::new(&Limits { objects: 1, revisions: 2 });
    check.delivered(Delivered::Note(&note(REVISION)));
    let mut other = note(REVISION);
    other.number = OTHER_TASK;
    check.delivered(Delivered::Note(&other));
    assert_eq!(check.sent(KEY, &delete(REVISION)), Some(undelivered(REVISION)));
}
