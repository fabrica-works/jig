use jig_client_world::{Until, stories};
use jig_fake_person::{Find, Step};
use jig_view::Role;
use skein_lib::Duration;
const WITHIN: Duration = Duration::from_secs(1);
#[test]
fn signing_in_and_starting_a_chat() {
    let mut w = stories::chat_world(1);
    w.front()
        .engine
        .act(skein_lib::Time::ZERO, skein_lib::Wall::EPOCH, jig_scripted_engine::Change::SignedOut { sign_in: 7 })
        .expect("sign out");
    let c = w.client(7, stories::conversation());
    w.person(
        c,
        vec![
            Step::See { find: Find::named(Role::Button, b"Sign in with Test"), within: WITHIN },
            Step::Press { find: Find::named(Role::Button, b"Sign in with Test") },
            Step::See { find: Find::named(Role::TextArea, b"First words"), within: WITHIN },
            Step::Type { find: Find::named(Role::TextArea, b"First words"), words: Box::from(*b"Hello") },
            Step::Press { find: Find::named(Role::Button, b"Start a conversation") },
            Step::See { find: Find::text(b"Chat nine"), within: WITHIN },
        ],
    );
    let outcome = w.run(Until::Done);
    assert!(outcome.callbacks > 4);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    assert_eq!(w.front().engine.commits().iter().filter(|commit| commit.key.is_some()).count(), 1);
}
#[test]
fn a_draft_is_kept_across_a_reload() {
    let mut w = stories::chat_ready(2);
    let c = w.client(7, stories::chat());
    w.person(
        c,
        vec![
            Step::See { find: Find::named(Role::Button, b"Write"), within: WITHIN },
            Step::Press { find: Find::named(Role::Button, b"Write") },
            Step::Type { find: Find::named(Role::TextArea, b"Your answer"), words: Box::from(*b"Unsent draft") },
            Step::Pause { span: Duration::from_millis(30) },
            Step::Reload,
            Step::See { find: Find::named(Role::Button, b"Write"), within: WITHIN },
            Step::Press { find: Find::named(Role::Button, b"Write") },
        ],
    );
    w.run(Until::Done);
    assert!(w.kept(c).iter().any(|r| matches!(r,jig_test_domain_client::store::Record::Jig(jig_domain_client::store::Record::Draft { words,.. }) if words.bytes()==b"Unsent draft")));
    assert!(w.tree(c).nodes().iter().any(|n| n.value.as_ref().is_some_and(|v| v.text.as_ref() == b"Unsent draft")));
}

#[test]
fn a_held_chat_is_released() {
    for seed in [11, 12] {
        let mut w = stories::held_chat(seed);
        let c = w.client(7, stories::chat());
        w.person(
            c,
            vec![
                Step::See { find: Find::named(Role::Button, b"Release"), within: WITHIN },
                Step::Press { find: Find::named(Role::Button, b"Release") },
                Step::See { find: Find::text(b"Being decided"), within: WITHIN },
                Step::Press { find: Find::named(Role::Button, b"Confirm") },
                Step::Press { find: Find::named(Role::Button, b"Overview") },
                Step::See { find: Find::text(b"Starting"), within: WITHIN },
            ],
        );
        w.run(Until::Done);
        assert_eq!(w.front().engine.commits().iter().filter(|c| c.key.is_some()).count(), 1);
    }
}

#[test]
fn a_proposal_accepted_in_the_inbox_leaves_its_chat_too() {
    for seed in [13, 14] {
        let mut w = stories::proposal_world(seed);
        let inbox = w.client(7, stories::inbox());
        let chat = w.client(7, stories::chat());
        w.person(
            inbox,
            vec![
                Step::See { find: Find::named(Role::Button, b"Accept"), within: WITHIN },
                Step::Press { find: Find::named(Role::Button, b"Accept") },
                Step::Press { find: Find::named(Role::Button, b"Confirm") },
                Step::See { find: Find::text(b"Accepted"), within: WITHIN },
            ],
        );
        w.person(
            chat,
            vec![
                Step::See { find: Find::text(b"Resolve the incident"), within: WITHIN },
                Step::See { find: Find::text(b"Accepted"), within: WITHIN },
            ],
        );
        w.run(Until::Done);
        assert_eq!(w.front().engine.commits().iter().filter(|c| c.key.is_some()).count(), 1);
    }
}

