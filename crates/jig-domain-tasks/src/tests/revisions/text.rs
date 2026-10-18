use super::{Harness, amendment, spec, task, to};
use crate::{Amendment, Event, Parameter, Party, Problem, Refusal, Request, Spec};

const TASK: u64 = 1;
const REQUESTER: u64 = 9;
const MESSAGE: u64 = 1;
const PARAMETER: u32 = 1;
const NON_TEXT: [u8; 32] = [0xff; 32];

fn parameter_spec() -> Spec {
    let mut spec = spec(b"task");
    spec.parameters = Box::new([Parameter::Bytes { name: PARAMETER, value: Box::new(NON_TEXT) }]);
    spec
}

fn refuse_batch(spec: Spec) {
    let mut test = Harness::new();
    let member = crate::New { spec, ..task(TASK, &[]) };
    let out = test.send(Event::Make { reply_to: to(), creator: Party::Person(REQUESTER), batch: Box::new([member]) });
    assert_eq!(out, vec![Request::Refused { reply_to: to(), problem: Problem::new(Some(TASK), Refusal::Spec) }]);
}

fn refuse_amendment(spec: Spec) {
    let mut test = Harness::new();
    test.make(vec![task(TASK, &[])]);
    let out = test.send(Event::Amend {
        funder: None,
        reply_to: to(),
        by: Party::Person(REQUESTER),
        task: TASK,
        message: MESSAGE,
        stop_run: false,
        amendment: Amendment { spec: Some(spec), ..amendment() },
    });
    assert_eq!(out, vec![Request::Refused { reply_to: to(), problem: Problem::new(Some(TASK), Refusal::Spec) }]);
}

#[test]
fn a_batch_refuses_a_parameter_that_is_not_text() {
    refuse_batch(parameter_spec());
}

#[test]
fn a_batch_refuses_words_that_are_not_text() {
    refuse_batch(spec(&NON_TEXT));
}

#[test]
fn an_amendment_refuses_a_parameter_that_is_not_text() {
    refuse_amendment(parameter_spec());
}

#[test]
fn an_amendment_refuses_words_that_are_not_text() {
    refuse_amendment(spec(&NON_TEXT));
}
