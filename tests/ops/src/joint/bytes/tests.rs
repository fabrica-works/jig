use super::*;
use jig_client_world::harness::Front;
use skein_fake_person::{Find, Step};
use skein_lib::Duration;
use skein_ui::Role;
type World = jig_client_world::World<super::super::fixture::Client, Joint>;
fn see(words: &[u8]) -> Step<d::Address> {
    Step::See { find: Find::text(words), within: Duration::from_secs(3) }
}
fn linked_conversation(seed: u64) -> (World, ClientId) {
    let mut world = super::super::fixture::world(seed);
    let client = world.client_with(
        1,
        d::Address::Jig(jig_client_view::address::Address::Conversations { project: None }),
        super::super::fixture::limits(),
        super::super::fixture::client,
    );
    super::super::fixture::watch_setup(world.front());
    world.person(
        client,
        vec![
            Step::See { find: Find::named(Role::TextArea, b"First words"), within: Duration::from_secs(3) },
            Step::Type { find: Find::named(Role::TextArea, b"First words"), words: b"@watch_setup".as_slice().into() },
            Step::Press { find: Find::named(Role::Button, b"Start a conversation") },
            see(b"delegate"),
        ],
    );
    let outcome = world.run(jig_client_world::Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
    (world, client)
}
#[test]
fn a_linked_ops_root_starts_on_the_common_world_clock() {
    let (mut world, _) = linked_conversation(511);
    super::super::fixture::alert(world.front());
    let due = Time::from_nanos(world.front().engine.clock().now);
    let outcome = world.run(jig_client_world::Until::Time(due.saturating_add(Duration::from_secs(1))));
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
    assert!(outcome.now >= due);
}
fn incident(world: &mut World, client: ClientId) -> u64 {
    super::super::fixture::alert(world.front());
    let service =
        jig_ops_domain_infrastructure::Service::new(b"production".as_slice().into(), b"checkout".as_slice().into());
    let fact =
        world.front().engine.systems.infrastructure(jig_ops_domain_infrastructure::SystemRequest::Service { service });
    world.front().source(root::Event::Infrastructure(jig_ops_domain_infrastructure::Event::System(fact)));
    world.person(
        client,
        vec![
            Step::Go {
                address: d::Address::Observability(d::observability::Address::Incidents { project: 1, open: true }),
            },
            Step::See { find: Find::named(Role::Link, b"Incident 12"), within: Duration::from_secs(3) },
            Step::Press { find: Find::named(Role::Link, b"Incident 12") },
            see(b"Lead"),
        ],
    );
    let outcome = world.run(jig_client_world::Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
    world
        .front()
        .engine
        .store
        .rows
        .values()
        .find_map(|row| match row {
            root::Record::Observability(jig_ops_domain_observability::Record::Incident(value))
                if value.number == 12 =>
            {
                value.lead
            }
            root::Record::Core(_) | root::Record::Infrastructure(_) | root::Record::Observability(_) => None,
        })
        .expect("incident lead")
}
#[test]
fn an_incident_leads_to_its_task_activity_across_overlapping_exchanges() {
    let (mut world, client) = linked_conversation(511);
    let lead = incident(&mut world, client);
    world.person(
        client,
        vec![
            Step::Go {
                address: d::Address::Jig(jig_client_view::address::Address::Task {
                    task: lead,
                    section: jig_client_view::address::TaskSection::Activity,
                }),
            },
            see(b"read_series"),
        ],
    );
    let outcome = world.run(jig_client_world::Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
}
fn go(task: u64, section: jig_client_view::address::TaskSection) -> Step<d::Address> {
    Step::Go { address: d::Address::Jig(jig_client_view::address::Address::Task { task, section }) }
}
fn press(words: &[u8]) -> Step<d::Address> {
    Step::Press { find: Find::named(Role::Button, words) }
}
fn remediation(world: &mut World, lead: u64) -> u64 {
    world
        .front()
        .engine
        .store
        .rows
        .values()
        .find_map(|row| match row {
            root::Record::Core(core::Record::Tasks(
                jig_domain_tasks::Stored::Live(task) | jig_domain_tasks::Stored::Ended(task),
            )) if task.requester == jig_domain_tasks::Party::Task(lead) => Some(task.number),
            root::Record::Core(_) | root::Record::Infrastructure(_) | root::Record::Observability(_) => None,
        })
        .expect("remediation delegate")
}
fn offered(world: &mut World, client: ClientId, lead: u64) {
    use jig_client_view::address::{Address, TaskSection};
    world.person(
        client,
        vec![
            go(lead, TaskSection::Activity),
            see(b"read_series"),
            see(b"delegate"),
            Step::Go { address: d::Address::Jig(Address::Inbox { project: None }) },
            see(b"Restart service"),
            see(b"checkout"),
            see(b"production"),
        ],
    );
    let outcome = world.run(jig_client_world::Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
}
#[test]
fn an_accepted_restart_opens_its_remediation_panels() {
    use jig_client_view::address::{Address, TaskSection};
    let (mut world, client) = linked_conversation(511);
    let lead = incident(&mut world, client);
    offered(&mut world, client, lead);
    let remedy = remediation(&mut world, lead);
    world.person(
        client,
        vec![
            go(lead, TaskSection::Plan),
            see(b"restart checkout"),
            see(b"Restart service"),
            see(b"checkout"),
            see(b"production"),
            Step::See { find: Find::named(Role::Button, b"Accept"), within: Duration::from_secs(3) },
            press(b"Accept"),
            press(b"Confirm"),
            see(b"Accepted"),
            Step::Go { address: d::Address::Jig(Address::Inbox { project: None }) },
            Step::Gone { find: Find::text(b"Restart service"), within: Duration::from_secs(8) },
            see(b"0 need you"),
            go(remedy, TaskSection::Panels),
            see(b"Restart service"),
            see(b"Restarting"),
            see(b"Waiting for healthy"),
        ],
    );
    let outcome = world.run(jig_client_world::Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
    let outcome = world.run(jig_client_world::Until::Time(outcome.now.saturating_add(Duration::from_secs(8))));
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
}
fn environment_chat(seed: u64) -> (World, ClientId) {
    use jig_client_view::address::Address;
    let mut world = super::super::fixture::environment_world(seed);
    let client = world.client_with(
        1,
        d::Address::Jig(Address::Conversations { project: None }),
        super::super::fixture::limits(),
        super::super::fixture::client,
    );
    world.person(
        client,
        vec![Step::See { find: Find::named(Role::TextArea, b"First words"), within: Duration::from_secs(3) }],
    );
    let outcome = world.run(jig_client_world::Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
    world.person(
        client,
        vec![
            Step::See { find: Find::named(Role::TextArea, b"First words"), within: Duration::from_secs(3) },
            Step::Type {
                find: Find::named(Role::TextArea, b"First words"),
                words: b"@environment_chat staging for payments, a week".as_slice().into(),
            },
            press(b"Start a conversation"),
            see(b"propose"),
            Step::Go { address: d::Address::Jig(Address::Inbox { project: None }) },
            see(b"staging for payments"),
            Step::See { find: Find::named(Role::Button, b"Accept"), within: Duration::from_secs(3) },
            press(b"Accept"),
            see(b"40"),
            see(b"Funded by pool in project 1, person 1, period 1"),
            press(b"Confirm"),
            see(b"Accepted"),
        ],
    );
    let outcome = world.run(jig_client_world::Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
    (world, client)
}
#[test]
fn an_environment_chat_stays_live_while_its_provision_finishes() {
    let (mut world, client) = environment_chat(534);
    let until = world.now().saturating_add(Duration::from_secs(8));
    let outcome = world.run(jig_client_world::Until::Time(until));
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
    let native =
        jig_ops_domain_infrastructure::Environment::new(b"staging".as_slice().into(), b"payments".as_slice().into());
    let fact = world
        .front()
        .engine
        .systems
        .infrastructure(jig_ops_domain_infrastructure::SystemRequest::Environment { environment: native });
    world.front().source(root::Event::Infrastructure(jig_ops_domain_infrastructure::Event::System(fact)));
    world.person(
        client,
        vec![go(1, jig_client_view::address::TaskSection::Activity), see(b"staging/payments is ready")],
    );
    let outcome = world.run(jig_client_world::Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
}
#[test]
fn a_seeded_restart_reopens_its_incident_page() {
    let (mut world, client) = linked_conversation(537);
    world.front().engine.cut(jig_conformance::Cut::Random { seed: 0x0219 ^ 0x33 });
    let lead = incident(&mut world, client);
    assert!(lead > 0);
}
#[test]
fn random_navigation_keeps_each_waiting_card_accounted_for() {
    let (mut world, client) = linked_conversation(537);
    world.front().engine.cut(jig_conformance::Cut::Random { seed: 0x0219 ^ 0x33 });
    let lead = incident(&mut world, client);
    offered(&mut world, client, lead);
    world.person(
        client,
        vec![Step::Go {
            address: d::Address::Observability(d::observability::Address::Incidents { project: 1, open: true }),
        }],
    );
    world.run(jig_client_world::Until::Done);
    world.random_person_without_button(client, b"Sign out");
    let outcome = world.run(jig_client_world::Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
}
fn ready_environment(world: &mut World, client: ClientId) {
    let until = world.now().saturating_add(Duration::from_secs(8));
    let outcome = world.run(jig_client_world::Until::Time(until));
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
    let environment = d::infrastructure::native::Environment {
        pool: b"staging".as_slice().into(),
        name: b"payments".as_slice().into(),
    };
    world.person(
        client,
        vec![
            Step::Go {
                address: d::Address::Infrastructure(d::infrastructure::Address::NativeEnvironment {
                    project: 1,
                    environment,
                }),
            },
            see(b"Ready at (seconds since epoch):"),
            see(b"Revision 0"),
            Step::Type {
                find: Find::named(Role::TextField, b"New end (seconds since epoch)"),
                words: b"1209600".as_slice().into(),
            },
        ],
    );
    let outcome = world.run(jig_client_world::Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
}
#[test]
fn an_environment_answer_can_be_lost_at_its_native_byte_face() {
    let (mut world, client) = environment_chat(534);
    ready_environment(&mut world, client);
    world.fault(jig_client_world::harness::FrontFault::NoAnswer);
    world.person(client, vec![press(b"Move end")]);
    let until = world.now().saturating_add(Duration::from_secs(2));
    world.run(jig_client_world::Until::Time(until));
    assert_eq!(world.front().lost_final_answers, 1);
    world.reload(client);
    world.person(client, vec![see(b"Revision 1"), see(b"1209600")]);
    let outcome = world.run(jig_client_world::Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
}
#[test]
fn a_linked_ops_root_signs_in_through_the_byte_face() {
    let mut world = super::super::fixture::world(511);
    let joint = world.front();
    joint.bind(ClientId(0), 1);
    let limits = super::super::fixture::limits();
    let mut root = d::new(&limits, 71);
    let mut requests = Queue::with_capacity(d::max_out(&limits));
    let mut env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    d::step(
        &mut root,
        &env,
        d::Event::Start {
            address: d::Address::Jig(jig_client_view::address::Address::Conversations { project: None }),
            kept: skein_lib::List::with_capacity(0),
            offset: jig_client_view::boundary::Offset { minutes: 0 },
        },
        &mut requests,
    );
    joint.starting(ClientId(0));
    for _ in 0..300 {
        while let Some(request) = requests.pop() {
            if let d::Request::Client { request } = request {
                joint.wire(ClientId(0), env.now, env.wall, d::Request::Client { request });
            }
        }
        for (_, event) in joint.poll(env.now, env.wall) {
            d::step(&mut root, &env, event, &mut requests);
        }
        if root.client().link().state == jig_client::Link::Live {
            return;
        }
        env.now = env.now.saturating_add(skein_lib::Duration::from_millis(10));
        d::fire(&mut root, &env, &mut requests);
    }
    panic!("not live: {:?}", root.client().link());
}
