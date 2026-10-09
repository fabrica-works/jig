use jig_client_world::{Until, stories};
use skein_lib::Time;
#[test]
fn separate_tabs_reload_and_drop_old_callbacks_while_history_moves_both_ways() {
    let mut w = stories::chat_ready(3);
    let a = w.client(7, stories::inbox());
    let b = w.tab(a);
    w.run(Until::Time(Time::from_nanos(50_000_000)));
    w.go(a, stories::chat());
    w.reload(a);
    w.back(a);
    w.forward(a);
    let result = w.run(Until::Time(Time::from_nanos(100_000_000)));
    assert!(!w.tree(a).is_empty());
    assert!(!w.tree(b).is_empty());
    assert!(result.callbacks > 6);
}
#[test]
fn a_failed_store_stops_the_steps_keyed_sends_and_notifies_the_table() {
    let mut w = stories::chat_world(4);
    let c = w.client(7, stories::conversation());
    w.run(Until::Time(Time::from_nanos(30_000_000)));
    w.fail_store();
    w.event(
        c,
        jig_test_client_domain::Event::Jig(jig_client::Event::Ask {
            ask: jig_client::intents::ByValue {
                asked: skein_lib::Token::new(99),
                scope: 7,
                ask: Box::new(jig_client::contract::Ask::StartChat { project: 1, words: stories::text(b"Hello") }),
            },
        }),
    );
    assert!(w.front().engine.commits().iter().all(|c| c.key.is_none()), "failed write withheld the send");
    w.run(Until::Time(Time::from_nanos(100_000_000)));
    assert_eq!(
        w.front().engine.commits().iter().filter(|c| c.key.is_some()).count(),
        1,
        "same key retries after lost envelope"
    );
    assert!(
        w.kept(c).iter().all(|r| !matches!(r, jig_test_client_domain::store::Record::Table { .. })),
        "answered keys are erased"
    );
}
