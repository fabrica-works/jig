#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::arithmetic_side_effects,
    reason = "step tests are ordinary Rust (testing-strategy.md, section 4)"
)]
use crate::*;
use skein_lib::{Duration, Env, List, Queue, ReplyTo, Time, Token, Wall};

const LIMITS: Limits = Limits {
    people: 4,
    inbox_entries: 2,
    inbox_tasks: 16,
    inbox_recipients: 64,
    sign_ins: 4,
    projects: 3,
    holdings: 4,
    goals: 4,
    initial_owners: 3,
    requests: 4,
    pending: 2,
    waiters: 2,
    identity_bytes: 16,
    words: 8,
    amendment_bytes: 256,
    sign_in_lifetime: Duration::from_secs(60),
    request_retention: Duration::from_secs(120),
    room_refusals: 2,
    facts: 8,
};

#[test]
fn application_parts_use_party_admission_and_preserve_the_original_key() {
    let mut test = Test::new(LIMITS);
    test.signin(1, 10, identity(0, 1));
    test.send(Event::Roles { project: 1, holdings: Box::new([Holding { person: 1, role: Role::Observer }]) });
    let ask = Ask::Application { project: 1, connector: 7, digest: [8; 32] };
    let request = route(&test.request(10, 1, ask.clone()));
    let observer_part = Ask::SetPool { project: 1, person: 1, budget: 20 };
    assert_eq!(
        application_part(&mut test.d, &test.env, request, observer_part.clone()),
        Ok(Request::Route { request, person: 1, project: 1, role: Some(Role::Observer), ask: Box::new(observer_part) })
    );
    application_part_done(&mut test.d, request);
    assert_eq!(
        application_part(&mut test.d, &test.env, request, Ask::SetPool { project: 2, person: 1, budget: 20 }),
        Err(Outcome::Refused(Refusal::Unknown))
    );
    assert_eq!(
        application_part(&mut test.d, &test.env, request, ask.clone()),
        Err(Outcome::Refused(Refusal::NotOffered))
    );
    test.send(Event::Roles { project: 1, holdings: Box::new([Holding { person: 1, role: Role::Owner }]) });
    let part = Ask::SetRoles { project: 1, holdings: Box::new([Holding { person: 1, role: Role::Owner }]) };
    assert_eq!(
        application_part(&mut test.d, &test.env, request, part.clone()),
        Ok(Request::Route { request, person: 1, project: 1, role: Some(Role::Owner), ask: Box::new(part) })
    );
    let to = test.to();
    let applied = test.send(Event::ApplyRoles { reply_to: to, request });
    assert!(applied.iter().any(|row| if let Request::RolesApplied { result, .. } = row {
        result.is_ok()
    } else {
        false
    }));
    application_part_done(&mut test.d, request);
    assert!(test.request(10, 1, ask.clone()).is_empty(), "same digest still joins the original flight");
    let outcome = Outcome::Applied { connector: 7, token: Token::new(40) };
    let completed = test.send(Event::Decided { request, outcome });
    assert_eq!(reply(&completed), Reply::Outcome(outcome));
    assert_eq!(
        saved_answer(&completed),
        Stored::Answer {
            key: RequestKey { person: 1, key: [1; 16] },
            ask: Box::new(ask.clone()),
            outcome,
            at: Wall::EPOCH
        }
    );
    assert_eq!(reply(&test.request(10, 1, ask)), Reply::Outcome(outcome));
}

#[test]
fn only_keyed_mark_seen_moves_results_and_restores_the_position() {
    let mut test = Test::new(LIMITS);
    test.signin(1, 10, identity(0, 1));
    test.send(Event::Roles { project: 1, holdings: Box::new([Holding { person: 1, role: Role::Member }]) });
    for (task, position) in [(4, 4), (3, 3), (1, 1), (2, 2)] {
        test.d.remember_result(&LIMITS, 1, ResultRef { task, project: 1, at: Wall::EPOCH, position });
    }
    assert_eq!(test.d.cached_results(1).unwrap().len(), 3, "one overflow witness beyond the count limit");
    assert_eq!(test.d.counts(&LIMITS, 1), Some(Counts { needs: 0, unread: 2, more_unread: true }));
    assert_eq!(test.d.results_seen(1), Some(0));
    let seen = Seen::Results { through: 3 };
    let request = route(&test.request(10, 1, Ask::MarkSeen { seen }));
    let outputs = test.send(Event::Decided { request, outcome: Outcome::MarkedSeen { seen } });
    assert_eq!(test.d.results_seen(1), Some(3));
    assert_eq!(test.d.cached_results(1), Some(&[ResultRef { task: 4, project: 1, at: Wall::EPOCH, position: 4 }][..]));
    assert!(outputs.contains(&Request::Save { record: Stored::ReadResults { party: 1, through: 3 } }));
    assert_eq!(reply(&test.request(10, 1, Ask::MarkSeen { seen })), Reply::Outcome(Outcome::MarkedSeen { seen }));
    let backwards = Seen::Results { through: 1 };
    let request = route(&test.request(10, 2, Ask::MarkSeen { seen: backwards }));
    test.send(Event::Decided { request, outcome: Outcome::MarkedSeen { seen: backwards } });
    assert_eq!(test.d.results_seen(1), Some(3));
    let mut restored = Test::new(LIMITS);
    restored.d = Domain::new(&LIMITS, Box::new([]), 2);
    restored.send(Event::Restore { record: Stored::ReadResults { party: 1, through: 3 } });
    restored.send(Event::Restore { record: Stored::Person { number: 1, identity: identity(0, 1) } });
    restored.send(Event::Restored);
    assert_eq!(restored.d.results_seen(1), Some(3));
}

#[test]
fn waiting_cache_keeps_newest_references_and_removes_one_task() {
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: LIMITS };
    let mut out = Queue::with_capacity(max_out(&LIMITS));
    let mut domain = Domain::new(&LIMITS, Box::new([]), 2);
    step(
        &mut domain,
        &env,
        Event::Restore { record: Stored::Person { number: 1, identity: identity(0, 1) } },
        &mut out,
    );
    step(&mut domain, &env, Event::Restored, &mut out);
    for task in 1..=3 {
        let entry = Entry { task, project: 1, whom: Whom::Person(1), kind: EntryKind::PersonTask, at: Wall::EPOCH };
        step(&mut domain, &env, Event::Waiting { task, entries: Box::new([entry]) }, &mut out);
    }
    let cached = domain.cached_waiting(&LIMITS, 1).expect("restored person");
    assert_eq!(cached.iter().map(|entry| entry.task).collect::<Vec<_>>(), [3, 2]);
    step(&mut domain, &env, Event::Waiting { task: 3, entries: Box::new([]) }, &mut out);
    let cached = domain.cached_waiting(&LIMITS, 1).expect("restored person");
    assert_eq!(cached.iter().map(|entry| entry.task).collect::<Vec<_>>(), [2]);
}

fn identity(provider: u16, subject: u64) -> Identity {
    Identity {
        key: IdentityKey { provider, subject: subject.to_be_bytes().into() },
        login: Box::new([b'l']),
        name: Box::new([b'n']),
    }
}

#[test]
fn an_owner_routes_a_connector_resource_adoption_once_by_its_key() {
    let mut test = Test::new(LIMITS);
    test.signin(1, 10, identity(0, 1));
    test.send(Event::Roles { project: 1, holdings: Box::new([Holding { person: 1, role: Role::Owner }]) });
    let ask = Ask::Adopt {
        project: 1,
        adoption: Adoption {
            resource: ResourceName { connector: 7, path: Box::new([b"resource".as_slice().into()]) },
            role: ResourceRole::Context,
            options: Box::new([b"read".as_slice().into()]),
        },
    };
    let request = route(&test.request(10, 1, ask.clone()));
    assert_eq!(
        reply(&test.send(Event::Decided { request, outcome: Outcome::Adopted { project: 1 } })),
        Reply::Outcome(Outcome::Adopted { project: 1 })
    );
    assert_eq!(reply(&test.request(10, 1, ask)), Reply::Outcome(Outcome::Adopted { project: 1 }));
}

