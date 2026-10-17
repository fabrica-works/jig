use jig_client_world::{ClientId, Fault, FrontFault, Scripted, Test, Until, World, stories};
use skein_fake_person::{Face, Find, Found, Step, TreeFace};
use skein_lib::{Duration, Time, Wall};
use skein_ui::Act;
use skein_ui::{NodeId, Role};
fn node(w: &World<Test, Scripted>, c: ClientId, find: &Find) -> NodeId {
    let Found::Node(node) = TreeFace::new(w.tree(c)).find(find) else {
        panic!("missing {find}");
    };
    node
}
fn prepared(seed: u64) -> (World<Test, Scripted>, ClientId) {
    let mut w = stories::chat_world(seed);
    let c = w.client(7, stories::conversation());
    w.person(
        c,
        vec![
            Step::See { find: Find::named(Role::TextArea, b"First words"), within: Duration::from_secs(1) },
            Step::Type { find: Find::named(Role::TextArea, b"First words"), words: Box::from(*b"Hello") },
        ],
    );
    w.run(Until::Done);
    (w, c)
}
fn begin(w: &mut World<Test, Scripted>, c: ClientId) {
    let n = node(w, c, &Find::named(Role::Button, b"Start a conversation"));
    w.act(c, n, Act::Press);
}
fn finish(w: &mut World<Test, Scripted>, c: ClientId) {
    w.person(c, vec![Step::See { find: Find::text(b"Chat nine"), within: Duration::from_secs(1) }]);
    let result = w.run(Until::Done);
    assert_eq!(result.verdict, skein_world::domain::Verdict::Passed);
    assert_eq!(
        w.front().engine.commits().iter().filter(|c| c.key.is_some()).count(),
        1,
        "one durable key despite the fault"
    );
}
#[test]
fn busy_is_retried_under_the_original_key() {
    let (mut w, c) = prepared(31);
    w.fault(FrontFault::Busy);
    begin(&mut w, c);
    finish(&mut w, c);
}
#[test]
fn not_ready_is_retried_under_the_original_key() {
    let (mut w, c) = prepared(32);
    w.fault(FrontFault::NotReady);
    begin(&mut w, c);
    finish(&mut w, c);
}
#[test]
fn slow_delivery_preserves_the_request_and_its_outcome() {
    let (mut w, c) = prepared(33);
    w.fault(FrontFault::Slow { span: Duration::from_millis(50) });
    begin(&mut w, c);
    finish(&mut w, c);
}
#[test]
fn a_lost_answer_keeps_the_durable_key_once() {
    let (mut w, c) = prepared(34);
    begin(&mut w, c);
    w.fault(FrontFault::NoAnswer);
    finish(&mut w, c);
}
#[test]
fn a_restart_keeps_committed_and_pending_keys_once() {
    let (mut w, c) = prepared(35);
    begin(&mut w, c);
    w.fault(FrontFault::Restart { offline: Duration::from_millis(120) });
    finish(&mut w, c);
}
#[test]
fn a_double_press_emits_one_decision() {
    let (mut w, c) = prepared(36);
    let n = node(&w, c, &Find::named(Role::Button, b"Start a conversation"));
    w.fault(Fault::DoublePress { client: c, node: n });
    finish(&mut w, c);
}
#[test]
fn a_reload_retains_the_request_before_resending() {
    let (mut w, c) = prepared(37);
    begin(&mut w, c);
    assert!(w.kept(c).iter().any(|r| matches!(r, jig_client_test_domain::store::Record::Unlinked(..))));
    w.fault(Fault::Reload { client: c });
    finish(&mut w, c);
}
#[test]
fn a_stale_press_is_dropped_without_a_callback() {
    let mut w = stories::chat_ready(38);
    let c = w.client(7, stories::chat());
    w.run(Until::Time(Time::from_nanos(30_000_000)));
    let n = node(&w, c, &Find::named(Role::Button, b"Write"));
    w.go(c, stories::inbox());
    assert!(w.tree(c).find(n).is_none());
    let before = w.run(Until::Steps(1)).callbacks;
    w.fault(Fault::StalePress { client: c, node: n });
    assert_eq!(w.run(Until::Steps(1)).callbacks, before);
    assert!(w.front().engine.commits().iter().all(|c| c.key.is_none()));
}
#[test]
fn a_dropped_watch_marks_the_page_behind_then_reopens() {
    let mut w = stories::chat_ready(39);
    let c = w.client(7, stories::chat());
    w.run(Until::Time(Time::from_nanos(30_000_000)));
    w.fault(FrontFault::DropWatch);
    w.person(
        c,
        vec![
            Step::See { find: Find::text(b"Behind"), within: Duration::from_secs(1) },
            Step::Gone { find: Find::text(b"Behind"), within: Duration::from_secs(1) },
        ],
    );
    w.run(Until::Done);
}
#[test]
fn an_object_gone_under_its_card_reports_why_it_left() {
    let mut w = stories::proposal_world(40);
    let c = w.client(7, stories::inbox());
    w.run(Until::Time(Time::from_nanos(30_000_000)));
    w.fault(FrontFault::ObjectGone);
    w.person(c, vec![Step::See { find: Find::text(b"Ended; its final facts"), within: Duration::from_secs(1) }]);
    w.run(Until::Done);
}
#[test]
fn signed_out_pending_work_parks_and_resumes_for_its_party() {
    let (mut w, c) = prepared(41);
    begin(&mut w, c);
    w.fault(FrontFault::SignedOut);
    w.person(
        c,
        vec![
            Step::See { find: Find::named(Role::Button, b"Sign in with Test"), within: Duration::from_secs(1) },
            Step::Press { find: Find::named(Role::Button, b"Sign in with Test") },
        ],
    );
    finish(&mut w, c);
}
#[test]
fn another_party_keeps_the_original_partys_work_parked_and_hidden() {
    let (mut w, c) = prepared(42);
    let now = w.now();
    w.front()
        .engine
        .act(now, Wall::EPOCH, jig_client_scripted_engine::Change::Me(stories::frame(8)))
        .expect("second party");
    w.run(Until::Time(now.saturating_add(Duration::from_millis(10))));
    begin(&mut w, c);
    w.fault(Fault::OtherParty { client: c, party: 8 });
    w.person(c, vec![Step::See { find: Find::text(b"Someone else's work"), within: Duration::from_secs(1) }]);
    w.run(Until::Done);
    assert!(w.kept(c).iter().any(|r| matches!(r, jig_client_test_domain::store::Record::Unlinked(..))));
    w.sign_as(c, 7);
    finish(&mut w, c);
}
#[test]
fn an_out_of_version_engine_keeps_pending_work_for_an_update() {
    let (mut w, c) = prepared(43);
    begin(&mut w, c);
    w.fault(FrontFault::Versions);
    w.person(c, vec![Step::See { find: Find::text(b"out of date"), within: Duration::from_secs(1) }]);
    w.run(Until::Done);
}

#[test]
fn two_tabs_pressing_the_same_card_receive_the_first_decision() {
    let mut w = stories::proposal_world(44);
    let a = w.client(7, stories::inbox());
    let b = w.tab(a);
    for c in [a, b] {
        w.person(c, vec![Step::See { find: Find::named(Role::Button, b"Accept"), within: Duration::from_secs(1) }]);
    }
    w.run(Until::Done);
    let first_node = node(&w, a, &Find::named(Role::Button, b"Accept"));
    let second_node = node(&w, b, &Find::named(Role::Button, b"Accept"));
    w.fault(Fault::TwoTabs { first: a, first_node, second: b, second_node });
    for c in [a, b] {
        w.person(
            c,
            vec![
                Step::Press { find: Find::named(Role::Button, b"Confirm") },
                Step::See { find: Find::text(b"Accepted"), within: Duration::from_secs(1) },
            ],
        );
    }
    w.run(Until::Done);
    assert_eq!(w.front().engine.commits().iter().filter(|c| c.key.is_some()).count(), 2);
    assert!(w.front().engine.commits().iter().any(|c| matches!(
        c.answer,
        Some(jig_client_domain::contract::Answer::Done(jig_client_domain::contract::Outcome::DecidedFirst { .. }))
    )));
}
