//! A task's LLM choice and `me`'s models by their configured names (contract.md, 3.2 and 5.1).
use super::fixtures;
use crate::{Component, Unusable, translate};
use alloc::boxed::Box;
use jig_client_protocol_wire::v10 as w;
use jig_domain_people as p;
use jig_domain_tasks as t;
use skein_lib::{Queue, Reader, Writer};

fn choice() -> t::LlmChoice {
    t::LlmChoice {
        main: t::ModelEffort { model: 0, effort: Some(1) },
        subagents: Box::new([t::ModelEffort { model: 1, effort: None }]),
    }
}
/// The choice as `choice()` names it: `careful` at `high`, with `quick` for sub-agents.
fn assert_named(choice: &w::LlmChoice) {
    assert_eq!(choice.main().model(), b"careful");
    assert_eq!(choice.main().effort().as_deref(), Some(b"high".as_slice()));
    assert_eq!(choice.subagents().len(), 1);
    let subagent = choice.subagents().get(0).unwrap();
    assert_eq!((subagent.model(), subagent.effort()), (b"quick".as_slice(), &None));
}
fn project(record: &w::Me, at: usize) -> w::MeProject {
    let mut reader = Reader::new(record.projects());
    for _ in 0..at {
        skein_codec::piece::split(&mut reader).unwrap().unwrap();
    }
    let bytes = skein_codec::piece::split(&mut reader).unwrap().unwrap();
    w::MeProject::decode(&w::CEILINGS, &mut Reader::new(&bytes)).unwrap()
}

#[test]
fn me_lists_only_the_models_the_party_s_authority_allows_by_name() {
    let mut frame = fixtures::frame_two_projects();
    frame.projects.get_mut(0).unwrap().models = jig_domain_authority::Models(0b10);
    frame.projects.get_mut(1).unwrap().models = jig_domain_authority::Models::ALL;
    let record = translate::owner::frame(&frame, &fixtures::models(), &w::CEILINGS).unwrap();
    let some = project(&record, 0);
    assert_eq!(some.models().len(), 1);
    assert_eq!(some.models().get(0).unwrap().name(), b"quick");
    let all = project(&record, 1);
    assert_eq!(all.models().len(), 2);
    let careful = all.models().get(0).unwrap();
    assert_eq!(careful.name(), b"careful");
    assert_eq!((careful.provider(), careful.window(), careful.output()), (&Some(3), 200_000, 32_000));
    assert_eq!(careful.efforts().len(), 2);
    assert_eq!(careful.efforts().get(0).unwrap().as_ref(), b"low");
    assert_eq!(careful.efforts().get(1).unwrap().as_ref(), b"high");
    let quick = all.models().get(1).unwrap();
    assert_eq!((quick.name(), quick.provider(), quick.efforts().len()), (b"quick".as_slice(), &None, 0));
}

#[test]
fn a_template_s_and_a_task_s_choice_are_named_by_configuration() {
    let models = fixtures::models();
    let mut frame = fixtures::frame_two_projects();
    let mut template = fixtures::template();
    template.llm = Some(choice());
    frame.projects.get_mut(0).unwrap().templates = fixtures::list([template]);
    let record = translate::owner::frame(&frame, &models, &w::CEILINGS).unwrap();
    let first = project(&record, 0);
    let bytes = skein_codec::piece::split(&mut Reader::new(first.templates())).unwrap().unwrap();
    let template = w::Template::decode(&w::CEILINGS, &mut Reader::new(&bytes)).unwrap();
    assert_named(template.llm().as_ref().unwrap());

    let mut task = fixtures::task_with_hold();
    task.llm = Some(choice());
    let record = translate::task::document(&task, &models, &w::CEILINGS).unwrap();
    assert_named(record.llm().as_ref().unwrap());
    task.llm = None;
    assert_eq!(translate::task::document(&task, &models, &w::CEILINGS).unwrap().llm(), &None);
}

#[test]
fn configured_models_past_the_wire_or_named_twice_cannot_start_the_component() {
    let env = fixtures::env();
    let quick = fixtures::models().get(1).unwrap().clone();
    let mut twice = fixtures::settings();
    twice.models = Box::new([quick.clone(), quick]);
    let mut long = fixtures::settings();
    let mut careful = fixtures::models().first().unwrap().clone();
    careful.efforts = Box::new([Box::from([b'e'; 33].as_slice())]);
    long.models = Box::new([careful]);
    for settings in [twice, long] {
        let mut out = Queue::with_capacity(8);
        assert_eq!(Component::new(settings, &env, &mut out).err(), Some(Unusable::Limits));
    }
    let mut out = Queue::with_capacity(8);
    assert_eq!(Component::new(fixtures::settings(), &env, &mut out).err(), None);
}

