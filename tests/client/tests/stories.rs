use jig_client_view::Role;
use jig_client_world::{Until, stories};
use jig_fake_person::{Find, Step};
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
    assert!(w.kept(c).iter().any(|r| matches!(r,jig_test_client_domain::store::Record::Jig(jig_client::store::Record::Draft { words,.. }) if words.bytes()==b"Unsent draft")));
    assert!(w.tree(c).nodes().iter().any(|n| n.value.as_ref().is_some_and(|v| v.text.as_ref() == b"Unsent draft")));
}