#[test]
fn owner_makes_a_service_which_can_request_work_but_cannot_start_a_chat() {
    let mut test = Test::new(LIMITS);
    test.signin(1, 10, identity(0, 1));
    test.send(Event::Roles { project: 1, holdings: Box::new([Holding { person: 1, role: Role::Owner }]) });

    let ask = Ask::MakeService { project: 1, name: b"builder".as_slice().into(), role: Role::Member };
    let request = route(&test.request(10, 1, ask.clone()));
    let made = test.send(Event::MakeService { request, person: 2 });
    let mut person_saved = false;
    let mut roles_saved = false;
    let mut service_made = false;
    for row in &made {
        if let Request::Save { record: Stored::Person { number: 2, .. } } = row {
            person_saved = true;
        }
        if let Request::Save { record: Stored::Roles { project: 1, .. } } = row {
            roles_saved = true;
        }
        if let Request::ServiceMade { outcome: Outcome::ServiceMade { person: 2 }, .. } = row {
            service_made = true;
        }
    }
    assert!(person_saved && roles_saved && service_made);
    let decided = test.send(Event::Decided { request, outcome: Outcome::ServiceMade { person: 2 } });
    assert_eq!(reply(&decided), Reply::Outcome(Outcome::ServiceMade { person: 2 }));
    assert_eq!(reply(&test.request(10, 1, ask)), Reply::Outcome(Outcome::ServiceMade { person: 2 }));

    let unknown_to = test.to();
    let unknown = test.send(Event::SignedIn {
        reply_to: unknown_to,
        person: 3,
        sign_in: 20,
        identity: identity(2, 3),
        kind: Kind::Service,
    });
    assert_eq!(reply(&unknown), Reply::SignedOut);
    let service_to = test.to();
    let service = test.send(Event::SignedIn {
        reply_to: service_to,
        person: 4,
        sign_in: 21,
        identity: identity(2, 2),
        kind: Kind::Service,
    });
    assert_eq!(reply(&service), Reply::SignedIn { person: 2, expires: Wall::from_nanos(60_000_000_000) });
    assert_eq!(
        reply(&test.request(21, 2, Ask::StartChat { project: 1, words: Box::new([]) })),
        Reply::Outcome(Outcome::Refused(Refusal::Role { held: Some(Role::Member) }))
    );
    let routed = test.request(
        21,
        3,
        Ask::SetGoal {
            project: 1,
            spec: b"work".as_slice().into(),
            charter: 1,
            budget: 1,
            priority: 1,
            deadline: None,
        },
    );
    let mut service_routed = false;
    for row in &routed {
        if let Request::Route { person, .. } = row {
            assert_eq!(*person, 2);
            service_routed = true;
        }
    }
    assert!(service_routed);
}

fn ask(project: u32) -> Ask {
    Ask::StartChat { project, words: Box::new([1]) }
}

struct Test {
    d: Domain,
    env: Env<Limits>,
    serial: u64,
}

impl Test {
    fn new(limits: Limits) -> Test {
        let mut test = Test {
            d: Domain::new(&limits, Box::new([]), 2),
            env: Env { limits, now: Time::ZERO, wall: Wall::EPOCH },
            serial: 0,
        };
        assert!(test.send(Event::Restored).is_empty());
        test
    }

    fn to(&mut self) -> ReplyTo {
        self.serial += 1;
        ReplyTo::new(Token::new(self.serial))
    }

    fn send(&mut self, event: Event) -> Vec<Request> {
        let mut out = Queue::with_capacity(max_out(&self.env.limits));
        step(&mut self.d, &self.env, event, &mut out);
        let mut requests = Vec::new();
        while let Some(request) = out.pop() {
            requests.push(request);
        }
        self.d.reclaim();
        requests
    }

    fn send_bounded(&mut self, event: Event) -> List<Request> {
        let capacity = max_out(&self.env.limits);
        let mut out = Queue::with_capacity(capacity);
        step(&mut self.d, &self.env, event, &mut out);
        let mut requests = List::with_capacity(capacity);
        for _ in 0..capacity {
            let Some(request) = out.pop() else {
                break;
            };
            requests.push(request).expect("one bounded output snapshot");
        }
        assert!(out.is_empty());
        self.d.reclaim();
        requests
    }

    fn signin_bounded(&mut self, person: u64, sign_in: u64, identity: Identity) -> List<Request> {
        let reply_to = self.to();
        self.send_bounded(Event::SignedIn { reply_to, person, sign_in, identity, kind: Kind::Person })
    }

    fn request_bounded(&mut self, sign_in: u64, key: u8, ask: Ask) -> List<Request> {
        let reply_to = self.to();
        self.send_bounded(Event::Ask { reply_to, sign_in, key: [key; 16], ask })
    }

    fn signin(&mut self, person: u64, sign_in: u64, identity: Identity) -> Vec<Request> {
        let reply_to = self.to();
        self.send(Event::SignedIn { reply_to, person, sign_in, identity, kind: Kind::Person })
    }

    fn request(&mut self, sign_in: u64, key: u8, ask: Ask) -> Vec<Request> {
        let reply_to = self.to();
        self.send(Event::Ask { reply_to, sign_in, key: [key; 16], ask })
    }

    fn member(&mut self) {
        self.signin(1, 10, identity(0, 1));
        self.send(Event::Roles { project: 1, holdings: Box::new([Holding { person: 1, role: Role::Member }]) });
    }
}

fn reply(rows: &[Request]) -> Reply {
    rows.iter()
        .find_map(|r| match r {
            Request::Reply { reply, .. } => Some(*reply),
            Request::Save { .. }
            | Request::Erase { .. }
            | Request::Route { .. }
            | Request::RolesApplied { .. }
            | Request::RolesRefused { .. }
            | Request::ServiceMade { .. }
            | Request::RestoreRefused { .. } => None,
        })
        .expect("reply emitted")
}

fn route(rows: &[Request]) -> Token {
    rows.iter()
        .find_map(|r| match r {
            Request::Route { request, .. } => Some(*request),
            Request::Save { .. }
            | Request::Erase { .. }
            | Request::Reply { .. }
            | Request::RolesApplied { .. }
            | Request::RolesRefused { .. }
            | Request::ServiceMade { .. }
            | Request::RestoreRefused { .. } => None,
        })
        .expect("route emitted")
}

fn saved_answer(rows: &[Request]) -> Stored {
    rows.iter()
        .find_map(|row| match row {
            Request::Save { record } => match record {
                Stored::Answer { .. } => Some(record.clone()),
                Stored::Person { .. }
                | Stored::ReadResults { .. }
                | Stored::ReadChat { .. }
                | Stored::SignIn { .. }
                | Stored::Roles { .. }
                | Stored::Policy { .. } => None,
            },
            Request::Reply { .. }
            | Request::Erase { .. }
            | Request::Route { .. }
            | Request::RolesApplied { .. }
            | Request::RolesRefused { .. }
            | Request::ServiceMade { .. }
            | Request::RestoreRefused { .. } => None,
        })
        .expect("keyed answer saved")
}

#[test]
fn provider_and_subject_identify_a_party_independent_of_display_fields() {
    let mut test = Test::new(LIMITS);
    assert_eq!(signed_person(reply(&test.signin(1, 10, identity(0, 1)))), Some(1));
    let mut renamed = identity(0, 1);
    renamed.login = b"new".as_slice().into();
    assert_eq!(signed_person(reply(&test.signin(2, 11, renamed))), Some(1));
    assert_eq!(signed_person(reply(&test.signin(3, 12, identity(1, 1)))), Some(3));
    assert_eq!(reply(&test.signin(4, 10, identity(0, 4))), Reply::SignedOut);
    let directory = Identity {
        key: IdentityKey { provider: 1, subject: b"alice".as_slice().into() },
        login: b"a".as_slice().into(),
        name: b"A".as_slice().into(),
    };
    assert_eq!(signed_person(reply(&test.signin(4, 13, directory))), Some(4));
}

#[test]
fn all_roles_route_to_the_core_and_its_refusals_are_saved() {
    for role in [Role::Owner, Role::Maintainer, Role::Member, Role::Observer, Role::Policy { role: 4 }] {
        let mut test = Test::new(LIMITS);
        test.member();
        test.send(Event::Roles { project: 1, holdings: Box::new([Holding { person: 1, role }]) });
        let rows = test.request(10, 1, ask(1));
        let request = route(&rows);
        assert!(rows.iter().any(|row| if let Request::Route { role: Some(held), .. } = row {
            *held == role
        } else {
            false
        }));
        if role == Role::Observer {
            let refused =
                test.send(Event::Decided { request, outcome: Outcome::Refused(Refusal::Role { held: Some(role) }) });
            assert_eq!(reply(&refused), Reply::Outcome(Outcome::Refused(Refusal::Role { held: Some(role) })));
            assert!(refused.iter().any(is_answer));
            assert_eq!(
                reply(&test.request(10, 1, ask(1))),
                Reply::Outcome(Outcome::Refused(Refusal::Role { held: Some(role) }))
            );
        }
    }
    let mut test = Test::new(LIMITS);
    test.member();
    let request = route(&test.request(10, 1, ask(2)));
    let out = test.send(Event::Decided { request, outcome: Outcome::Refused(Refusal::Role { held: None }) });
    assert_eq!(reply(&out), Reply::Outcome(Outcome::Refused(Refusal::Role { held: None })));
}

