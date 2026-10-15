//! A card and dialog share facts, while each draft has one platform input.
use jig_client_world::{
    Until,
    stories::{
        self,
        person_confirmation::{self, Form},
    },
};
use jig_fake_person::{Find, Step};
use jig_view::ui::Role;
use skein_lib::Duration;

fn answer(form: Form, seed: u64) {
    let mut world = person_confirmation::world(seed, form);
    let client = world.client(7, stories::inbox());
    let within = Duration::from_secs(1);
    world.person(
        client,
        vec![
            Step::See { find: Find::text(b"first whole report"), within },
            Step::See { find: Find::text(b"second whole report"), within },
            Step::Press { find: Find::exactly_named(Role::Button, b"Answer") },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    assert!(world.tree(client).nodes().iter().any(|node| node.role == Role::Dialog));
    let field = if form.choice().is_some() { b"Your reason".as_slice() } else { b"Your answer".as_slice() };
    let before = world
        .tree(client)
        .nodes()
        .iter()
        .find(|node| node.role == Role::TextArea && node.name.as_deref() == Some(field))
        .expect("the confirmation field is visible");
    let before_id = before.id;
    let before_written = before.value.as_ref().expect("input value").written;
    let mut steps = Vec::new();
    if let Some(choice) = form.choice() {
        steps.push(Step::Press { find: Find::exactly_named(Role::Choice, choice) });
    }
    let input = if form.choice().is_some() { b"second report".as_slice() } else { b"my complete answer".as_slice() };
    steps.push(Step::Type { find: Find::exactly_named(Role::TextArea, field), words: input.into() });
    world.person(client, steps);
    // Each callback updates only the targeted platform input, applies patches,
    // and independently compares the complete projection with the retained tree.
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    let fields: Vec<_> = world
        .tree(client)
        .nodes()
        .iter()
        .filter(|node| node.role == Role::TextArea && node.name.as_deref() == Some(field))
        .collect();
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].id, before_id);
    let value = fields[0].value.as_ref().expect("the typed value remains visible");
    assert_eq!(value.text.as_ref(), input);
    assert_eq!(value.written, before_written, "local typing is not a domain-written replacement");
    world.person(client, vec![Step::Press { find: Find::exactly_named(Role::Button, b"Confirm") }]);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    let asks: Vec<_> =
        world.front().engine.commits().iter().filter_map(|commit| commit.ask.as_ref()).cloned().collect();
    assert_eq!(asks, [form.ask()]);
}

#[test]
fn a_native_choice_card_and_confirmation_keep_one_platform_reason_input() {
    answer(Form::OwnerChoice, 9311);
}
#[test]
fn a_choice_card_and_confirmation_keep_one_platform_reason_input() {
    answer(Form::Choice, 9312);
}
#[test]
fn a_native_report_card_and_confirmation_keep_one_platform_answer_input() {
    answer(Form::OwnerReport, 9313);
}
#[test]
fn a_words_card_and_confirmation_keep_one_platform_answer_input() {
    answer(Form::Words, 9314);
}