/// The create fixture's first task, choosing `choice`.
fn create(choice: w::LlmChoice) -> w::Ask {
    let limits = fixtures::wire();
    let (w::Ask::Create(create), _) = fixtures::create_ask() else { panic!("create fixture") };
    let bytes = skein_codec::piece::split(&mut Reader::new(create.tasks())).unwrap().unwrap();
    let mut parts = w::NewTask::decode(&limits, &mut Reader::new(&bytes)).unwrap().into_parts();
    parts.llm = Some(choice);
    let task = w::NewTask::new(&limits, parts).unwrap();
    let mut out = Writer::new(usize::try_from(task.measure()).unwrap());
    task.encode(&mut out).unwrap();
    let tasks = fixtures::pieces(&[out.finish()]);
    w::Ask::Create(w::Create::new(&limits, w::CreateParts { project: fixtures::PROJECT, tasks }).unwrap())
}
/// An amendment changing only the task's choice.
fn amend(choice: w::LlmChoice) -> Box<w::Amend> {
    let limits = fixtures::wire();
    let mut change = fixtures::empty_amendment(None).into_parts();
    change.llm = Some(choice);
    let parts = w::AmendParts {
        project: fixtures::PROJECT,
        task: 8,
        revision: None,
        change: w::Amendment::new(&limits, change).unwrap(),
        reason: Box::new([]),
    };
    Box::new(w::Amend::new(&limits, parts).unwrap())
}
/// The place and refusal the engine answers an ask with before the core sees it.
fn refused(ask: &w::Ask) -> (Option<u32>, w::Refusal) {
    let limits = fixtures::wire();
    let answer = translate::llm::refused(ask, &fixtures::models(), &limits).unwrap().unwrap();
    let w::Final::Refused(value) = fixtures::final_record(answer, &limits) else { panic!("refusal") };
    (*value.at(), value.refusal().clone())
}
fn named(model: &[u8], why: w::LlmWhy) -> w::Refusal {
    w::Refusal::Llm(w::RefusalLlm::new(&fixtures::wire(), w::RefusalLlmParts { model: model.into(), why }).unwrap())
}

#[test]
fn a_choice_naming_a_model_or_an_effort_configuration_lacks_is_refused_by_its_name() {
    let l = fixtures::wire();
    let models = fixtures::models();
    let unknown = create(fixtures::llm_choice(&l, b"careless", None, b"quick"));
    assert_eq!(refused(&unknown), (None, named(b"careless", w::LlmWhy::Unconfigured)));
    let effort = w::Ask::Amend(amend(fixtures::llm_choice(&l, b"careful", Some(b"medium"), b"quick")));
    assert_eq!(refused(&effort), (None, named(b"careful", w::LlmWhy::Effort)));
    let message = w::MessageParts { project: fixtures::PROJECT, task: 8, words: b"words".as_slice().into() };
    let operations = [
        w::Operation::Message(w::Message::new(&l, message).unwrap()),
        w::Operation::Amend(amend(fixtures::llm_choice(&l, b"careful", Some(b"high"), b"hasty"))),
    ];
    let compound = fixtures::compound(fixtures::PROJECT, &operations);
    assert_eq!(refused(&compound), (Some(1), named(b"hasty", w::LlmWhy::Unconfigured)));
    let configured = create(fixtures::careful_and_quick(&l));
    assert!(translate::llm::refused(&configured, &models, &l).unwrap().is_none());
}

#[test]
fn an_amended_choice_reaches_the_people_child_in_configuration_s_order() {
    let l = fixtures::wire();
    let ask = w::Ask::Amend(amend(fixtures::careful_and_quick(&l)));
    let Ok(p::Ask::Amend { amendment, .. }) = translate::asks::ask(ask, &fixtures::models(), &l) else {
        panic!("amendment")
    };
    let expected = p::LlmChoice {
        main: p::ModelEffort { model: 0, effort: Some(1) },
        subagents: Box::new([p::ModelEffort { model: 1, effort: None }]),
    };
    assert_eq!(amendment.llm, Some(expected));
}

#[test]
fn a_model_the_core_refuses_is_named_by_configuration() {
    let l = fixtures::wire();
    let reply = p::Reply::Outcome(p::Outcome::Refused(p::Refusal::Llm { model: 1, why: p::LlmWhy::Charter }));
    let w::Final::Refused(value) = fixtures::translated(&reply, &l) else { panic!("refusal") };
    assert_eq!(value.refusal(), &named(b"quick", w::LlmWhy::Charter));
    let past = p::Reply::Outcome(p::Outcome::Refused(p::Refusal::Llm { model: 2, why: p::LlmWhy::Authority }));
    assert_eq!(
        translate::answers::answer(&past, &fixtures::models(), &l),
        Err(w::Problem { path: w::Path::RefusalLlmModel, reason: skein_codec::Reason::Bound })
    );
}