#[test]
fn policy_role_receives_shared_waiting_and_replays_its_key_after_restart() {
    let mut live = Test::new(LIMITS);
    live.signin(1, 10, identity(0, 1));
    live.signin(2, 20, identity(0, 2));
    let holdings = Box::new([
        Holding { person: 1, role: Role::Policy { role: 4 } },
        Holding { person: 2, role: Role::Policy { role: 4 } },
    ]);
    live.send(Event::Roles { project: 1, holdings: holdings.clone() });
    let entry = Entry {
        task: 7,
        project: 1,
        whom: Whom::Role { project: 1, role: 4 },
        kind: EntryKind::PersonTask,
        at: Wall::EPOCH,
    };
    live.send(Event::Waiting { task: 7, entries: Box::new([entry]) });
    assert_eq!(live.d.cached_waiting(&LIMITS, 1).as_deref(), Some(&[entry][..]));
    assert_eq!(live.d.cached_waiting(&LIMITS, 2).as_deref(), Some(&[entry][..]));
    let ask = Ask::TakePerson { project: 1, task: 7 };
    let request = route(&live.request(10, 1, ask.clone()));
    let answered = live.send(Event::Decided { request, outcome: Outcome::PersonTaken { task: 7 } });
    let saved = saved_answer(&answered);
    live.send(Event::Waiting { task: 7, entries: Box::new([]) });
    assert_eq!(live.d.cached_waiting(&LIMITS, 1).as_deref(), Some(&[][..]));
    assert_eq!(live.d.cached_waiting(&LIMITS, 2).as_deref(), Some(&[][..]));

    let mut cold = Test::new(LIMITS);
    cold.d = Domain::new(&LIMITS, Box::new([]), 2);
    cold.send(Event::Restore { record: Stored::Person { number: 1, identity: identity(0, 1) } });
    cold.send(Event::Restore { record: Stored::Person { number: 2, identity: identity(0, 2) } });
    cold.send(Event::Restore { record: Stored::Roles { project: 1, holdings } });
    cold.send(Event::Restore {
        record: Stored::SignIn { number: 10, person: 1, expires: Wall::from_nanos(Duration::from_secs(60).as_nanos()) },
    });
    cold.send(Event::Restore { record: saved });
    cold.send(Event::Restored);
    assert_eq!(reply(&cold.request(10, 1, ask)), Reply::Outcome(Outcome::PersonTaken { task: 7 }));
}

#[test]
fn a_note_edit_routes_its_membership_and_retains_the_core_refusal() {
    let ask = Ask::EditNote {
        project: 1,
        name: 9,
        scope: Box::new(NoteScope::Project),
        change: Box::new(NoteChange::Delete { recalled: 1 }),
    };
    let mut member = Test::new(LIMITS);
    member.member();
    let request = route(&member.request(10, 14, ask.clone()));
    let refused = member.send(Event::Decided { request, outcome: Outcome::Refused(Refusal::NotOffered) });
    assert!(refused.iter().any(is_answer));
    assert_eq!(reply(&member.request(10, 14, ask.clone())), Reply::Outcome(Outcome::Refused(Refusal::NotOffered)));

    let mut observer = Test::new(LIMITS);
    observer.member();
    observer.send(Event::Roles { project: 1, holdings: Box::new([Holding { person: 1, role: Role::Observer }]) });
    let request = route(&observer.request(10, 14, ask.clone()));
    let refused = observer
        .send(Event::Decided { request, outcome: Outcome::Refused(Refusal::Role { held: Some(Role::Observer) }) });
    assert_eq!(reply(&refused), Reply::Outcome(Outcome::Refused(Refusal::Role { held: Some(Role::Observer) })));
    assert!(refused.iter().any(is_answer));
    assert_eq!(
        reply(&observer.request(10, 14, ask)),
        Reply::Outcome(Outcome::Refused(Refusal::Role { held: Some(Role::Observer) }))
    );
}

#[test]
fn only_held_policy_roles_consume_inbox_cache_room() {
    let mut test = Test::new(Limits { holdings: 1, ..LIMITS });
    test.member();
    test.send(Event::Roles {
        project: 1,
        holdings: Box::new([Holding { person: 1, role: Role::Policy { role: 99 } }]),
    });
    let absent = Entry {
        task: 7,
        project: 1,
        whom: Whom::Role { project: 1, role: 100 },
        kind: EntryKind::PersonTask,
        at: Wall::EPOCH,
    };
    test.send(Event::Waiting { task: 7, entries: Box::new([absent]) });
    let held = Entry { task: 8, whom: Whom::Role { project: 1, role: 99 }, ..absent };
    test.send(Event::Waiting { task: 8, entries: Box::new([held]) });
    assert_eq!(test.d.cached_waiting(&test.env.limits, 1).as_deref(), Some(&[held][..]));
    test.send(Event::Roles { project: 1, holdings: Box::new([]) });
    assert_eq!(test.d.cached_waiting(&test.env.limits, 1).as_deref(), Some(&[][..]));
}

#[test]
fn pending_duplicates_share_one_route_and_reserve_answer_room() {
    let mut test = Test::new(Limits { requests: 1, ..LIMITS });
    test.member();
    let routed = test.request(10, 1, ask(1));
    let token = route(&routed);
    assert!(test.request(10, 1, ask(1)).is_empty());
    assert_eq!(reply(&test.request(10, 1, ask(1))), Reply::Again(Again::Busy));
    assert_eq!(reply(&test.request(10, 2, ask(1))), Reply::Again(Again::Busy));
    assert_eq!(reply(&test.request(10, 1, ask(2))), Reply::Refused(Refusal::KeyConflict));
    let completed = test.send(Event::Decided { request: token, outcome: Outcome::Started { task: 90 } });
    assert_eq!(completed.iter().filter(|r| is_reply(r)).count(), 2);
    assert_eq!(reply(&test.request(10, 1, ask(1))), Reply::Outcome(Outcome::Started { task: 90 }));
}

#[test]
fn answered_keys_follow_people_across_signins_and_role_changes() {
    let mut test = Test::new(LIMITS);
    test.member();
    let rows = test.request(10, 1, ask(1));
    let token = route(&rows);
    test.send(Event::Decided { request: token, outcome: Outcome::Started { task: 90 } });
    let reply_to = test.to();
    test.send(Event::SignOut { reply_to, sign_in: 10 });
    test.signin(2, 11, identity(0, 1));
    test.send(Event::Roles { project: 1, holdings: Box::new([Holding { person: 1, role: Role::Observer }]) });
    assert_eq!(reply(&test.request(11, 1, ask(1))), Reply::Outcome(Outcome::Started { task: 90 }));
    assert_eq!(reply(&test.request(11, 1, ask(2))), Reply::Refused(Refusal::KeyConflict));
}

#[test]
fn completed_key_replays_until_its_deadline_then_can_be_reused() {
    let mut test = Test::new(LIMITS);
    test.member();
    let token = route(&test.request(10, 1, ask(1)));
    test.send(Event::Decided { request: token, outcome: Outcome::Started { task: 90 } });
    assert_eq!(reply(&test.request(10, 1, ask(1))), Reply::Outcome(Outcome::Started { task: 90 }));
    test.env.now = Time::from_nanos(Duration::from_secs(120).as_nanos());
    test.env.wall = Wall::from_nanos(Duration::from_secs(120).as_nanos());
    // The old sign-in has expired too; a new sign-in still belongs to the same person.
    test.signin(2, 11, identity(0, 1));
    let fresh = test.request(11, 1, ask(1));
    assert!(fresh.contains(&Request::Erase { key: Key::Answer(RequestKey { person: 1, key: [1; 16] }) }));
    let token = route(&fresh);
    assert_ne!(token, Token::new(0));
}

#[test]
fn expired_completed_key_is_erased_from_a_restore_page_and_frees_its_slot() {
    let limits = Limits { requests: 1, ..LIMITS };
    let mut test = Test::new(limits);
    test.d = Domain::new(&limits, Box::new([]), 2);
    test.env.now = Time::from_nanos(Duration::from_secs(120).as_nanos());
    test.env.wall = Wall::from_nanos(Duration::from_secs(120).as_nanos());
    let key = RequestKey { person: 1, key: [1; 16] };
    let erased = test.send(Event::Restore {
        record: Stored::Answer { key, ask: Box::new(ask(1)), outcome: Outcome::Started { task: 90 }, at: Wall::EPOCH },
    });
    assert_eq!(erased, [Request::Erase { key: Key::Answer(key) }]);
    test.send(Event::Restore { record: Stored::Person { number: 1, identity: identity(0, 1) } });
    test.send(Event::Restore {
        record: Stored::Roles { project: 1, holdings: Box::new([Holding { person: 1, role: Role::Member }]) },
    });
    test.send(Event::Restore {
        record: Stored::SignIn {
            number: 10,
            person: 1,
            expires: Wall::from_nanos(Duration::from_secs(180).as_nanos()),
        },
    });
    test.send(Event::Restored);
    assert!(test.d.ready());
    let _routed = route(&test.request(10, 1, ask(1)));
}

#[test]
fn every_admission_point_refuses_without_partial_state() {
    let mut test = Test::new(Limits { people: 1, sign_ins: 1, projects: 1, holdings: 1, words: 1, ..LIMITS });
    test.member();
    assert_eq!(reply(&test.signin(2, 11, identity(0, 2))), Reply::Again(Again::Busy));
    let reply_to = test.to();
    test.send(Event::SignOut { reply_to, sign_in: 10 });
    assert_eq!(reply(&test.signin(2, 11, identity(0, 2))), Reply::Again(Again::Busy));
    assert_eq!(signed_person(reply(&test.signin(3, 11, identity(0, 1)))), Some(1));
    let big = Ask::StartChat { project: 1, words: Box::new([1, 2]) };
    assert_eq!(reply(&test.request(11, 1, big)), Reply::Refused(Refusal::Limit { which: LimitKind::Words }));
    let roles = test.send(Event::Roles { project: 2, holdings: Box::new([]) });
    assert_eq!(roles, [Request::RolesRefused { project: 2, outcome: Outcome::Again(Again::Busy) }]);
    let roles = test.send(Event::Roles {
        project: 1,
        holdings: Box::new([Holding { person: 1, role: Role::Owner }, Holding { person: 2, role: Role::Member }]),
    });
    assert_eq!(
        roles,
        [Request::RolesRefused { project: 1, outcome: Outcome::Refused(Refusal::Limit { which: LimitKind::Roles }) }]
    );
}

