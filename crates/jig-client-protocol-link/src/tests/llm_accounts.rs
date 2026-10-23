use super::*;

// MODEL_DESCRIPTION_BYTES (configuration.md, 6).
const DESCRIPTION_BYTES: usize = 512;

fn me_with_model(prices: w::MePricesParts, description: Option<Box<[u8]>>) -> w::Me {
    let limits = fixtures::limits();
    let model = w::MeModel::new(
        &limits.wire,
        w::MeModelParts {
            name: b"chosen".as_slice().into(),
            provider: Some(3),
            window: 8192,
            output: 1024,
            efforts: skein_lib::List::with_capacity(0),
            prices: w::MePrices::new(&limits.wire, prices).unwrap(),
            description,
        },
    )
    .unwrap();
    let project = w::MeProject::new(
        &limits.wire,
        w::MeProjectParts {
            project: 3,
            name: w::DocumentText::new(&limits.wire, w::DocumentTextParts { bytes: Box::from(*b"Tally"), cut: false })
                .unwrap(),
            role: w::HeldRole::Owner,
            pool: None,
            templates: Box::default(),
            creating: None,
            models: fixtures::list([model]),
        },
    )
    .unwrap();
    let mut encoded = skein_lib::Writer::new(usize::try_from(project.measure()).unwrap());
    project.encode(&mut encoded).unwrap();
    let encoded = encoded.finish();
    let mut pieces = skein_lib::Writer::new(encoded.len().checked_add(4).unwrap());
    skein_codec::piece::write(&mut pieces, &encoded).unwrap();
    let mut me = fixtures::frame_record().into_parts();
    me.projects = pieces.finish();
    w::Me::new(&limits.wire, me).unwrap()
}

#[test]
fn models_decode_configured_prices_and_optional_descriptions() {
    let limits = fixtures::limits();
    let full: Box<[u8]> = Box::from([b'd'; DESCRIPTION_BYTES].as_slice());
    let described = c::Text::new(full.clone(), limits.wire.me_model_description_some).unwrap();
    for (description, expected) in [(None, None), (Some(full), Some(described))] {
        for cache_write in [None, Some(5)] {
            let me = me_with_model(
                w::MePricesParts { input: 2, cached: 1, cache_write, output: 7, unit: 1000 },
                description.clone(),
            );
            let me = translate::documents::frame(me, &limits).unwrap();
            let model = me.projects.get(0).unwrap().models.get(0).unwrap();
            assert_eq!(model.prices, c::ModelPrices { input: 2, cached: 1, cache_write, output: 7, unit: 1000 });
            assert_eq!(model.description, expected);
        }
    }
}

#[test]
fn an_empty_model_description_is_malformed() {
    let limits = fixtures::limits();
    let me = me_with_model(
        w::MePricesParts { input: 2, cached: 1, cache_write: None, output: 7, unit: 1000 },
        Some(Box::default()),
    );
    assert_eq!(translate::documents::frame(me, &limits), Err(Undecodable::Malformed));
}

#[test]
fn zero_model_prices_or_units_are_malformed() {
    let limits = fixtures::limits();
    for (input, cached, cache_write, output, unit) in [
        (0, 1, None, 7, 1000),
        (2, 0, None, 7, 1000),
        (2, 1, Some(0), 7, 1000),
        (2, 1, None, 0, 1000),
        (2, 1, None, 7, 0),
    ] {
        let me = me_with_model(w::MePricesParts { input, cached, cache_write, output, unit }, None);
        assert_eq!(translate::documents::frame(me, &limits), Err(Undecodable::Malformed));
    }
}

#[test]
fn an_explicit_account_preference_survives_link_storage() {
    let limits = fixtures::limits();
    let choice = c::LlmChoice {
        account: Some(42),
        main: c::ModelEffort { model: c::Text::new(b"chosen".as_slice().into(), 32).unwrap(), effort: None },
        subagents: skein_lib::List::with_capacity(0),
    };
    let mut ask = fixtures::create_ask();
    let c::Ask::Create { tasks, .. } = &mut ask else { panic!("create") };
    tasks.get_mut(0).unwrap().llm = Some(choice);
    let wire = translate::asks::ask(&ask, &limits.wire).unwrap();
    assert_eq!(translate::store::restore_ask(wire, &limits).unwrap(), ask);
}

#[test]
fn an_account_refusal_keeps_its_reason_in_the_link() {
    let limits = fixtures::limits();
    for (wire, client) in [
        (w::AccountWhy::Unknown, c::AccountWhy::Unknown),
        (w::AccountWhy::Provider, c::AccountWhy::Provider),
        (w::AccountWhy::Uses, c::AccountWhy::Uses),
        (w::AccountWhy::Limited, c::AccountWhy::Limited),
    ] {
        let wire = w::Refusal::Account(
            w::RefusalAccount::new(&limits.wire, w::RefusalAccountParts { account: 42, why: wire }).unwrap(),
        );
        assert_eq!(
            translate::answers::refusal(&wire, &limits).unwrap(),
            c::Refusal::Account { account: 42, why: client }
        );
    }
}

#[test]
fn an_unknown_limit_reset_stays_unknown_in_the_task_wait() {
    let limits = fixtures::limits();
    let state = w::TaskSourceState::Limited(
        w::TaskSourceStateUntil::new(&limits.wire, w::TaskSourceStateUntilParts { until: None }).unwrap(),
    );
    let why = w::TaskWaitWhy::Source(w::TaskWaitSource::new(&limits.wire, w::TaskWaitSourceParts { state }).unwrap());
    let wire = w::TaskRunWait::Account(
        w::TaskWaitAccount::new(&limits.wire, w::TaskWaitAccountParts { account: Some(42), source: Some(91), why })
            .unwrap(),
    );
    let mut task = fixtures::task_record().into_parts();
    task.run_wait = Some(wire);
    let wire = w::Task::new(&limits.wire, task).unwrap();
    assert_eq!(
        translate::documents::task_fields(&wire, &limits).unwrap().run_wait,
        Some(c::TaskRunWait::Account {
            account: Some(42),
            source: Some(91),
            why: c::TaskWaitWhy::Source(c::TaskSourceState::Limited { until: None }),
        })
    );
}
