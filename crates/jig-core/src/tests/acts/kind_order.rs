//! Actual offers and admission retain and use the configured literal grant order.
use super::*;

fn pair(connector: u16, kind: u16, implies: u16) -> authority::Implication {
    authority::Implication { connector, kind, implies }
}
fn grant(kind: u16, last: authority::Last) -> authority::Grant {
    authority::Grant {
        connector: 1,
        kind,
        pattern: authority::Pattern { segments: Box::new([b"repo".as_slice().into()]), last },
    }
}
fn configured(pairs: Box<[authority::Implication]>, rows: u32) -> (Core, Limits) {
    let mut limits = limits();
    limits.documents.lines = rows;
    limits.authority.implications = u32::try_from(pairs.len()).unwrap();
    let mut cfg = config(41);
    let mut rules = cfg.authority.rules().clone();
    let mut value = cfg.authority.policy(1).unwrap().clone();
    rules.implies = authority::Implies::new(pairs, limits.authority.implications).unwrap();
    rules.ceiling.grants = Box::new([grant(2, authority::Last::Open(Box::new([])))]);
    value.ceiling.grants = rules.ceiling.grants.clone();
    value.roles[0].authority.grants = Box::new([grant(2, authority::Last::Open(b"a".as_slice().into()))]);
    value.roles[0].period_spend = 1000;
    cfg.authority = authority::Domain::new(rules, limits.authority).unwrap();
    let mut out = Queue::with_capacity(authority::POLICY_MAX_OUT);
    authority::step(&mut cfg.authority, authority::Event::Policy { project: 1, policy: value }, &mut out);
    assert_eq!(out.pop(), Some(authority::PolicyFact::Added { project: 1 }));
    cfg.settings.chat_authority.grants = Box::new([grant(1, authority::Last::Exact(b"alpha".as_slice().into()))]);
    let (prepared, _) = core();
    let mut core = Core::new(cfg, &limits);
    core.people = prepared.people;
    make_task(&mut core, &limits, 1);
    send_task(
        &mut core,
        &limits,
        tasks::Event::CarvePool {
            reply_to: ReplyTo::new(Token::new(2)),
            project: 1,
            person: 1,
            period: 1,
            budget: 600,
        },
    );
    (core, limits)
}
fn change(core: &Core, spend: u64) -> tasks::Amendment {
    let mut after = core.tasks.task(1).unwrap().authority.clone();
    after.budget.spend = spend;
    tasks::Amendment {
        addressed: None,
        instructions: None,
        spec: None,
        wake: None,
        dependencies: None,
        authority: Some(after),
        release: None,
        reason: Box::new([]),
    }
}

#[test]
fn amendment_offers_keep_every_configured_pair_in_order_with_duplicates_and_fullwidth_values() {
    let pairs = Box::new([
        pair(u16::MAX, u16::MAX, u16::MAX - 1),
        pair(1, 2, 1),
        pair(u16::MAX, u16::MAX, u16::MAX - 1),
        pair(2, 2, 1),
    ]);
    let (core, limits) = configured(pairs.clone(), 4);
    let Admission::Bounded { within: Bounds::Amend { implications, .. }, .. } =
        offers::admit(&core, &limits, 1, Act::Amend, &Subject::Task { task: 1 })
    else {
        panic!("whole bounded amendment")
    };
    assert_eq!(implications.as_slice(), pairs.as_ref());
    assert_eq!(implications.capacity(), 4);
    assert_eq!(core.authority.rules().implies.entries(), pairs.as_ref());
    let Document::Task(document) = make(&core, &limits, 1, &Subject::Task { task: 1 }, &window()).unwrap() else {
        panic!("task document")
    };
    let offer = document.acts.iter().find(|offer| offer.act == Act::Amend).unwrap();
    let Admission::Bounded { within: Bounds::Amend { implications: received, .. }, .. } = &offer.admission else {
        panic!("whole received relation")
    };
    assert_eq!(received, &implications);
}

#[test]
fn budget_only_amendments_preserve_narrower_implied_grants_and_follow_direct_hard_and_recipient_facts() {
    let (mut core, limits) = configured(Box::new([pair(1, 2, 1)]), 4);
    let before = core.tasks.task(1).unwrap();
    let revisions = before.revisions;
    let draft = change(&core, 150);
    assert_eq!(
        core.amend_admit(1, Some(people::Role::Owner), 1, 1, &draft, limits.tasks.depth, &limits.tasks),
        Ok((false, false))
    );
    let mut mixed = change(&core, 550);
    mixed.authority.as_mut().unwrap().grants[0].pattern.last = tasks::Last::Exact(b"zeta".as_slice().into());
    assert_eq!(
        core.amend_admit(1, Some(people::Role::Owner), 1, 1, &mixed, limits.tasks.depth, &limits.tasks),
        Ok((true, true))
    );
    let Admission::Bounded { beyond: Beyond::Proposed { to }, .. } =
        offers::admit(&core, &limits, 1, Act::Amend, &Subject::Task { task: 1 })
    else {
        panic!("proposal facts")
    };
    assert_eq!(to, policy_holder(&[0]));
    policy(&mut core, false, 1000);
    let Admission::Bounded { beyond: Beyond::Refused(why), .. } =
        offers::admit(&core, &limits, 1, Act::Amend, &Subject::Task { task: 1 })
    else {
        panic!("recipient absence")
    };
    assert_eq!(
        core.amend_admit(1, Some(people::Role::Owner), 1, 1, &mixed, limits.tasks.depth, &limits.tasks),
        Err(why)
    );
    mixed.authority = change(&core, 90).authority;
    assert_eq!(
        core.amend_admit(1, Some(people::Role::Owner), 1, 1, &mixed, limits.tasks.depth, &limits.tasks),
        Ok((true, false))
    );
    assert_eq!(core.tasks.task(1).unwrap().revisions, revisions, "pure admission moves no named revision");
}

#[test]
fn an_amendment_relation_at_its_receiving_boundary_fits_and_one_under_refuses_the_whole_document() {
    let pairs = Box::new([pair(1, 2, 1), pair(2, 2, 1), pair(u16::MAX, 2, 1), pair(1, 2, 1)]);
    let (core, mut limits) = configured(pairs, 4);
    assert!(make(&core, &limits, 1, &Subject::Task { task: 1 }, &window()).is_ok());
    limits.documents.lines = 3;
    assert_eq!(
        offers::admit(&core, &limits, 1, Act::Amend, &Subject::Task { task: 1 }),
        Admission::Refused(people::Refusal::Limit { which: people::LimitKind::Authority })
    );
    assert_eq!(make(&core, &limits, 1, &Subject::Task { task: 1 }, &window()), Err(Missing::TooLarge));
}

#[test]
fn an_empty_amendment_relation_is_not_expanded_into_equality_rows() {
    let (mut core, limits) = core();
    make_task(&mut core, &limits, 1);
    let Admission::Bounded { within: Bounds::Amend { implications, .. }, .. } =
        offers::admit(&core, &limits, 1, Act::Amend, &Subject::Task { task: 1 })
    else {
        panic!("identity-only relation")
    };
    assert!(implications.is_empty());
    assert_eq!(implications.capacity(), limits.documents.lines);
    assert!(core.authority.rules().implies.allows(u16::MAX, u16::MAX, u16::MAX));
}