#[test]
fn expiry_is_monotonic_even_when_wall_clock_moves_back() {
    let mut test = Test::new(LIMITS);
    test.member();
    test.env.now = Time::from_nanos(Duration::from_secs(60).as_nanos());
    test.env.wall = Wall::EPOCH;
    assert_eq!(reply(&test.request(10, 1, ask(1))), Reply::SignedOut);
    let mut out = Queue::with_capacity(max_out(&LIMITS));
    fire(&mut test.d, &test.env, &mut out);
    assert_eq!(out.pop(), Some(Request::Erase { key: Key::SignIn(10) }));
}

#[test]
fn result_query_checks_monotonic_expiry_before_fire_after_backward_wall_jump() {
    let mut test = Test::new(LIMITS);
    test.env.wall = Wall::from_nanos(Duration::from_secs(100).as_nanos());
    test.member();
    assert_eq!(test.d.person(10, test.env.now, test.env.wall), Some(1));
    test.env.now = Time::from_nanos(Duration::from_secs(60).as_nanos());
    test.env.wall = Wall::EPOCH;
    assert_eq!(test.d.person(10, test.env.now, test.env.wall), None);
    assert_eq!(reply(&test.request(10, 1, ask(1))), Reply::SignedOut);
}

#[test]
fn restore_order_is_independent_and_bad_or_oversized_records_refuse_start() {
    let mut test = Test::new(LIMITS);
    test.d = Domain::new(&LIMITS, Box::new([]), 2);
    test.send(Event::Restore {
        record: Stored::SignIn { number: 10, person: 1, expires: Wall::from_nanos(Duration::from_secs(60).as_nanos()) },
    });
    test.send(Event::Restore { record: Stored::Person { number: 1, identity: identity(0, 1) } });
    test.send(Event::Restored);
    assert!(test.d.ready());
    test.d = Domain::new(&LIMITS, Box::new([]), 2);
    test.send(Event::Restore { record: Stored::SignIn { number: 10, person: 99, expires: Wall::EPOCH } });
    assert_eq!(
        test.send(Event::Restored),
        [Request::RestoreRefused { key: Key::SignIn(10), refusal: Refusal::Unknown }]
    );
    assert!(!test.d.ready());
    test.env.limits.people = 0;
    test.d = Domain::new(&test.env.limits, Box::new([]), 2);
    assert_eq!(
        test.send(Event::Restore { record: Stored::Person { number: 1, identity: identity(0, 1) } }),
        [Request::RestoreRefused { key: Key::Person(1), refusal: Refusal::Limit { which: LimitKind::Restore } }]
    );
}

#[test]
fn role_success_restores_before_identities_and_replays_after_roster_changes() {
    let mut live = Test::new(LIMITS);
    live.signin_bounded(1, 10, identity(0, 1));
    live.signin_bounded(2, 20, identity(0, 2));
    live.send_bounded(Event::Roles { project: 1, holdings: Box::new([Holding { person: 1, role: Role::Owner }]) });
    let ask = Ask::SetRoles { project: 1, holdings: Box::new([Holding { person: 2, role: Role::Member }]) };
    let request = route(live.request_bounded(10, 7, ask.clone()).as_slice());
    let reply_token = live.to().into_token();
    assert_eq!(
        live.send_bounded(Event::ApplyRoles { reply_to: ReplyTo::new(reply_token), request }).as_slice(),
        [
            Request::Save {
                record: Stored::Roles { project: 1, holdings: Box::new([Holding { person: 2, role: Role::Member }]) }
            },
            Request::RolesApplied { reply_to: ReplyTo::new(reply_token), request, result: Ok(()) },
        ]
    );
    let outcome = Outcome::RolesSet { project: 1 };
    let completed = live.send_bounded(Event::Decided { request, outcome });
    assert_eq!(reply(completed.as_slice()), Reply::Outcome(outcome));
    let mut cold = Test::new(LIMITS);
    cold.d = Domain::new(&LIMITS, Box::new([]), 2);
    assert!(cold.send_bounded(Event::Restore { record: saved_answer(completed.as_slice()) }).is_empty());
    // The historical roster differs from today's; the original owner is now an observer.
    assert!(
        cold.send_bounded(Event::Restore {
            record: Stored::Roles { project: 1, holdings: Box::new([Holding { person: 1, role: Role::Observer }]) }
        })
        .is_empty()
    );
    for person in [2, 1] {
        assert!(
            cold.send_bounded(Event::Restore {
                record: Stored::Person { number: person, identity: identity(0, person) }
            })
            .is_empty()
        );
    }
    assert!(cold.send_bounded(Event::Restored).is_empty());
    assert!(cold.d.ready());
    cold.signin_bounded(3, 30, identity(0, 1));
    let replayed = cold.request_bounded(30, 7, ask.clone());
    assert_eq!(replayed.len(), 1, "replay emits no route or role write");
    assert_eq!(reply(replayed.as_slice()), Reply::Outcome(outcome));
    assert_eq!(cold.d.role(1, 1), Some(Role::Observer));
    assert_eq!(cold.d.role(2, 1), None);
    let request = route(cold.request_bounded(30, 8, ask).as_slice());
    assert_eq!(
        reply(
            cold.send_bounded(Event::Decided {
                request,
                outcome: Outcome::Refused(Refusal::Role { held: Some(Role::Observer) }),
            })
            .as_slice()
        ),
        Reply::Outcome(Outcome::Refused(Refusal::Role { held: Some(Role::Observer) }))
    );
}

#[test]
fn restored_role_success_requires_matching_ask_kind_and_project() {
    let key = RequestKey { person: 1, key: [7; 16] };
    for (ask, outcome) in [
        (Ask::SetRoles { project: 1, holdings: Box::new([]) }, Outcome::RolesSet { project: 2 }),
        (ask(1), Outcome::RolesSet { project: 1 }),
        (
            Ask::DecideEscalation { project: 1, task: 9, revision: 2, decision: EscalationDecision::Pass },
            Outcome::RolesSet { project: 1 },
        ),
        (Ask::SetRoles { project: 1, holdings: Box::new([]) }, Outcome::Started { task: 9 }),
        (
            Ask::SetRoles { project: 1, holdings: Box::new([]) },
            Outcome::EscalationDecided {
                task: 9,
                revision: 2,
                by: 1,
                at: Wall::from_nanos(0),
                choice: EscalationChoice::Passed,
            },
        ),
    ] {
        let mut test = Test::new(LIMITS);
        test.d = Domain::new(&LIMITS, Box::new([]), 2);
        assert_eq!(
            test.send_bounded(Event::Restore {
                record: Stored::Answer { key, ask: Box::new(ask), outcome, at: Wall::EPOCH }
            })
            .as_slice(),
            [Request::RestoreRefused { key: Key::Answer(key), refusal: Refusal::Limit { which: LimitKind::Restore } }]
        );
        assert!(test.send_bounded(Event::Restored).is_empty());
        assert!(!test.d.ready());
    }
}

#[test]
fn restored_successful_role_rosters_require_positive_unique_people() {
    let key = RequestKey { person: 1, key: [7; 16] };
    for holdings in [
        Box::from([Holding { person: 0, role: Role::Member }]),
        Box::from([Holding { person: 2, role: Role::Owner }, Holding { person: 2, role: Role::Member }]),
    ] {
        let mut test = Test::new(LIMITS);
        test.d = Domain::new(&LIMITS, Box::new([]), 2);
        assert_eq!(
            test.send_bounded(Event::Restore {
                record: Stored::Answer {
                    key,
                    ask: Box::new(Ask::SetRoles { project: 1, holdings }),
                    outcome: Outcome::RolesSet { project: 1 },
                    at: Wall::EPOCH,
                }
            })
            .as_slice(),
            [Request::RestoreRefused { key: Key::Answer(key), refusal: Refusal::Limit { which: LimitKind::Restore } }]
        );
        assert!(test.send_bounded(Event::Restored).is_empty());
        assert!(!test.d.ready());
    }
}

#[test]
fn restored_role_success_checks_historical_people_and_project_after_all_rows_arrive() {
    let key = RequestKey { person: 1, key: [7; 16] };
    for (project, person) in [(1, 99), (2, 1)] {
        let mut test = Test::new(LIMITS);
        test.d = Domain::new(&LIMITS, Box::new([]), 2);
        assert!(
            test.send_bounded(Event::Restore {
                record: Stored::Answer {
                    key,
                    ask: Box::new(Ask::SetRoles {
                        project,
                        holdings: Box::new([Holding { person, role: Role::Member }])
                    }),
                    outcome: Outcome::RolesSet { project },
                    at: Wall::EPOCH,
                }
            })
            .is_empty()
        );
        test.send_bounded(Event::Restore { record: Stored::Person { number: 1, identity: identity(0, 1) } });
        test.send_bounded(Event::Restore { record: Stored::Roles { project: 1, holdings: Box::new([]) } });
        assert_eq!(
            test.send_bounded(Event::Restored).as_slice(),
            [Request::RestoreRefused { key: Key::Answer(key), refusal: Refusal::Unknown }]
        );
        assert!(!test.d.ready());
    }
}

