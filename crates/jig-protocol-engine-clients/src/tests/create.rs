use super::fixtures as f;
use crate::translate::asks;
use jig_client_protocol_wire::v10 as w;
use jig_domain_people as p;
#[test]
fn a_create_reaches_the_people_child_with_each_task_s_fields() {
    let (wire, expected) = f::create_ask();
    assert_eq!(asks::ask(wire, &f::wire()), Ok(expected));
}
#[test]
fn an_as_allowed_answer_names_what_was_given() {
    let l = f::wire();
    let w::Final::Done(value) = f::translated(
        &p::Reply::Outcome(p::Outcome::Created {
            first: f::FIRST_TASK,
            last: f::LAST_TASK,
            given: Some(p::Given { task: f::LAST_TASK, amount: 42 }),
        }),
        &l,
    ) else {
        panic!("done")
    };
    assert_eq!(value.outcome(), &f::created_outcome());
}
#[test]
fn a_create_refusal_keeps_its_member_and_exact_category() {
    let l = f::wire();
    for (name, why, expected) in f::create_refusals() {
        let w::Final::Refused(value) = f::translated(&p::Reply::Refused(why), &l) else { panic!("{name}: refusal") };
        assert_eq!((value.refusal(), *value.at()), (&expected, None), "{name}");
    }
}
