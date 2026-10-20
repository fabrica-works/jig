//! A task's LLM choice and `me`'s models by their configured names (contract.md, 3.2 and 5.1).
use super::fixtures;
use crate::{Component, Unusable, translate};
use alloc::boxed::Box;
use jig_client_protocol_wire::v10 as w;
use jig_domain_tasks as t;
use skein_lib::{Queue, Reader};

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