#[test]
fn refused_role_requests_restore_invalid_rosters_and_unknown_targets_for_replay() {
    for (role, project, holdings, refusal) in [
        (
            Role::Observer,
            1,
            Box::from([Holding { person: 0, role: Role::Owner }, Holding { person: 0, role: Role::Member }]),
            Refusal::Role { held: Some(Role::Observer) },
        ),
        (Role::Owner, 2, Box::from([Holding { person: 99, role: Role::Member }]), Refusal::Unknown),
        (
            Role::Owner,
            1,
            Box::from([Holding { person: 0, role: Role::Member }]),
            Refusal::Limit { which: LimitKind::Roles },
        ),
        (
            Role::Owner,
            1,
            Box::from([Holding { person: 1, role: Role::Owner }, Holding { person: 1, role: Role::Member }]),
            Refusal::Limit { which: LimitKind::Roles },
        ),
        (Role::Owner, 1, Box::from([Holding { person: 99, role: Role::Member }]), Refusal::Unknown),
    ] {
        let mut live = Test::new(LIMITS);
        live.signin_bounded(1, 10, identity(0, 1));
        live.send_bounded(Event::Roles { project: 1, holdings: Box::new([Holding { person: 1, role }]) });
        let ask = Ask::SetRoles { project, holdings };
        let mut rows = live.request_bounded(10, 7, ask.clone());
        let request = route(rows.as_slice());
        if role == Role::Owner && project == 1 {
            let reply_token = live.to().into_token();
            assert_eq!(
                live.send_bounded(Event::ApplyRoles { reply_to: ReplyTo::new(reply_token), request }).as_slice(),
                [Request::RolesApplied {
                    reply_to: ReplyTo::new(reply_token),
                    request,
                    result: Err(Outcome::Refused(refusal))
                }]
            );
        }
        rows = live.send_bounded(Event::Decided { request, outcome: Outcome::Refused(refusal) });
        assert_eq!(reply(rows.as_slice()), Reply::Outcome(Outcome::Refused(refusal)));
        let mut cold = Test::new(LIMITS);
        cold.d = Domain::new(&LIMITS, Box::new([]), 2);
        assert!(cold.send_bounded(Event::Restore { record: saved_answer(rows.as_slice()) }).is_empty());
        cold.send_bounded(Event::Restore { record: Stored::Person { number: 1, identity: identity(0, 1) } });
        // Even a refused unknown project needs no present membership row.
        assert!(cold.send_bounded(Event::Restored).is_empty());
        assert!(cold.d.ready());
        cold.signin_bounded(2, 20, identity(0, 1));
        let replayed = cold.request_bounded(20, 7, ask);
        assert_eq!(replayed.len(), 1);
        assert_eq!(reply(replayed.as_slice()), Reply::Outcome(Outcome::Refused(refusal)));
    }
}

#[test]
fn initial_owners_bootstrap_all_projects_atomically_and_never_regrant() {
    let owners = Box::new([
        InitialOwner { project: 1, identity: IdentityKey { provider: 0, subject: 1_u64.to_be_bytes().into() } },
        InitialOwner { project: 2, identity: IdentityKey { provider: 0, subject: 1_u64.to_be_bytes().into() } },
    ]);
    let mut test = Test::new(Limits { holdings: 1, ..LIMITS });
    test.d = Domain::new(&test.env.limits, owners, 2);
    test.send(Event::Roles { project: 1, holdings: Box::new([]) });
    test.send(Event::Roles { project: 2, holdings: Box::new([Holding { person: 9, role: Role::Member }]) });
    test.send(Event::Restore { record: Stored::Person { number: 9, identity: identity(0, 9) } });
    test.send(Event::Restored);
    let refused = test.signin(1, 10, identity(0, 1));
    assert_eq!(reply(&refused), Reply::Again(Again::Busy));
    assert_eq!(refused.len(), 1, "bootstrap refusal makes no record");
    test.send(Event::Roles { project: 2, holdings: Box::new([]) });
    let made = test.signin(2, 10, identity(0, 1));
    assert_eq!(made.iter().filter(|r| is_roles(r)).count(), 2);
    test.send(Event::Roles { project: 1, holdings: Box::new([Holding { person: 2, role: Role::Observer }]) });
    let again = test.signin(3, 11, identity(0, 1));
    assert!(!again.iter().any(is_roles));
    let request = route(&test.request(11, 1, ask(1)));
    assert_eq!(
        reply(
            &test.send(Event::Decided {
                request,
                outcome: Outcome::Refused(Refusal::Role { held: Some(Role::Observer) }),
            })
        ),
        Reply::Outcome(Outcome::Refused(Refusal::Role { held: Some(Role::Observer) }))
    );
}

fn signed_person(reply: Reply) -> Option<u64> {
    match reply {
        Reply::SignedIn { person, .. } => Some(person),
        Reply::SignedOut | Reply::Again(_) | Reply::Outcome(_) | Reply::Refused(_) => None,
    }
}

fn is_reply(request: &Request) -> bool {
    match request {
        Request::Reply { .. } => true,
        Request::Save { .. }
        | Request::Erase { .. }
        | Request::Route { .. }
        | Request::RolesApplied { .. }
        | Request::RolesRefused { .. }
        | Request::ServiceMade { .. }
        | Request::RestoreRefused { .. } => false,
    }
}

fn is_answer(request: &Request) -> bool {
    match request {
        Request::Save { record } => match record {
            Stored::Answer { .. } => true,
            Stored::Person { .. }
            | Stored::ReadResults { .. }
            | Stored::ReadChat { .. }
            | Stored::SignIn { .. }
            | Stored::Roles { .. }
            | Stored::Policy { .. } => false,
        },
        Request::Reply { .. }
        | Request::Erase { .. }
        | Request::Route { .. }
        | Request::RolesApplied { .. }
        | Request::RolesRefused { .. }
        | Request::ServiceMade { .. }
        | Request::RestoreRefused { .. } => false,
    }
}

fn is_roles(request: &Request) -> bool {
    match request {
        Request::Save { record } => match record {
            Stored::Roles { .. } => true,
            Stored::Person { .. }
            | Stored::ReadResults { .. }
            | Stored::ReadChat { .. }
            | Stored::SignIn { .. }
            | Stored::Answer { .. }
            | Stored::Policy { .. } => false,
        },
        Request::Reply { .. }
        | Request::Erase { .. }
        | Request::Route { .. }
        | Request::RolesApplied { .. }
        | Request::RolesRefused { .. }
        | Request::ServiceMade { .. }
        | Request::RestoreRefused { .. } => false,
    }
}

#[test]
fn equal_keys_of_different_people_are_independent() {
    let mut test = Test::new(LIMITS);
    test.member();
    test.signin(2, 20, identity(0, 2));
    test.send(Event::Roles {
        project: 1,
        holdings: Box::new([Holding { person: 1, role: Role::Member }, Holding { person: 2, role: Role::Member }]),
    });
    let first = route(&test.request(10, 1, ask(1)));
    let second = route(&test.request(20, 1, ask(1)));
    assert_ne!(first, second);
    test.send(Event::Decided { request: first, outcome: Outcome::Started { task: 90 } });
    test.send(Event::Decided { request: second, outcome: Outcome::Started { task: 91 } });
    assert_eq!(reply(&test.request(10, 1, ask(1))), Reply::Outcome(Outcome::Started { task: 90 }));
    assert_eq!(reply(&test.request(20, 1, ask(1))), Reply::Outcome(Outcome::Started { task: 91 }));
}

#[test]
fn pending_capacity_refuses_before_routing_and_the_key_can_retry() {
    let mut test = Test::new(Limits { pending: 1, ..LIMITS });
    test.member();
    let first = route(&test.request(10, 1, ask(1)));
    assert_eq!(reply(&test.request(10, 2, ask(1))), Reply::Again(Again::Busy));
    assert_eq!(test.d.capacity_refusals().get(0).expect("pending pressure").cause, CapacityCause::Pending { limit: 1 });
    test.send(Event::Decided {
        request: first,
        outcome: Outcome::Refused(Refusal::Authority { proposable: None, lacking: Lack::Shape }),
    });
    route(&test.request(10, 2, ask(1)));
}

#[test]
fn root_pressure_answers_every_waiter_and_releases_the_key_for_retry() {
    for again in [Again::Busy, Again::NotReady] {
        let mut test = Test::new(Limits { requests: 1, ..LIMITS });
        test.member();
        let first = route(&test.request(10, 1, ask(1)));
        assert!(test.request(10, 1, ask(1)).is_empty());
        let rows = test.send(Event::Decided { request: first, outcome: Outcome::Again(again) });
        assert_eq!(rows.len(), 2);
        assert!(rows.iter().all(is_reply), "retryable pressure saves no completed answer");
        assert_eq!(reply(&rows), Reply::Again(again));
        let retry = route(&test.request(10, 1, ask(1)));
        assert_ne!(first, retry, "retired route tokens fence earlier completions");
        test.send(Event::Decided { request: retry, outcome: Outcome::Started { task: 90 } });
        assert_eq!(reply(&test.request(10, 1, ask(1))), Reply::Outcome(Outcome::Started { task: 90 }));
    }
}

