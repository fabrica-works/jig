use super::*;

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