#[test]
fn two_tabs_deciding_one_thing_are_told_who_decided_first() {
    for seed in [15, 16] {
        let mut w = stories::proposal_world(seed);
        let a = w.client(7, stories::inbox());
        let b = w.tab(a);
        for c in [a, b] {
            w.person(
                c,
                vec![
                    Step::See { find: Find::named(Role::Button, b"Accept"), within: WITHIN },
                    Step::Press { find: Find::named(Role::Button, b"Accept") },
                    Step::Press { find: Find::named(Role::Button, b"Confirm") },
                    Step::See { find: Find::text(b"Accepted"), within: WITHIN },
                ],
            );
        }
        w.run(Until::Done);
        assert_eq!(w.front().engine.commits().iter().filter(|c| c.key.is_some()).count(), 2);
        assert!([a, b].into_iter().any(|c| {
            w.tree(c)
                .nodes()
                .iter()
                .flat_map(|n| &n.text)
                .any(|r| r.text.windows(b"Decided first".len()).any(|part| part == b"Decided first"))
        }));
    }
}

fn start_with_fault(seed: u64, fault: Option<jig_client_world::FrontFault>, reload: bool) {
    let mut w = stories::chat_world(seed);
    let c = w.client(7, stories::conversation());
    w.person(c, stories::start_chat_steps());
    w.run(Until::Done);
    if let Some(fault) = fault {
        w.fault(fault);
    }
    if reload {
        w.reload(c);
    }
    w.person(c, vec![Step::See { find: Find::text(b"Chat nine"), within: WITHIN }]);
    w.run(Until::Done);
    assert_eq!(w.front().engine.commits().iter().filter(|c| c.key.is_some()).count(), 1);
}
#[test]
fn a_reload_with_requests_pending_sends_them_again_under_their_keys() {
    for seed in [17, 18] {
        start_with_fault(seed, None, true);
    }
}
#[test]
fn the_engine_restarting_under_a_pending_request_answers_it_once() {
    for seed in [19, 20] {
        start_with_fault(
            seed,
            Some(jig_client_world::FrontFault::Restart { offline: Duration::from_millis(120) }),
            false,
        );
    }
}
#[test]
fn a_dropped_watch_shows_the_page_behind_until_it_reopens() {
    for seed in [21, 22] {
        let mut w = stories::chat_ready(seed);
        let c = w.client(7, stories::chat());
        w.person(c, vec![Step::See { find: Find::text(b"Chat nine"), within: WITHIN }]);
        w.run(Until::Done);
        w.fault(jig_client_world::FrontFault::DropWatch);
        w.person(
            c,
            vec![
                Step::See { find: Find::text(b"Behind"), within: WITHIN },
                Step::Gone { find: Find::text(b"Behind"), within: WITHIN },
            ],
        );
        w.run(Until::Done);
    }
}
#[test]
fn a_page_left_with_reads_in_flight_drops_their_answers() {
    for seed in [23, 24] {
        let mut w = stories::proposal_world(seed);
        let c = w.client(7, stories::inbox());
        w.person(c, vec![Step::See { find: Find::text(b"Live"), within: WITHIN }]);
        w.run(Until::Done);
        w.fault(jig_client_world::FrontFault::Slow { span: Duration::from_millis(50) });
        w.go(c, stories::chat());
        w.go(c, stories::conversation());
        w.run(Until::Time(w.now().saturating_add(Duration::from_millis(80))));
        assert!(w.tree(c).nodes().iter().all(|n| n.name.as_deref() != Some(b"Accept".as_slice())));
        assert!(w.front().engine.commits().iter().all(|c| c.key.is_none()));
    }
}