#[test]
fn identity_limits_clock_overflow_and_duplicate_holdings_leave_no_records() {
    let mut test = Test::new(Limits { identity_bytes: 1, ..LIMITS });
    assert_eq!(
        reply(&test.signin(1, 10, identity(0, 1))),
        Reply::Refused(Refusal::Limit { which: LimitKind::Identity })
    );
    test.env.limits = LIMITS;
    test.d = Domain::new(&LIMITS, Box::new([]), 2);
    test.send(Event::Restored);
    test.env.wall = Wall::from_nanos(u64::MAX);
    assert_eq!(
        reply(&test.signin(1, 10, identity(0, 1))),
        Reply::Refused(Refusal::Limit { which: LimitKind::Arithmetic })
    );
    test.env.wall = Wall::EPOCH;
    test.member();
    let rows = test.send(Event::Roles {
        project: 1,
        holdings: Box::new([Holding { person: 1, role: Role::Owner }, Holding { person: 1, role: Role::Member }]),
    });
    assert_eq!(
        rows,
        [Request::RolesRefused { project: 1, outcome: Outcome::Refused(Refusal::Limit { which: LimitKind::Roles }) }]
    );
    route(&test.request(10, 1, ask(1)));
}

#[test]
fn calls_before_restore_and_after_failed_restore_are_answered_not_ready() {
    let mut test = Test::new(LIMITS);
    test.d = Domain::new(&LIMITS, Box::new([]), 2);
    assert_eq!(reply(&test.signin(1, 10, identity(0, 1))), Reply::Again(Again::NotReady));
    assert_eq!(reply(&test.request(10, 1, ask(1))), Reply::Again(Again::NotReady));
    test.send(Event::Restore {
        record: Stored::Answer {
            key: RequestKey { person: 99, key: [1; 16] },
            ask: Box::new(ask(1)),
            outcome: Outcome::Started { task: 90 },
            at: Wall::EPOCH,
        },
    });
    test.send(Event::Restored);
    assert!(!test.d.ready());
    assert_eq!(reply(&test.signin(1, 10, identity(0, 1))), Reply::Again(Again::NotReady));
}

#[test]
fn expiry_projection_uses_the_environment_when_restore_finishes() {
    let mut test = Test::new(LIMITS);
    test.d = Domain::new(&LIMITS, Box::new([]), 2);
    test.send(Event::Restore { record: Stored::Person { number: 1, identity: identity(0, 1) } });
    test.send(Event::Restore {
        record: Stored::SignIn { number: 10, person: 1, expires: Wall::from_nanos(Duration::from_secs(60).as_nanos()) },
    });
    test.env.now = Time::from_nanos(Duration::from_secs(30).as_nanos());
    test.env.wall = Wall::from_nanos(Duration::from_secs(50).as_nanos());
    test.send(Event::Restored);
    test.env.now = Time::from_nanos(Duration::from_secs(45).as_nanos());
    test.env.wall = Wall::from_nanos(Duration::from_secs(55).as_nanos());
    assert_eq!(reply(&test.request(10, 1, ask(1))), Reply::SignedOut);
}

#[test]
fn authenticated_escalation_decisions_route_without_membership_and_io_pressure_is_retryable() {
    let mut test = Test::new(LIMITS);
    test.signin(1, 10, identity(0, 1));
    let ask = Ask::DecideEscalation { project: 1, task: 9, revision: 2, decision: EscalationDecision::Pass };
    let routed = test.request(10, 2, ask.clone());
    assert!(routed.iter().any(|row| match row {
        Request::Route { person, role, ask: routed, .. } => *person == 1 && role.is_none() && **routed == ask,
        Request::Reply { .. }
        | Request::Save { .. }
        | Request::Erase { .. }
        | Request::RolesApplied { .. }
        | Request::RolesRefused { .. }
        | Request::ServiceMade { .. }
        | Request::RestoreRefused { .. } => false,
    }));
    let request = route(&routed);
    assert_eq!(
        reply(&test.send(Event::Decided { request, outcome: Outcome::Again(Again::Busy) })),
        Reply::Again(Again::Busy)
    );
    let retried = test.request(10, 2, ask.clone());
    let request = route(&retried);
    let outcome = Outcome::EscalationDecided {
        task: 9,
        revision: 2,
        by: 7,
        at: Wall::from_nanos(0),
        choice: EscalationChoice::Passed,
    };
    test.send(Event::Decided { request, outcome });
    assert_eq!(reply(&test.request(10, 2, ask)), Reply::Outcome(outcome));
    assert_eq!(
        reply(&test.request(
            10,
            2,
            Ask::DecideEscalation { project: 1, task: 9, revision: 2, decision: EscalationDecision::Release }
        )),
        Reply::Refused(Refusal::KeyConflict)
    );
    test.env.now = Time::from_nanos(Duration::from_secs(61).as_nanos());
    assert_eq!(
        reply(&test.request(
            10,
            3,
            Ask::DecideEscalation { project: 1, task: 9, revision: 2, decision: EscalationDecision::Pass }
        )),
        Reply::SignedOut
    );
}

#[test]
#[expect(clippy::wildcard_enum_match_arm, reason = "the test scripts only three revision-bearing asks")]
fn a_decisions_revision_is_part_of_its_keyed_payload_and_moved_facts_replay() {
    let asks = [
        Ask::Release { project: 1, task: 7, revision: 1, again: None },
        Ask::AnswerPerson { project: 1, task: 7, revision: 1, result: PersonResult::Report { words: Box::new([1]) } },
        Ask::Amend {
            release: None,
            project: 1,
            task: 7,
            revision: 1,
            amendment: Amendment {
                addressed: None,
                instructions: None,
                spec: None,
                wake: None,
                dependencies: None,
                authority: None,
                reason: Box::new([]),
            },
        },
    ];
    let facts = [
        Moved::Hold { revision: 2, reason: HoldReason::Budget },
        Moved::Person { revision: 2 },
        Moved::Spec { revision: 2 },
    ];
    for (ask, now) in asks.into_iter().zip(facts) {
        let mut test = Test::new(LIMITS);
        test.member();
        let request = route(&test.request(10, 14, ask.clone()));
        let outcome = Outcome::Refused(Refusal::Moved { now });
        let outputs = test.send(Event::Decided { request, outcome });
        assert_eq!(reply(&outputs), Reply::Outcome(outcome));
        assert_eq!(reply(&test.request(10, 14, ask.clone())), Reply::Outcome(outcome));
        let changed = match ask {
            Ask::Release { project, task, .. } => Ask::Release { project, task, revision: 2, again: None },
            Ask::AnswerPerson { project, task, result, .. } => Ask::AnswerPerson { project, task, revision: 2, result },
            Ask::Amend { project, task, amendment, .. } => {
                Ask::Amend { project, task, revision: 2, amendment, release: None }
            }
            _ => panic!("only revision-bearing asks scripted"),
        };
        assert_eq!(reply(&test.request(10, 14, changed)), Reply::Refused(Refusal::KeyConflict));
    }
}

#[test]
fn every_final_refusal_keeps_its_facts_under_the_key_when_roles_and_time_change() {
    for refusal in [
        Refusal::Role { held: Some(Role::Member) },
        Refusal::Authority { proposable: None, lacking: Lack::Budget { needs: 30, left: 11 } },
        Refusal::Authority { proposable: None, lacking: Lack::Grant { kind: 5 } },
        Refusal::Authority { proposable: None, lacking: Lack::TaskKind },
        Refusal::Authority { proposable: Some(false), lacking: Lack::Approver },
        Refusal::Authority { proposable: None, lacking: Lack::Depth { most: 2 } },
        Refusal::Authority { proposable: None, lacking: Lack::Deadline },
        Refusal::Authority { proposable: None, lacking: Lack::Tasks { left: 3 } },
        Refusal::Limit { which: LimitKind::ProjectLiveTasks },
        Refusal::Limit { which: LimitKind::InboxFull },
        Refusal::Limit { which: LimitKind::Words },
        Refusal::Moved { now: Moved::Hold { revision: 8, reason: HoldReason::StoppedBy { party: 2 } } },
        Refusal::Moved { now: Moved::Spec { revision: 4 } },
        Refusal::Moved { now: Moved::Person { revision: 3 } },
        Refusal::Moved { now: Moved::Note { revision: 2 } },
        Refusal::Ended { how: Ending::Done },
        Refusal::Ended { how: Ending::Failed },
        Refusal::Ended { how: Ending::Cancelled { by: Party::Person(2), at: Wall::from_nanos(10) } },
        Refusal::Ended { how: Ending::Decided { by: 2, at: Wall::from_nanos(10) } },
        Refusal::Ended { how: Ending::Deleted },
        Refusal::NotWaiting,
        Refusal::NoFurther,
        Refusal::Unknown,
        Refusal::NotOffered,
        Refusal::KeyConflict,
    ] {
        let mut test = Test::new(LIMITS);
        test.member();
        let request = route(&test.request(10, 1, ask(1)));
        let outcome = Outcome::Refused(refusal);
        let decided = test.send(Event::Decided { request, outcome });
        assert_eq!(reply(&decided), Reply::Outcome(outcome));
        assert!(decided.iter().any(is_answer));
        test.send(Event::Roles { project: 1, holdings: Box::new([Holding { person: 1, role: Role::Observer }]) });
        test.env.wall = Wall::from_nanos(20);
        let replay = test.request(10, 1, ask(1));
        assert_eq!(replay.len(), 1);
        assert_eq!(reply(&replay), Reply::Outcome(outcome));
    }
}

