use jig_client_world::{ClientId, Scripted, Test, Until, World, stories};
use skein_fake_person::{Find, Step};
use skein_lib::{Duration, Time};
use skein_ui::Role;
type ShellWorld = World<Test, Scripted>;
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
        jig_client_test_domain::Event::Jig(jig_client_view::Event::Ask {
            ask: jig_client_view::intents::ByValue {
                asked: skein_lib::Token::new(99),
                scope: 7,
                ask: Box::new(jig_client_domain::contract::Ask::StartChat {
                    project: 1,
                    words: stories::text(b"Hello"),
                }),
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
        w.kept(c).iter().all(|r| !matches!(r, jig_client_test_domain::store::Record::Unlinked { .. })),
        "answered keys are erased"
    );
}

#[test]
fn a_person_signs_out_from_the_frame_and_sees_the_sign_in_page() {
    let (mut w, c) = person_at_the_frame();
    w.person(
        c,
        vec![
            Step::See { find: sign_out_button(), within: Duration::from_secs(1) },
            Step::Press { find: sign_out_button() },
            Step::See { find: sign_in_button(), within: Duration::from_secs(1) },
        ],
    );
    let result = w.run(Until::Done);
    assert_eq!(result.verdict, skein_world::domain::Verdict::Passed);
}

fn sign_out_button() -> Find {
    Find::named(Role::Button, b"Sign out")
}

fn person_at_the_frame() -> (ShellWorld, ClientId) {
    let mut world = stories::chat_ready(5);
    let client = world.client(7, stories::inbox());
    (world, client)
}
fn sign_in_button() -> Find {
    Find::named(Role::Button, b"Sign in with Test")
}
#[test]
fn a_lost_sign_out_keeps_the_person_signed_in_and_says_so() {
    let (mut w, c) = person_at_the_frame();
    w.front().lose_next_sign_out();
    w.person(
        c,
        vec![
            Step::See { find: sign_out_button(), within: Duration::from_secs(1) },
            Step::Press { find: sign_out_button() },
            Step::See { find: lost_sign_out_notice(), within: Duration::from_secs(1) },
            Step::See { find: sign_out_button(), within: Duration::from_secs(1) },
            Step::Gone { find: sign_in_button(), within: Duration::from_secs(1) },
        ],
    );
    assert_eq!(w.run(Until::Done).verdict, skein_world::domain::Verdict::Passed);
    assert!(signed_in(&mut w));
    w.person(
        c,
        vec![
            Step::Press { find: sign_out_button() },
            Step::See { find: sign_in_button(), within: Duration::from_secs(1) },
            Step::Gone { find: lost_sign_out_notice(), within: Duration::from_secs(1) },
        ],
    );
    assert_eq!(w.run(Until::Done).verdict, skein_world::domain::Verdict::Passed);
    assert!(!signed_in(&mut w));
}

fn lost_sign_out_notice() -> Find {
    Find::text(b"Not signed out: the engine was not reached")
}

fn signed_in(world: &mut ShellWorld) -> bool {
    world.front().engine.truth().sign_ins.contains_key(&7)
}