#[test]
fn a_goal_beyond_the_allotment_is_proposed_as_the_confirmation_said() {
    for seed in [25, 26] {
        let mut w = stories::goal_world(seed);
        let c = w.client(
            7,
            jig_test_domain_client::Address::Jig(jig_domain_client::address::Address::Project {
                project: 1,
                section: jig_domain_client::address::ProjectSection::Goals,
            }),
        );
        w.person(
            c,
            vec![
                Step::See { find: Find::named(Role::TextArea, b"Goal specification"), within: WITHIN },
                Step::Type {
                    find: Find::named(Role::TextArea, b"Goal specification"),
                    words: Box::from(*b"Resolve incident"),
                },
                Step::Press { find: Find::named(Role::Button, b"Charter 1") },
                Step::Type { find: Find::named(Role::TextField, b"Goal budget"), words: Box::from(*b"2") },
                Step::See { find: Find::text(b"Beyond your pool, this will be proposed"), within: WITHIN },
                Step::Press { find: Find::named(Role::Button, b"Set goal") },
                Step::See { find: Find::text(b"proposed"), within: WITHIN },
                Step::Press { find: Find::named(Role::Button, b"Confirm") },
                Step::See { find: Find::text(b"Proposed by task"), within: WITHIN },
            ],
        );
        w.run(Until::Done);
        let commits = w.front().engine.commits();
        let requests: Vec<_> = commits.iter().filter_map(|c| c.ask.as_ref()).collect();
        assert_eq!(requests.len(), 1);
        assert!(
            matches!(requests[0], jig_domain_client::contract::Ask::SetGoal { budget: 2, spec, .. } if spec.bytes() == b"Resolve incident")
        );
        assert!(matches!(
            commits.iter().find(|c| c.key.is_some()).expect("incident fixture fits and exists").answer,
            Some(jig_domain_client::contract::Answer::Done(jig_domain_client::contract::Outcome::Proposed { .. }))
        ));
    }
}

#[test]
fn a_procedure_only_goal_form_confirms_and_sends_the_supplied_executor_once() {
    for seed in [51, 52] {
        let mut w = stories::procedure_goal_world(seed);
        let c = w.client(
            7,
            jig_test_domain_client::Address::Jig(jig_domain_client::address::Address::Project {
                project: 1,
                section: jig_domain_client::address::ProjectSection::Goals,
            }),
        );
        w.person(
            c,
            vec![
                Step::See { find: Find::named(Role::TextArea, b"Goal specification"), within: WITHIN },
                Step::Type {
                    find: Find::named(Role::TextArea, b"Goal specification"),
                    words: Box::from(*b"Resolve incident"),
                },
                Step::Press { find: Find::named(Role::Button, b"Connector 2, procedure 7") },
                Step::Type { find: Find::named(Role::TextField, b"Goal budget"), words: Box::from(*b"2") },
                Step::Press { find: Find::named(Role::Button, b"Set goal") },
                Step::See { find: Find::text(b"Connector 2, procedure 7"), within: WITHIN },
                Step::See { find: Find::text(b"proposed"), within: WITHIN },
                Step::Press { find: Find::named(Role::Button, b"Confirm") },
                Step::See { find: Find::text(b"Proposed by"), within: WITHIN },
            ],
        );
        let result = w.run(Until::Done);
        assert_eq!(result.verdict, skein_world::domain::Verdict::Passed);
        let commits = w.front().engine.commits();
        let requests: Vec<_> = commits.iter().filter_map(|c| c.ask.as_ref()).collect();
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0],
            &jig_domain_client::contract::Ask::SetGoal {
                project: 1,
                spec: stories::text(b"Resolve incident"),
                executor: jig_domain_client::contract::GoalExecutor::Procedure { connector: 2, code: 7 },
                budget: 2,
                priority: 0,
                deadline: None,
            }
        );
        assert!(matches!(
            commits.iter().find(|c| c.key.is_some()).expect("the procedure goal has one keyed commit").answer,
            Some(jig_domain_client::contract::Answer::Done(jig_domain_client::contract::Outcome::Proposed { .. }))
        ));
    }
}