#[test]
fn retry_and_signed_out_answers_cannot_restore_as_completed_keys() {
    for outcome in [Outcome::Again(Again::Busy), Outcome::Again(Again::NotReady), Outcome::SignedOut] {
        let mut test = Test::new(LIMITS);
        test.d = Domain::new(&LIMITS, Box::new([]), 2);
        let key = RequestKey { person: 1, key: [1; 16] };
        assert_eq!(
            test.send(Event::Restore {
                record: Stored::Answer { key, ask: Box::new(ask(1)), outcome, at: Wall::EPOCH }
            }),
            [Request::RestoreRefused { key: Key::Answer(key), refusal: Refusal::Limit { which: LimitKind::Restore } }],
        );
        assert!(!test.d.ready());
    }
}

fn application(digest: u8) -> Ask {
    Ask::Application { project: 1, connector: 2, digest: [digest; 32] }
}

#[test]
fn application_keys_join_flights_and_replay_opaque_tokens_across_sign_ins_and_restore() {
    let mut test = Test::new(LIMITS);
    test.member();
    test.signin(99, 11, identity(0, 1));
    let request = route(&test.request(10, 7, application(1)));
    assert!(test.request(11, 7, application(1)).is_empty());
    assert_eq!(reply(&test.request(10, 7, application(2))), Reply::Refused(Refusal::KeyConflict));
    assert_eq!(reply(&test.request(10, 7, ask(1))), Reply::Refused(Refusal::KeyConflict));
    let outcome = Outcome::Applied { connector: 2, token: Token::new(42) };
    let rows = test.send(Event::Decided { request, outcome });
    assert_eq!(
        rows.iter()
            .filter(|row| match row {
                Request::Reply { .. } => true,
                Request::Route { .. }
                | Request::Save { .. }
                | Request::Erase { .. }
                | Request::ServiceMade { .. }
                | Request::RolesApplied { .. }
                | Request::RolesRefused { .. }
                | Request::RestoreRefused { .. } => false,
            })
            .count(),
        2
    );
    let saved = saved_answer(&rows);
    assert_eq!(reply(&test.request(11, 7, application(1))), Reply::Outcome(outcome));
    test.send(Event::Roles { project: 1, holdings: Box::new([]) });
    assert_eq!(reply(&test.request(10, 7, application(1))), Reply::Outcome(outcome));
    let mut d = Domain::new(&LIMITS, Box::new([]), 2);
    for record in [
        Stored::Person { number: 1, identity: identity(0, 1) },
        Stored::SignIn { number: 10, person: 1, expires: Wall::from_nanos(LIMITS.sign_in_lifetime.as_nanos()) },
        saved,
    ] {
        let mut out = Queue::with_capacity(max_out(&LIMITS));
        step(&mut d, &test.env, Event::Restore { record }, &mut out);
        assert!(out.is_empty());
    }
    test.d = d;
    assert!(test.send(Event::Restored).is_empty());
    assert_eq!(reply(&test.request(10, 7, application(1))), Reply::Outcome(outcome));
    assert_eq!(reply(&test.request(10, 7, application(2))), Reply::Refused(Refusal::KeyConflict));
}

#[test]
fn application_admission_shares_capacity_and_retention_and_requires_a_live_sign_in_and_role() {
    let mut test = Test::new(Limits { requests: 1, ..LIMITS });
    assert_eq!(reply(&test.request(10, 1, application(1))), Reply::SignedOut);
    test.member();
    test.send(Event::Roles { project: 1, holdings: Box::new([Holding { person: 1, role: Role::Observer }]) });
    let request = route(&test.request(10, 1, application(1)));
    assert_eq!(reply(&test.request(10, 2, ask(1))), Reply::Again(Again::Busy));
    let outcome = Outcome::Applied { connector: 2, token: Token::new(42) };
    test.send(Event::Decided { request, outcome });
    assert_eq!(reply(&test.request(10, 2, application(1))), Reply::Again(Again::Busy));
    test.env.now = test.env.now.saturating_add(LIMITS.request_retention);
    test.env.wall = Wall::from_nanos(LIMITS.request_retention.as_nanos());
    test.signin(1, 11, identity(0, 1));
    assert_ne!(route(&test.request(11, 1, application(2))), Token::new(0));
    let mut no_role = Test::new(LIMITS);
    no_role.signin(1, 10, identity(0, 1));
    assert_eq!(
        reply(&no_role.request(10, 1, application(1))),
        Reply::Outcome(Outcome::Refused(Refusal::Role { held: None }))
    );
}

#[test]
fn restored_application_tokens_must_name_their_connector_and_a_positive_outcome() {
    for outcome in [
        Outcome::Applied { connector: 3, token: Token::new(42) },
        Outcome::Applied { connector: 2, token: Token::new(0) },
        Outcome::Started { task: 42 },
    ] {
        let mut test = Test::new(LIMITS);
        test.d = Domain::new(&LIMITS, Box::new([]), 2);
        let rows = test.send(Event::Restore {
            record: Stored::Answer {
                key: RequestKey { person: 1, key: [1; 16] },
                ask: Box::new(application(1)),
                outcome,
                at: Wall::EPOCH,
            },
        });
        match rows.as_slice() {
            [Request::RestoreRefused { .. }] => {}
            _ => panic!("invalid restored application outcome is refused"),
        }
        test.send(Event::Restored);
        assert!(!test.d.ready());
    }
}

#[test]
fn a_said_answer_keeps_the_release_fact_and_request_choice() {
    let mut test = Test::new(LIMITS);
    test.member();
    let ask = Ask::Say { project: 1, task: 7, words: Box::new([1]), release: true };
    let request = route(&test.request(10, 1, ask.clone()));
    let outcome = Outcome::Said { task: 7, message: 8, released: true };
    let out = test.send(Event::Decided { request, outcome });
    assert_eq!(reply(&out), Reply::Outcome(outcome));
    assert_eq!(reply(&test.request(10, 1, ask)), Reply::Outcome(outcome));
}

#[test]
fn needs_count_every_live_entry_across_projects_past_the_cache() {
    let mut test = Test::new(LIMITS);
    test.signin(1, 10, identity(0, 1));
    for project in [1, 2] {
        test.send(Event::Roles { project, holdings: Box::new([Holding { person: 1, role: Role::Member }]) });
    }
    for task in 1..=5 {
        let project = if task % 2 == 0 { 2 } else { 1 };
        let entry = Entry { task, project, whom: Whom::Person(1), kind: EntryKind::PersonTask, at: Wall::EPOCH };
        test.send(Event::Waiting { task, entries: Box::new([entry]) });
    }
    assert_eq!(test.d.needs(1), Some(5));
    assert_eq!(test.d.cached_waiting(&LIMITS, 1).unwrap().len(), 2);
    assert!(test.d.task_needs(1, 1), "oldest need remains counted after cache eviction");
    test.send(Event::Waiting { task: 1, entries: Box::new([]) });
    assert_eq!(test.d.needs(1), Some(4));
    test.send(Event::Roles { project: 2, holdings: Box::new([]) });
    assert_eq!(test.d.needs(1), Some(2), "only current project roles contribute");
}

#[test]
fn a_role_need_leaves_every_holders_count_when_one_acts() {
    let mut test = Test::new(LIMITS);
    test.signin(1, 10, identity(0, 1));
    test.signin(2, 20, identity(0, 2));
    test.send(Event::Roles {
        project: 1,
        holdings: Box::new([Holding { person: 1, role: Role::Member }, Holding { person: 2, role: Role::Member }]),
    });
    for task in 1..=3 {
        let entry = Entry {
            task,
            project: 1,
            whom: Whom::Role { project: 1, role: Role::Member.number() },
            kind: EntryKind::PersonTask,
            at: Wall::EPOCH,
        };
        test.send(Event::Waiting { task, entries: Box::new([entry]) });
    }
    assert_eq!(test.d.needs(1), Some(3));
    assert_eq!(test.d.needs(2), Some(3));
    let request = route(&test.request(10, 1, Ask::TakePerson { project: 1, task: 1 }));
    test.send(Event::Decided { request, outcome: Outcome::PersonTaken { task: 1 } });
    test.send(Event::Waiting { task: 1, entries: Box::new([]) });
    assert_eq!(test.d.needs(1), Some(2));
    assert_eq!(test.d.needs(2), Some(2));
}

