use jig_client::address::{Address, ProjectSection as P, TaskSection as T};
use jig_client_world::{Settings, Until, stories};
use skein_lib::{Time, Wall};
use skein_world::domain::heap::Counting;
#[global_allocator]
static HEAP: Counting = Counting;
fn full_contract() -> jig_client::contract::TaskContractDocument {
    use jig_client::contract::{FieldRule, TaskContractDocument, VerdictChoice};
    let fields = jig_client_world::test::list([
        FieldRule { name: vec![0xff; 32].into_boxed_slice(), max: u32::MAX },
        FieldRule { name: vec![0xfe; 32].into_boxed_slice(), max: u32::MAX },
    ]);
    let choice = VerdictChoice { code: u32::MAX, words: u32::MAX, followups: u32::MAX, fields };
    TaskContractDocument::Verdict { choices: jig_client_world::test::list([choice.clone(), choice]), cut: true }
}
fn full_task() -> jig_client::contract::Task {
    let mut task = stories::task();
    task.goal = Some(u64::MAX);
    task.result_contract = full_contract();
    task.chip.title = jig_client::contract::DocumentText::complete(stories::text(&[b't'; 32]));
    task.lineage = jig_client_world::test::list([8, 7]);
    task.subscriptions.entries = jig_client_world::test::list([
        jig_client::contract::Subscription {
            number: 1,
            kind: jig_client::contract::SubscriptionKind::Topic { connector: jig_test_client_domain::FIRST, topic: 1 },
        },
        jig_client::contract::Subscription {
            number: 2,
            kind: jig_client::contract::SubscriptionKind::Topic { connector: jig_test_client_domain::SECOND, topic: 2 },
        },
    ]);
    task.parts = jig_client_world::test::list(
        (1..=8).map(|n| jig_client::contract::Part::Panel { connector: jig_test_client_domain::FIRST, task: n }),
    );
    task.resources = jig_client_world::test::list(
        (1..=8).map(|n| jig_client::contract::Part::Panel { connector: jig_test_client_domain::SECOND, task: n }),
    );
    task.spec.specification.words.bytes = stories::text(&[b's'; 32]);
    task.instructions = Some(jig_client::contract::DocumentText::complete(stories::text(&[b'i'; 32])));
    task.dependencies = jig_client_world::test::list([
        jig_client::contract::OwnerTaskLink::Live(task.chip.clone()),
        jig_client::contract::OwnerTaskLink::Live(task.chip.clone()),
    ]);
    task.references = task.dependencies.clone();
    task.inputs = task.dependencies.clone();
    task.lineage_chips = jig_client_world::test::list([task.chip.clone(), task.chip.clone()]);
    task.delegates = jig_client_world::test::list([
        jig_client::contract::LiveDelegate { chip: task.chip.clone(), takeover_budget: 1 },
        jig_client::contract::LiveDelegate { chip: task.chip.clone(), takeover_budget: 1 },
    ]);
    task.writer_slots = jig_client_world::test::list([
        jig_client::contract::WriterSlot {
            resource: jig_client::contract::Part::Resource {
                connector: jig_test_client_domain::FIRST,
                path: {
                    let mut rows = skein_lib::List::with_capacity(1);
                    rows.push(stories::text(&[b'r'; 32]).into_bytes())
                        .expect("one supplied segment fits its reserved slot");
                    rows
                },
            },
            task: 9,
            attempt: Some(1),
        },
        jig_client::contract::WriterSlot {
            resource: jig_client::contract::Part::Resource {
                connector: jig_test_client_domain::FIRST,
                path: {
                    let mut rows = skein_lib::List::with_capacity(1);
                    rows.push(stories::text(&[b'r'; 32]).into_bytes())
                        .expect("one supplied segment fits its reserved slot");
                    rows
                },
            },
            task: 9,
            attempt: Some(2),
        },
    ]);
    task
}
fn full_world() -> jig_client_world::World<jig_client_world::Test, jig_client_world::Scripted> {
    let mut w = stories::configured(Settings { memory: true, seed: 5, ..Settings::default() });
    let task = full_task();
    w.front()
        .engine
        .act(
            Time::ZERO,
            Wall::EPOCH,
            jig_scripted_engine::Change::Object {
                project: 1,
                key: jig_client::contract::ObjectKey::Task(9),
                value: jig_client::contract::ReadValue::Task(Box::new(task)),
            },
        )
        .expect("full task fits");
    let mut frame = stories::frame(7);
    frame.party.name = jig_client::contract::DocumentText::complete(stories::text(&[b'p'; 32]));
    frame.party.login = jig_client::contract::DocumentText::complete(stories::text(&[b'l'; 32]));
    let mut project = frame.projects.iter().next().expect("project exists").clone();
    project.name = Some(jig_client::contract::DocumentText::complete(stories::text(&[b'p'; 32])));
    project.role = jig_client::contract::HeldRole::Policy { role: 1 };
    let mut other = project.clone();
    other.project = 2;
    other.role = jig_client::contract::HeldRole::Policy { role: 2 };
    frame.projects = jig_client_world::test::list([project, other]);
    let mut offer = frame.charters.iter().next().expect("charter offers exist").clone();
    offer.chats = jig_client_world::test::list([
        jig_client::contract::Charter {
            number: 1,
            name: Some(jig_client::contract::DocumentText::complete(stories::text(&[b'c'; 32]))),
        },
        jig_client::contract::Charter {
            number: 2,
            name: Some(jig_client::contract::DocumentText::complete(stories::text(&[b'c'; 32]))),
        },
    ]);
    offer.goals = offer.chats.clone();
    let mut other_offer = offer.clone();
    other_offer.project = 2;
    frame.charters = jig_client_world::test::list([offer, other_offer]);
    w.front().engine.act(Time::ZERO, Wall::EPOCH, jig_scripted_engine::Change::Frame(frame)).expect("full frame fits");
    let mut family = jig_client_world::scripted::Family::default();
    for n in 1..=8 {
        let subject = jig_test_client_domain::family::Subject {
            scope: jig_test_client_domain::family::Scope::Project(1),
            kind: jig_test_client_domain::family::Kind::Panel(n),
        };
        family.documents.insert(
            subject,
            jig_test_client_domain::family::Document::Part(jig_test_client_domain::family::PartDocument {
                subject,
                revision: jig_client::contract::Revision { value: 1 },
                value: n,
                parameters: None,
            }),
        );
    }
    let subject = jig_test_client_domain::family::Subject {
        scope: jig_test_client_domain::family::Scope::Project(1),
        kind: jig_test_client_domain::family::Kind::Item(1),
    };
    family.documents.insert(
        subject,
        jig_test_client_domain::family::Document::Item(jig_test_client_domain::family::Item {
            project: 1,
            key: 1,
            revision: jig_client::contract::Revision { value: 1 },
            state: jig_test_client_domain::family::ItemState::Ready,
            value: 1,
            task: 9,
            proposals: jig_client_world::test::list([]),
            touchable: true,
        }),
    );
    w.front()
        .engine
        .act_application(Time::ZERO, Wall::EPOCH, jig_test_client_domain::FIRST, 1, family, vec![])
        .expect("full family fits");
    for change in [
        jig_scripted_engine::Change::Project(Box::new(stories::project())),
        jig_scripted_engine::Change::Deployment(stories::deployment()),
    ] {
        w.front().engine.act(Time::ZERO, Wall::EPOCH, change).expect("full page fixture fits");
    }
    w
}
#[test]
fn full_documents_tabs_and_all_page_kinds_fit_the_client_and_view_worst_cases() {
    let mut w = full_world();
    let c = w.client(7, stories::inbox());
    let _tab = w.tab(c);
    w.run(Until::Time(Time::from_nanos(50_000_000)));
    let mut addresses = vec![
        stories::conversation(),
        jig_test_client_domain::Address::Jig(Address::Deployment),
        jig_test_client_domain::Address::First(jig_test_client_domain::first::Address { project: 1, item: 1 }),
    ];
    addresses.extend(
        [T::Overview, T::Activity, T::Plan, T::History, T::Budget, T::Panels]
            .map(|section| jig_test_client_domain::Address::Jig(Address::Task { task: 9, section })),
    );
    addresses.extend(
        [
            P::Overview,
            P::Goals,
            P::Tasks,
            P::Notes,
            P::Settings,
            P::Resources,
            P::Policy,
            P::Roles,
            P::Spend,
            P::Recurring,
        ]
        .map(|section| jig_test_client_domain::Address::Jig(Address::Project { project: 1, section })),
    );
    for address in addresses {
        let task = address == stories::chat();
        w.go(c, address);
        w.run(Until::Time(w.now().saturating_add(skein_lib::Duration::from_millis(20))));
        if task {
            w.person(
                c,
                vec![jig_fake_person::Step::See {
                    find: jig_fake_person::Find::text(&[b't'; 32]),
                    within: skein_lib::Duration::from_secs(1),
                }],
            );
            w.run(Until::Done);
        }
    }
    // Each field reaches its text envelope, then one more proves refusal is bounded.
    for field in [
        jig_client::fields::Purpose::NewChat,
        jig_client::fields::Purpose::Answer,
        jig_client::fields::Purpose::Reason,
        jig_client::fields::Purpose::Amendment,
        jig_client::fields::Purpose::Instructions,
    ] {
        w.event(
            c,
            jig_test_client_domain::Event::Jig(jig_client::Event::Act {
                act: jig_client::action::Act::Edit { field, text: stories::text(&[b'f'; 32]) },
            }),
        );
    }
    // The added owner count reaches every distinct restored-scope slot too.
    let counted = w.client(7, stories::inbox());
    for scope in [7, 8, 9, 10] {
        w.event(counted, jig_test_client_domain::Event::Jig(jig_client::Event::Kept { scope, requests: 1 }));
    }
    let result = w.run(Until::Steps(2));
    eprintln!("full client fixture: heap peak {} bytes, bound {} bytes", result.heap_peak, result.heap_bound);
    assert!(result.heap_peak > 0, "counted every iteration");
    assert!(result.heap_peak <= result.heap_bound, "composed root, view and shell model envelope");
}

/// A real client allocation crosses a deliberately small declared budget.
/// The same callback guard used by the full fixture must catch it.
#[test]
#[should_panic(expected = "bounded: client and view exceeded the heap envelope")]
fn a_client_that_exceeds_its_declared_heap_budget_is_caught() {
    let mut world = stories::configured(Settings { memory: true, heap_bytes_per_client: 1024, ..Settings::default() });
    world.client(7, stories::inbox());
}
