//! A task's LLM choice and `me`'s models by their configured names (contract.md, 3.2 and 5.1).
use super::fixtures;
use crate::{Component, Unusable, translate};
use alloc::boxed::Box;
use jig_client_protocol_wire::v10 as w;
use jig_domain_people as p;
use jig_domain_tasks as t;
use skein_lib::{Queue, Reader, Writer};

// MODEL_DESCRIPTION_BYTES (configuration.md, 6); each é takes two bytes.
const DESCRIPTION_BYTES: usize = 512;
const DESCRIPTION_CHARACTERS: usize = DESCRIPTION_BYTES / 2;
// Repeated name and long effort, three invalid descriptions, and five zero price fields.
const INVALID_MODEL_CASES: u32 = 2 + 3 + 5;

fn choice() -> t::LlmChoice {
    t::LlmChoice {
        account: None,
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
    frame.projects.get_mut(0).unwrap().models = jig_domain_authority::Models::Named(0b10);
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
    assert_eq!(
        careful.prices().clone().into_parts(),
        w::MePricesParts { input: 3, cached: 1, cache_write: Some(5), output: 15, unit: 1_000_000 }
    );
    assert_eq!(careful.description().as_deref(), Some(b"For careful work".as_slice()));
    let quick = all.models().get(1).unwrap();
    assert_eq!((quick.name(), quick.provider(), quick.efforts().len()), (b"quick".as_slice(), &None, 0));
    assert_eq!(
        quick.prices().clone().into_parts(),
        w::MePricesParts { input: 1, cached: 1, cache_write: None, output: 2, unit: 1000 }
    );
    assert_eq!(quick.description(), &None);
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
    let mut cases = skein_lib::List::with_capacity(INVALID_MODEL_CASES);
    cases.push(("repeated name", twice)).unwrap();
    cases.push(("long effort", long)).unwrap();
    for (name, description) in [
        ("long description", Box::from([b'd'; DESCRIPTION_BYTES + 1].as_slice())),
        ("empty description", Box::from(b"".as_slice())),
        ("invalid description text", Box::from([0xff].as_slice())),
    ] {
        let mut model = fixtures::models().first().unwrap().clone();
        model.description = Some(description);
        cases.push((name, settings_with(model))).unwrap();
    }
    let prices = fixtures::models().first().unwrap().prices;
    for (name, prices) in [
        ("zero input price", crate::Prices { input: 0, ..prices }),
        ("zero cached price", crate::Prices { cached: 0, ..prices }),
        ("zero cache-write price", crate::Prices { cache_write: Some(0), ..prices }),
        ("zero output price", crate::Prices { output: 0, ..prices }),
        ("zero price unit", crate::Prices { unit: 0, ..prices }),
    ] {
        let mut model = fixtures::models().first().unwrap().clone();
        model.prices = prices;
        cases.push((name, settings_with(model))).unwrap();
    }
    for (name, settings) in cases.into_boxed() {
        let mut out = Queue::with_capacity(8);
        assert_eq!(Component::new(settings, &env, &mut out).err(), Some(Unusable::Limits), "{name}");
    }
    let mut smaller = env;
    smaller.limits.wire.me_model_description_some = 3;
    let mut out = Queue::with_capacity(8);
    assert_eq!(Component::new(fixtures::settings(), &smaller, &mut out).err(), Some(Unusable::Limits));
}

fn settings_with(model: crate::Model) -> crate::Settings {
    crate::Settings { models: Box::new([model]), ..fixtures::settings() }
}

#[test]
fn configured_prices_and_bounded_descriptions_can_start_the_component() {
    let env = fixtures::env();
    let full = "é".repeat(DESCRIPTION_CHARACTERS).into_bytes().into_boxed_slice();
    for description in [None, Some(full)] {
        for cache_write in [None, Some(1), Some(u64::MAX)] {
            let mut model = fixtures::models().first().unwrap().clone();
            model.description = description.clone();
            model.prices = crate::Prices { input: u64::MAX, cached: 1, cache_write, output: u64::MAX, unit: u32::MAX };
            let mut out = Queue::with_capacity(8);
            assert_eq!(Component::new(settings_with(model), &env, &mut out).err(), None);
        }
    }
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
        account: None,
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

fn selector_ask(names: &[&[u8]]) -> w::Ask {
    let limits = w::CEILINGS;
    let (w::Ask::ChangePolicy(ask), _) = fixtures::authority_ask() else { panic!("role edit") };
    let mut ask = ask.into_parts();
    let w::PolicyEdit::Role(role) = ask.edit else { panic!("role") };
    let mut role = role.into_parts();
    let mut value = role.value.into_parts();
    let mut authority = value.authority.into_parts();
    let mut rows = skein_lib::List::with_capacity(u32::try_from(names.len()).unwrap());
    for name in names {
        rows.push(Box::from(*name)).unwrap();
    }
    authority.models =
        Some(w::ModelSet::Names(w::ModelSetNames::new(&limits, w::ModelSetNamesParts { names: rows }).unwrap()));
    value.authority = w::OwnerAuthority::new(&limits, authority).unwrap();
    role.value = w::PolicyEditRole::new(&limits, value).unwrap();
    ask.edit = w::PolicyEdit::Role(w::PolicyEditRoleValue::new(&limits, role).unwrap());
    w::Ask::ChangePolicy(w::ChangePolicy::new(&limits, ask).unwrap())
}
#[test]
fn original_selector_names_are_canonical_and_unknowns_reach_retained_key_lookup() {
    let models = fixtures::models();
    let a = translate::asks::named(selector_ask(&[b"quick", b"careful"]), &models, &w::CEILINGS).unwrap();
    let b = translate::asks::named(selector_ask(&[b"careful", b"quick"]), &models, &w::CEILINGS).unwrap();
    assert_eq!(a.ask, b.ask);
    assert_eq!(a.models, b.models);
    let unknown = translate::asks::named(selector_ask(&[b"removed"]), &models, &w::CEILINGS).unwrap();
    assert_eq!(unknown.models.as_ref(), &[Box::<[u8]>::from(b"removed".as_slice())]);
    for names in [&[b"quick".as_slice(), b"quick".as_slice()][..], &[b"".as_slice()][..]] {
        translate::asks::named(selector_ask(names), &models, &w::CEILINGS).expect_err("invalid selector");
    }
}

#[test]
fn an_explicit_account_reaches_the_people_child() {
    let l = fixtures::wire();
    let mut choice = fixtures::careful_and_quick(&l).into_parts();
    choice.account = Some(42);
    let ask = w::Ask::Amend(amend(w::LlmChoice::new(&l, choice).unwrap()));
    let Ok(p::Ask::Amend { amendment, .. }) = translate::asks::ask(ask, &fixtures::models(), &l) else {
        panic!("amendment")
    };
    assert_eq!(amendment.llm.unwrap().account, Some(42));
}

#[test]
fn task_documents_keep_the_claimed_account_separate_from_the_preference() {
    let l = fixtures::wire();
    let mut task = fixtures::task_with_hold();
    let mut preferred = choice();
    preferred.account = Some(42);
    task.llm = Some(preferred);
    task.kept_account = Some(41);
    task.run_wait = Some(jig_domain_core::documents::TaskRunWait::Account {
        account: Some(41),
        source: Some(91),
        why: jig_domain_core::documents::TaskWaitWhy::Source(jig_domain_core::documents::TaskSourceState::Limited {
            until: None,
        }),
    });
    let record = translate::task::document(&task, &fixtures::models(), &l).unwrap();
    assert_eq!(*record.llm().as_ref().unwrap().account(), Some(42));
    assert_eq!(*record.kept_account(), Some(41));
    let Some(w::TaskRunWait::Account(wait)) = record.run_wait() else { panic!("account wait") };
    assert_eq!((*wait.account(), *wait.source()), (Some(41), Some(91)));
    let w::TaskWaitWhy::Source(reason) = wait.why() else { panic!("source wait") };
    let w::TaskSourceState::Limited(limited) = reason.state() else { panic!("limited") };
    assert_eq!(*limited.until(), None);
}

#[test]
fn an_account_refusal_preserves_its_number() {
    let l = fixtures::wire();
    for why in [p::AccountWhy::Unknown, p::AccountWhy::Provider, p::AccountWhy::Uses, p::AccountWhy::Limited] {
        let reply = p::Reply::Outcome(p::Outcome::Refused(p::Refusal::Account { account: 42, why }));
        let w::Final::Refused(value) = fixtures::translated(&reply, &l) else { panic!("refusal") };
        let w::Refusal::Account(account) = value.refusal() else { panic!("typed account refusal") };
        assert_eq!(account.account(), 42);
    }
}