#[test]
fn an_amendment_answer_keeps_the_release_fact_and_named_hold_in_its_key() {
    let mut test = Test::new(LIMITS);
    test.member();
    let change = Amendment {
        addressed: None,
        instructions: None,
        spec: None,
        wake: None,
        dependencies: None,
        authority: None,
        reason: Box::new([]),
    };
    let ask = Ask::Amend { project: 1, task: 7, revision: 1, amendment: change.clone(), release: Some(2) };
    let request = route(&test.request(10, 1, ask.clone()));
    let outcome = Outcome::Amended { task: 7, released: true };
    let out = test.send(Event::Decided { request, outcome });
    assert_eq!(reply(&out), Reply::Outcome(outcome));
    assert_eq!(reply(&test.request(10, 1, ask)), Reply::Outcome(outcome));
    let changed = Ask::Amend { project: 1, task: 7, revision: 1, amendment: change, release: Some(3) };
    assert_eq!(reply(&test.request(10, 1, changed)), Reply::Refused(Refusal::KeyConflict));
}

#[test]
fn capacity_facts_keep_actual_times_replay_and_diagnostics_independent() {
    let mut test = Test::new(Limits { requests: 1, room_refusals: 1, facts: 0, ..LIMITS });
    test.member();
    let request = route(&test.request(10, 1, ask(1)));
    assert!(test.request(10, 1, ask(1)).is_empty());
    test.env.wall = Wall::from_nanos(7);
    assert_eq!(reply(&test.request(10, 1, ask(1))), Reply::Again(Again::Busy));
    let first = test.d.capacity_refusals();
    assert_eq!(first.len(), 1);
    assert_eq!(first.get(0).expect("waiter refusal").cause, CapacityCause::Waiters { limit: 2 });
    assert_eq!(first.get(0).expect("actual time").at, test.env.wall);
    test.env.wall = Wall::from_nanos(8);
    assert_eq!(reply(&test.request(10, 2, ask(1))), Reply::Again(Again::Busy));
    assert_eq!(test.d.capacity_refusals_lost(), 1);
    let entrance = test.d.capacity_refusals();
    let value = entrance.get(0).expect("request pressure");
    assert_eq!(value.key, RequestKey { person: 1, key: [2; 16] });
    assert_eq!(value.project, Some(1));
    assert_eq!(value.action, CapacityAction::Chat);
    assert_eq!(value.cause, CapacityCause::Requests { limit: 1 });
    assert_eq!(value.at, Wall::from_nanos(8));
    assert_eq!(reply(&test.request(10, 1, ask(2))), Reply::Refused(Refusal::KeyConflict));
    assert_eq!(test.d.capacity_refusals(), entrance);
    assert_eq!(test.d.pop_fact(), None);
    assert_eq!(test.d.capacity_refusals(), entrance);
    test.env.wall = Wall::from_nanos(9);
    let refusal = Outcome::Refused(Refusal::Limit { which: LimitKind::LiveTasks });
    let completed = test.send(Event::Decided { request, outcome: refusal });
    assert_eq!(reply(&completed), Reply::Outcome(refusal));
    let permanent = test.d.capacity_refusals();
    assert_eq!(permanent.len(), 2);
    assert_eq!(permanent.get(0).expect("newest answer").key.key, [1; 16]);
    assert_eq!(permanent.get(0).expect("original time").at, Wall::from_nanos(9));
    test.env.wall = Wall::from_nanos(10);
    assert_eq!(reply(&test.request(10, 1, ask(1))), Reply::Outcome(refusal));
    assert_eq!(test.d.capacity_refusals(), permanent, "replay creates no refusal or new timestamp");
    let mut cold = Test::new(test.env.limits);
    cold.env.wall = test.env.wall;
    cold.d = Domain::new(&cold.env.limits, Box::new([]), 2);
    cold.send(Event::Restore { record: saved_answer(&completed) });
    cold.send(Event::Restore { record: Stored::Person { number: 1, identity: identity(0, 1) } });
    cold.send(Event::Restored);
    assert!(cold.d.ready());
    let restored = cold.d.capacity_refusals();
    assert_eq!(restored.len(), 1, "entrance history is live-only");
    assert_eq!(restored.get(0), permanent.get(0), "permanent facts retain the durable answer time");
    assert_eq!(cold.d.capacity_refusals_lost(), 0);
    cold.env.wall = Wall::from_nanos(Duration::from_secs(120).as_nanos() + 10);
    cold.env.now = Time::from_nanos(cold.env.wall.as_nanos());
    let mut out = Queue::with_capacity(max_out(&cold.env.limits));
    fire(&mut cold.d, &cold.env, &mut out);
    assert!(cold.d.capacity_refusals().is_empty(), "permanent facts follow keyed-answer retention");
}

#[test]
fn zero_room_history_counts_omissions_without_changing_busy_or_key_admission() {
    let mut test = Test::new(Limits { pending: 0, room_refusals: 0, ..LIMITS });
    test.member();
    assert_eq!(reply(&test.request(10, 1, ask(1))), Reply::Again(Again::Busy));
    assert!(test.d.capacity_refusals().is_empty());
    assert_eq!(test.d.capacity_refusals_lost(), 1);
    assert_eq!(reply(&test.request(10, 1, ask(1))), Reply::Again(Again::Busy));
    assert_eq!(test.d.capacity_refusals_lost(), 2);
    assert_eq!(worst_case(&Limits { requests: u32::MAX, room_refusals: 1, ..LIMITS }), None);
    assert_eq!(capacity::permanent(Outcome::Refused(Refusal::Limit { which: LimitKind::Words })), None);
    assert_eq!(capacity::permanent(Outcome::Refused(Refusal::Role { held: Some(Role::Observer) })), None);
    assert_eq!(
        capacity::permanent(Outcome::Again(Again::Busy)),
        None,
        "routed busy is not assumed to be room pressure"
    );
}

#[test]
fn read_position_room_pressure_keeps_scoped_identity_without_a_project() {
    let mut test = Test::new(Limits { pending: 0, room_refusals: 1, ..LIMITS });
    test.member();
    let seen = Seen::Chat { task: 91, through: 8 };
    test.env.wall = Wall::from_nanos(17);
    assert_eq!(reply(&test.request(10, 3, Ask::MarkSeen { seen })), Reply::Again(Again::Busy));
    let facts = test.d.capacity_refusals();
    assert_eq!(facts.len(), 1);
    assert_eq!(
        *facts.get(0).expect("scoped refusal"),
        CapacityRefusal {
            key: RequestKey { person: 1, key: [3; 16] },
            project: None,
            action: CapacityAction::MarkSeen { seen },
            at: Wall::from_nanos(17),
            cause: CapacityCause::Pending { limit: 0 },
        }
    );
    assert_eq!(capacity::permanent(Outcome::MarkedSeen { seen }), None);
}

#[test]
fn a_goal_deadline_is_retained_under_its_key() {
    let mut test = Test::new(LIMITS);
    test.member();
    let ask = Ask::SetGoal {
        project: 1,
        spec: b"deadline".as_slice().into(),
        charter: 1,
        budget: 1,
        priority: 1,
        deadline: Some(Wall::from_nanos(1_000_000_000)),
    };
    let request = route(&test.request(10, 1, ask.clone()));
    let outcome = Outcome::GoalStarted { task: 7 };
    test.send(Event::Decided { request, outcome });
    assert_eq!(reply(&test.request(10, 1, ask.clone())), Reply::Outcome(outcome));
    let Ask::SetGoal { project, spec, charter, budget, priority, .. } = ask else { panic!() };
    let changed =
        Ask::SetGoal { project, spec, charter, budget, priority, deadline: Some(Wall::from_nanos(2_000_000_000)) };
    assert_eq!(reply(&test.request(10, 1, changed)), Reply::Refused(Refusal::KeyConflict));
}

mod closing_chat;
#[test]
fn inbox_counts_follow_current_project_roles_without_losing_archived_census() {
    let mut test = Test::new(LIMITS);
    test.signin(1, 10, identity(0, 1));
    for project in [1, 2] {
        test.send(Event::Roles { project, holdings: Box::new([Holding { person: 1, role: Role::Member }]) });
    }
    test.send(Event::ChatState { party: 1, project: 2, task: 9, last_reply: 3, live: false, replies: Box::new([]) });
    test.d.remember_result(&LIMITS, 1, ResultRef { task: 7, project: 2, at: Wall::from_nanos(7), position: 7 });
    test.d.remember_result(&LIMITS, 1, ResultRef { task: 8, project: 1, at: Wall::from_nanos(8), position: 8 });
    assert_eq!(test.d.counts(&LIMITS, 1), Some(Counts { needs: 0, unread: 2, more_unread: true }));
    test.send(Event::Roles { project: 2, holdings: Box::new([]) });
    assert_eq!(test.d.counts(&LIMITS, 1), Some(Counts { needs: 0, unread: 1, more_unread: false }));
    test.send(Event::Roles { project: 2, holdings: Box::new([Holding { person: 1, role: Role::Member }]) });
    assert_eq!(test.d.counts(&LIMITS, 1), Some(Counts { needs: 0, unread: 2, more_unread: true }));
    let updates = test.d.cached_updates(&LIMITS, 1).expect("known party");
    assert_eq!(updates[0], Update::Result(ResultRef { task: 8, project: 1, at: Wall::from_nanos(8), position: 8 }));
    assert_eq!(updates[1], Update::Result(ResultRef { task: 7, project: 2, at: Wall::from_nanos(7), position: 7 }));
}

mod need_counts;
