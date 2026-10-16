//! P and Q use the composed public client under the common shell referee.
use jig_client_domain::contract::{ObjectKey, ReadValue};
use jig_client_world::{Until, harness::App};
use jig_domain_client::address::{Address as JigAddress, TaskSection};
use jig_ops_client_domain::{Address, observability as obs};
use jig_ops_world::client::{Ops, stories};
use skein_fake_person::{Find, Step};
use skein_lib::Duration;
use skein_ui::Role;
use skein_world::domain::{Verdict, heap::Counting};
#[global_allocator]
static HEAP: Counting = Counting;
const WITHIN: Duration = Duration::from_secs(1);
fn see(words: &[u8]) -> Step<Address> {
    Step::See { find: Find::text(words), within: WITHIN }
}
fn press(words: &[u8]) -> Step<Address> {
    Step::Press { find: Find::named(Role::Button, words) }
}
#[test]
fn p_accepts_a_restart_from_the_incidents_page() {
    accept_restart(501);
}

#[test]
fn p_accepts_a_restart_from_the_incidents_page_with_a_second_seed() {
    accept_restart(502);
}

#[test]
fn p_accepts_the_restart_in_its_leads_plan_on_the_incidents_page() {
    let mut world = stories::incident_world(503);
    let p = world.client(7, stories::incident());
    world.person(
        p,
        vec![
            see(b"R restart checkout"),
            see(b"Restart checkout in production"),
            press(b"Accept"),
            press(b"Confirm"),
            see(b"Accepted"),
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    assert_eq!(world.front().engine.commits().iter().filter(|c| c.key.is_some()).count(), 1);
}

#[test]
fn p_accepts_the_restart_from_the_incidents_page_at_the_text_shell() {
    let mut world = stories::incident_world(504);
    let p = world.client(7, stories::incident());
    world.at_text_shell(p);
    world.person(
        p,
        vec![
            see(b"R restart checkout"),
            see(b"Restart checkout in production"),
            press(b"Accept"),
            press(b"Confirm"),
            see(b"Accepted"),
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    assert_eq!(world.front().engine.commits().iter().filter(|c| c.key.is_some()).count(), 1);
}

fn accept_restart(seed: u64) {
    let mut world = stories::incident_world(seed);
    world.measure_heap();
    let p = world.client(7, Address::Observability(obs::Address::Incidents { project: 1, open: true }));
    let inbox = world.client(7, Address::Jig(JigAddress::Inbox { project: None }));
    world.person(
        p,
        vec![
            Step::See { find: Find::named(Role::Link, b"Incident 12"), within: WITHIN },
            Step::Press { find: Find::named(Role::Link, b"Incident 12") },
            Step::See {
                find: Find::text(b"Error rate 30").within(Box::from([skein_fake_person::Target::Role {
                    role: Role::Drawing,
                    name: b"Error rate".as_slice().into(),
                }])),
                within: WITHIN,
            },
            see(b"I incident lead"),
            Step::Go { address: stories::lead() },
            see(b"delegate"),
            Step::Go { address: Address::Jig(JigAddress::Task { task: 9, section: TaskSection::Plan }) },
            see(b"R restart checkout"),
            see(b"Restart checkout in production"),
            press(b"Accept"),
            press(b"Confirm"),
            see(b"Accepted"),
            Step::Go { address: Address::Jig(JigAddress::Task { task: 10, section: TaskSection::Panels }) },
            see(b"Restart service"),
            see(b"Waiting for condition"),
        ],
    );
    world.person(
        inbox,
        vec![
            see(b"Restart checkout in production"),
            see(b"Accepted"),
            Step::Pause { span: Duration::from_millis(50) },
            see(b"Accepted"),
            Step::Pause { span: Ops::limits().jig.linger.saturating_add(Duration::from_millis(100)) },
            Step::Gone { find: Find::text(b"Restart checkout in production"), within: WITHIN },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    stories::resolve(&mut world);
    world.person(p, vec![Step::Go { address: stories::incident() }, see(b"Resolved"), see(b"Incident resolved")]);
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    assert_eq!(world.front().engine.commits().iter().filter(|c| c.key.is_some()).count(), 1);
    let ReadValue::Task(lead) = &world.front().engine.truth().objects[&ObjectKey::Task(9)] else { unreachable!() };
    assert!(lead.result.is_some());
    assert_eq!(lead.chip.task, 9);
}

#[test]
fn q_asks_for_an_environment_in_a_chat_and_accepts_what_it_costs() {
    for seed in [503, 504] {
        let mut world = stories::environment_world(seed);
        world.measure_heap();
        let q = world.client(7, Address::Jig(JigAddress::Conversations { project: None }));
        world.person(
            q,
            vec![
                Step::See { find: Find::named(Role::TextArea, b"First words"), within: WITHIN },
                Step::Type {
                    find: Find::named(Role::TextArea, b"First words"),
                    words: b"staging for one week".as_slice().into(),
                },
                press(b"Start a conversation"),
                see(b"Q staging chat"),
                see(b"Provision staging for one week"),
                see(b"40"),
                press(b"Accept"),
                press(b"Confirm"),
                see(b"Accepted"),
            ],
        );
        assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
        stories::provision(&mut world);
        world.person(
            q,
            vec![
                Step::Go { address: stories::environment() },
                see(b"staging-7"),
                see(b"Ready"),
                see(b"Cost: 40"),
                Step::Type {
                    find: Find::named(Role::TextField, b"New end (seconds since epoch)"),
                    words: b"900".as_slice().into(),
                },
                press(b"Move end"),
                see(b"End moved"),
            ],
        );
        assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
        let state = world.front().engine.application();
        let subject = stories::environment_document();
        let doc = state
            .documents
            .get(&jig_ops_world::client::Subject::Infrastructure(jig_ops_client_domain::infrastructure::Subject {
                scope: jig_ops_client_domain::infrastructure::Scope::Project(1),
                kind: jig_ops_client_domain::infrastructure::Kind::Environment(7),
            }))
            .expect("original owner environment retained");
        let jig_ops_world::client::Document::Infrastructure(
            jig_ops_client_domain::infrastructure::Document::Environment(after),
        ) = doc
        else {
            unreachable!()
        };
        assert_eq!(after.end.expect("owner desired end").revision.value, 6);
        assert_eq!(after.end.expect("owner desired end").until.as_secs(), 900);
        assert_eq!(after.observed_until, subject.observed_until);
        assert_eq!(after.created_by, subject.created_by);
        assert_eq!(world.front().engine.commits().iter().filter(|c| c.key.is_some()).count(), 3);
    }
}
