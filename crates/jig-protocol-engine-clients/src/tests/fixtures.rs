use super::super::*;
use skein_lib::{Duration, List, Secret, Wall, Writer};
pub const NUMBER: u64 = 41;
pub fn wire() -> w::Limits {
    w::Limits {
        keyed_request: 256,
        owned_record: 256,
        ending_rows: 128,
        start_chat_words: 64,
        amend_reason: 64,
        ..w::CEILINGS
    }
}
pub fn settings() -> Settings {
    let mut served = List::with_capacity(2);
    served.push(crate::Served { family: None, first: 8, last: 8 }).expect("one jig range");
    served.push(crate::Served { family: Some(9), first: 2, last: 2 }).expect("one family range");
    Settings {
        deployment_provider: 2,
        listener: skein_io::Addr::Ip(::core::net::SocketAddr::new(::core::net::IpAddr::from([127, 0, 0, 1]), 8080)),
        link: None,
        heartbeat: Duration::from_secs(5),
        rows: 32,
        served,
        providers: List::with_capacity(0),
        seed: Secret::new(Box::new([7; 32])),
        stream: 31,
    }
}
pub fn env() -> Env<Limits> {
    Env {
        now: Time::ZERO,
        wall: Wall::EPOCH,
        limits: Limits {
            connections: 4,
            streams: 2,
            watches_per_stream: 4,
            queued_bytes: 65536,
            identity_bytes: 64,
            families: 2,
            providers: 0,
            exchanges: 4,
            exchanges_per_client: 2,
            http: http::Limits { head: 512, headers: 4, body: 256, read: 32, response: 256, send: 32 },
            wire: wire(),
            link_span: Duration::from_secs(300),
            stall: Duration::from_secs(30),
        },
    }
}
pub fn encode_ask(ask: &w::Ask) -> Box<[u8]> {
    let mut out = Writer::new(usize::try_from(ask.measure()).expect("short ask"));
    ask.encode(&mut out).expect("bounded ask");
    out.finish()
}
pub fn keyed(owner: w::Owner, version: u32, request: Box<[u8]>) -> Box<[u8]> {
    let value = w::Keyed::new(&wire(), w::KeyedParts { key: [5; 16], owner, version, request }).expect("keyed");
    let mut out = Writer::new(usize::try_from(value.measure()).expect("short keyed record"));
    value.encode(&mut out).expect("bounded keyed");
    out.finish()
}
pub fn start(limits: &w::Limits) -> w::Ask {
    w::Ask::StartChat(
        w::StartChat::new(limits, w::StartChatParts { project: 3, words: Box::from(b"hello".as_slice()) })
            .expect("start"),
    )
}
pub fn revision(value: u64) -> w::Revision {
    w::Revision::new(&wire(), w::RevisionParts { value }).expect("revision")
}
pub fn amendment(release: Option<w::Revision>, reason: &[u8]) -> w::Ask {
    w::Ask::Amend(Box::new(
        w::Amend::new(
            &wire(),
            w::AmendParts {
                project: 3,
                task: 8,
                revision: revision(6),
                release,
                reason: Box::from(reason),
                change: w::Amendment::new(
                    &wire(),
                    w::AmendmentParts {
                        addressed: None,
                        spec: None,
                        wake: None,
                        dependencies: Some(List::with_capacity(0)),
                        authority: None,
                        instructions: Some(Box::new([])),
                    },
                )
                .expect("change"),
            },
        )
        .expect("amend"),
    ))
}
pub struct Exchange {
    pub component: Component,
    pub env: Env<Limits>,
    pub out: Queue<Request>,
    pub token: Box<[u8]>,
}
impl Exchange {
    pub fn new() -> Self {
        Self::with_env(env())
    }
    pub fn with_env(env: Env<Limits>) -> Self {
        let mut out = Queue::with_capacity(8);
        let component = Component::new(settings(), &env, &mut out).expect("starts");
        out.pop();
        let mut token = Writer::new(40);
        token.put(&NUMBER.to_be_bytes()).expect("number");
        token.put(&[7; 32]).expect("random bytes");
        Self { component, env, out, token: token.finish() }
    }
    pub fn post(&mut self, body: Box<[u8]>) -> Token {
        self.post_to(b"/jig/v1/request", body)
    }
    pub fn post_to(&mut self, target: &[u8], body: Box<[u8]>) -> Token {
        self.post_to_method(target, body, skein_http::Method::Post)
    }
    pub fn post_to_method(&mut self, target: &[u8], body: Box<[u8]>, method: skein_http::Method) -> Token {
        let owner = self.component.name();
        let mut connection = Connection::new(owner, &self.env.limits, self.env.now);
        connection.requests.pop();
        let encoded = sign_in::url(&self.token);
        let mut bearer = Writer::new(encoded.len().checked_add(7).expect("bearer size"));
        bearer.put(b"Bearer ").expect("prefix");
        bearer.put(&encoded).expect("token");
        let call = http::Call {
            method,
            target: Box::from(target),
            headers: Box::new([skein_http::Header {
                name: Box::from(b"authorization".as_slice()),
                value: bearer.finish(),
            }]),
            body: http::Body::Length(u64::try_from(body.len()).expect("short body")),
            version: skein_http::Version::Http11,
        };
        self.component.http(&self.env, &mut connection, http::Event::Call(call), &mut self.out);
        connection.body.append(&body).expect("body fits");
        self.component.http(&self.env, &mut connection, http::Event::Body(Up::End), &mut self.out);
        self.component.connections.insert(owner, connection).expect("test connection");
        owner
    }
    pub fn authenticate(&mut self) {
        let Some(Request::Secrets(SecretsRequest::Get { owner })) = self.out.pop() else {
            panic!("digest get");
        };
        assert_eq!(owner, SecretOwner::SignIn(NUMBER));
        self.component.down(
            &self.env,
            Event::Secrets(SecretsAnswer::Got {
                owner: SecretOwner::SignIn(NUMBER),
                value: Some(Kept {
                    digest: sign_in::digest(&self.token),
                    expires: Wall::from_nanos(86_400_000_000_000),
                }),
            }),
            &mut self.out,
        );
        assert!(self.component.up(&self.env, &mut self.out));
    }
    pub fn response(&self, owner: Token) -> (u16, &[u8]) {
        let connection = self.component.connections.get(&owner).expect("connection");
        let mut status = None;
        for request in &connection.requests {
            match request {
                http::Request::Respond(response) => status = Some(response.status),
                http::Request::Next
                | http::Request::Body(_)
                | http::Request::Discard
                | http::Request::Reply(_)
                | http::Request::Close => {}
            }
        }
        let status = status.expect("response");
        (status, &connection.response)
    }
}

pub fn other_asks() -> [(w::Ask, people::Ask); 20] {
    use jig_domain_people as p;
    let limits = wire();
    let mut priorities = List::with_capacity(1);
    priorities
        .push(w::Priority::new(&limits, w::PriorityParts { task: 8, priority: u32::MAX }).expect("priority"))
        .expect("room");
    let mut disposals = List::with_capacity(2);
    disposals
        .push(w::Disposal::Cancel(w::DisposalCancel::new(&limits, w::DisposalCancelParts { task: 9 }).expect("cancel")))
        .expect("room");
    disposals
        .push(w::Disposal::TakeOver(
            w::DisposalTakeOver::new(&limits, w::DisposalTakeOverParts { task: 10 }).expect("take"),
        ))
        .expect("room");
    let mut expected_disposals = List::with_capacity(2);
    expected_disposals.push(p::Disposal { task: 9, keep: p::Keep::Cancel }).expect("room");
    expected_disposals.push(p::Disposal { task: 10, keep: p::Keep::MakeMine }).expect("room");
    let mut holdings = List::with_capacity(1);
    holdings
        .push(
            w::Holding::new(
                &limits,
                w::HoldingParts {
                    party: 7,
                    role: w::HeldRole::Policy(
                        w::HeldRolePolicy::new(&limits, w::HeldRolePolicyParts { role: u32::MAX }).expect("role"),
                    ),
                },
            )
            .expect("holding"),
        )
        .expect("room");
    let mut references = List::with_capacity(1);
    references.push(u64::MAX).expect("room");
    [
        write_case(&limits),
        answer_question_case(&limits),
        take_case(&limits),
        hand_back_case(&limits),
        release_case(&limits),
        stop_case(&limits),
        cancel_case(&limits),
        take_over_case(&limits),
        set_pool_case(&limits),
        prioritise_case(&limits, priorities),
        close_case(&limits, disposals, expected_disposals),
        mark_seen_case(&limits),
        set_goal_case(&limits),
        answer_person_case(&limits),
        decide_proposal_case(&limits),
        set_roles_case(&limits, holdings),
        make_service_case(&limits),
        change_policy_case(&limits),
        delete_note_case(&limits),
        edit_note_case(&limits, references),
    ]
}
fn write_case(limits: &w::Limits) -> (w::Ask, people::Ask) {
    use jig_domain_people as p;
    (
        w::Ask::Write(
            w::Write::new(
                limits,
                w::WriteParts { project: 3, task: 8, words: Box::from(b"say".as_slice()), release: true },
            )
            .expect("Write"),
        ),
        p::Ask::Say { project: 3, task: 8, words: Box::from(b"say".as_slice()), release: true },
    )
}
fn answer_question_case(limits: &w::Limits) -> (w::Ask, people::Ask) {
    use jig_domain_people as p;
    (
        w::Ask::AnswerQuestion(
            w::AnswerQuestion::new(
                limits,
                w::AnswerQuestionParts { project: 3, task: 8, message: 19, words: Box::from(b"answer".as_slice()) },
            )
            .expect("AnswerQuestion"),
        ),
        p::Ask::AnswerQuestion { project: 3, task: 8, question: 19, words: Box::from(b"answer".as_slice()) },
    )
}
fn take_case(limits: &w::Limits) -> (w::Ask, people::Ask) {
    use jig_domain_people as p;
    (
        w::Ask::Take(w::Take::new(limits, w::TakeParts { project: 3, task: 8 }).expect("Take")),
        p::Ask::TakePerson { project: 3, task: 8 },
    )
}
fn hand_back_case(limits: &w::Limits) -> (w::Ask, people::Ask) {
    use jig_domain_people as p;
    (
        w::Ask::HandBack(w::HandBack::new(limits, w::HandBackParts { project: 3, task: 8 }).expect("HandBack")),
        p::Ask::HandBackPerson { project: 3, task: 8 },
    )
}
fn release_case(limits: &w::Limits) -> (w::Ask, people::Ask) {
    use jig_domain_people as p;
    (
        w::Ask::Release(
            w::Release::new(limits, w::ReleaseParts { project: 3, task: 8, revision: revision(6), again: Some(false) })
                .expect("Release"),
        ),
        p::Ask::Release { project: 3, task: 8, revision: 6, again: Some(false) },
    )
}
fn stop_case(limits: &w::Limits) -> (w::Ask, people::Ask) {
    use jig_domain_people as p;
    (
        w::Ask::Stop(w::Stop::new(limits, w::StopParts { project: 3, task: 8 }).expect("Stop")),
        p::Ask::Stop { project: 3, task: 8 },
    )
}
fn cancel_case(limits: &w::Limits) -> (w::Ask, people::Ask) {
    use jig_domain_people as p;
    (
        w::Ask::Cancel(
            w::Cancel::new(limits, w::CancelParts { project: 3, task: 8, reason: Box::from(b"cancel".as_slice()) })
                .expect("Cancel"),
        ),
        p::Ask::Cancel { project: 3, task: 8, reason: Box::from(b"cancel".as_slice()), grace: None },
    )
}
fn take_over_case(limits: &w::Limits) -> (w::Ask, people::Ask) {
    use jig_domain_people as p;
    (
        w::Ask::TakeOver(
            w::TakeOver::new(limits, w::TakeOverParts { project: 3, task: 8, reason: Box::from(b"take".as_slice()) })
                .expect("TakeOver"),
        ),
        p::Ask::Move { project: 3, task: 8, reason: Box::from(b"take".as_slice()) },
    )
}
fn set_pool_case(limits: &w::Limits) -> (w::Ask, people::Ask) {
    use jig_domain_people as p;
    (
        w::Ask::SetPool(
            w::SetPool::new(limits, w::SetPoolParts { project: 3, party: u64::MAX, budget: u64::MAX })
                .expect("SetPool"),
        ),
        p::Ask::SetPool { project: 3, person: u64::MAX, budget: u64::MAX },
    )
}
fn prioritise_case(limits: &w::Limits, priorities: List<w::Priority>) -> (w::Ask, people::Ask) {
    use jig_domain_people as p;
    (
        w::Ask::Prioritise(
            w::Prioritise::new(limits, w::PrioritiseParts { project: 3, goals: priorities }).expect("Prioritise"),
        ),
        p::Ask::Prioritise { project: 3, goals: Box::new([(8, u32::MAX)]) },
    )
}
fn close_case(
    limits: &w::Limits,
    disposals: List<w::Disposal>,
    expected_disposals: List<people::Disposal>,
) -> (w::Ask, people::Ask) {
    use jig_domain_people as p;
    (
        w::Ask::Close(w::Close::new(limits, w::CloseParts { project: 3, chat: 8, disposals }).expect("Close")),
        p::Ask::Close { project: 3, chat: 8, disposals: expected_disposals },
    )
}
fn mark_seen_case(limits: &w::Limits) -> (w::Ask, people::Ask) {
    use jig_domain_people as p;
    (
        w::Ask::MarkSeen(
            w::MarkSeen::new(
                limits,
                w::MarkSeenParts {
                    seen: w::Seen::Replies(
                        w::SeenReplies::new(limits, w::SeenRepliesParts { chat: 8, through: u64::MAX }).expect("seen"),
                    ),
                },
            )
            .expect("MarkSeen"),
        ),
        p::Ask::MarkSeen { seen: p::Seen::Chat { task: 8, through: u64::MAX } },
    )
}
fn set_goal_case(limits: &w::Limits) -> (w::Ask, people::Ask) {
    use jig_domain_people as p;
    (
        w::Ask::SetGoal(
            w::SetGoal::new(
                limits,
                w::SetGoalParts {
                    project: 3,
                    spec: Box::new([]),
                    executor: w::GoalExecutor::Procedure(
                        w::GoalExecutorProcedure::new(
                            limits,
                            w::GoalExecutorProcedureParts { connector: u16::MAX, code: u32::MAX },
                        )
                        .expect("executor"),
                    ),
                    budget: u64::MAX,
                    priority: u32::MAX,
                    deadline: Some(99),
                },
            )
            .expect("SetGoal"),
        ),
        p::Ask::SetGoal {
            project: 3,
            spec: Box::new([]),
            executor: p::GoalExecutor::Procedure { connector: u16::MAX, code: u32::MAX },
            budget: u64::MAX,
            priority: u32::MAX,
            deadline: Some(Wall::from_nanos(99)),
        },
    )
}
fn answer_person_case(limits: &w::Limits) -> (w::Ask, people::Ask) {
    use jig_domain_people as p;
    (
        w::Ask::AnswerPerson(
            w::AnswerPerson::new(
                limits,
                w::AnswerPersonParts {
                    project: 3,
                    task: 8,
                    revision: revision(6),
                    answer: w::PersonAnswer::Approved(
                        w::PersonAnswerApproved::new(
                            limits,
                            w::PersonAnswerApprovedParts { state: Box::new([255, 0]), words: Some(Box::new([])) },
                        )
                        .expect("answer"),
                    ),
                },
            )
            .expect("AnswerPerson"),
        ),
        p::Ask::AnswerPerson {
            project: 3,
            task: 8,
            revision: 6,
            result: p::PersonResult::Approved { state: Box::new([255, 0]), words: Some(Box::new([])) },
        },
    )
}
fn decide_proposal_case(limits: &w::Limits) -> (w::Ask, people::Ask) {
    use jig_domain_people as p;
    (
        w::Ask::DecideProposal(
            w::DecideProposal::new(
                limits,
                w::DecideProposalParts {
                    project: 3,
                    proposal: w::ProposalKey::new(
                        limits,
                        w::ProposalKeyParts {
                            proposer: w::Proposer::Party(
                                w::ProposerParty::new(limits, w::ProposerPartyParts { value: u64::MAX })
                                    .expect("party"),
                            ),
                            number: 5,
                        },
                    )
                    .expect("proposal"),
                    decision: w::Decision::Accepted,
                },
            )
            .expect("DecideProposal"),
        ),
        p::Ask::DecideProposal {
            project: 3,
            proposer: p::Proposer::Party(u64::MAX),
            proposal: 5,
            decision: p::ProposalDecision::Accept,
        },
    )
}
fn set_roles_case(limits: &w::Limits, holdings: List<w::Holding>) -> (w::Ask, people::Ask) {
    use jig_domain_people as p;
    (
        w::Ask::SetRoles(w::SetRoles::new(limits, w::SetRolesParts { project: 3, holdings }).expect("SetRoles")),
        p::Ask::SetRoles {
            project: 3,
            holdings: Box::new([p::Holding { person: 7, role: p::Role::Policy { role: u32::MAX } }]),
        },
    )
}
fn make_service_case(limits: &w::Limits) -> (w::Ask, people::Ask) {
    use jig_domain_people as p;
    (
        w::Ask::MakeService(
            w::MakeService::new(
                limits,
                w::MakeServiceParts { project: 3, name: Box::from(b"service".as_slice()), role: w::HeldRole::Owner },
            )
            .expect("MakeService"),
        ),
        p::Ask::MakeService { project: 3, name: Box::from(b"service".as_slice()), role: p::Role::Owner },
    )
}
fn change_policy_case(limits: &w::Limits) -> (w::Ask, people::Ask) {
    use jig_domain_people as p;
    (
        w::Ask::ChangePolicy(
            w::ChangePolicy::new(
                limits,
                w::ChangePolicyParts {
                    project: 3,
                    edit: w::PolicyEdit::ProjectSpend(
                        w::PolicyEditProjectSpend::new(
                            limits,
                            w::PolicyEditProjectSpendParts { period_spend: u64::MAX },
                        )
                        .expect("spend"),
                    ),
                },
            )
            .expect("ChangePolicy"),
        ),
        p::Ask::ChangePolicy { project: 3, change: p::PolicyChange::ProjectSpend { period_spend: u64::MAX } },
    )
}
fn delete_note_case(limits: &w::Limits) -> (w::Ask, people::Ask) {
    use jig_domain_people as p;
    (
        w::Ask::DeleteNote(
            w::DeleteNote::new(
                limits,
                w::DeleteNoteParts {
                    project: 3,
                    note: 9,
                    scope: w::NoteScope::Goal(
                        w::NoteScopeGoal::new(limits, w::NoteScopeGoalParts { project: 3, goal: 8 }).expect("scope"),
                    ),
                    revision: revision(u64::from(u32::MAX)),
                },
            )
            .expect("DeleteNote"),
        ),
        p::Ask::EditNote {
            project: 3,
            name: 9,
            scope: Box::new(p::NoteScope::Goal { goal: 8 }),
            change: Box::new(p::NoteChange::Delete { recalled: u32::MAX }),
        },
    )
}
fn edit_note_case(limits: &w::Limits, references: List<u64>) -> (w::Ask, people::Ask) {
    use jig_domain_people as p;
    (
        w::Ask::EditNote(
            w::EditNote::new(
                limits,
                w::EditNoteParts {
                    project: 3,
                    note: 9,
                    scope: w::NoteScope::Deployment,
                    revision: revision(6),
                    description: Box::new([]),
                    body: Box::new([]),
                    references,
                },
            )
            .expect("EditNote"),
        ),
        p::Ask::EditNote {
            project: 3,
            name: 9,
            scope: Box::new(p::NoteScope::Deployment),
            change: Box::new(p::NoteChange::Correct {
                description: Box::new([]),
                body: Box::new([]),
                references: Box::new([u64::MAX]),
                recalled: 6,
            }),
        },
    )
}

pub fn authority_ask() -> (w::Ask, people::Ask) {
    let limits = wire();
    let pattern = w::OwnerPattern::new(
        &limits,
        w::OwnerPatternParts {
            segments: List::with_capacity(0),
            last: w::OwnerPatternEnd::Open(
                w::OwnerPatternEndOpen::new(&limits, w::OwnerPatternEndOpenParts { value: Box::new([255, 0]) })
                    .expect("end"),
            ),
        },
    )
    .expect("pattern");
    let grant = w::OwnerGrant::new(&limits, w::OwnerGrantParts { connector: 5, kind: 6, pattern: pattern.clone() })
        .expect("grant");
    let mut writer = Writer::new(usize::try_from(grant.measure()).expect("grant size"));
    grant.encode(&mut writer).expect("grant bytes");
    let bytes = writer.finish();
    let grants = piece(&bytes);
    let resource =
        w::OwnerResourceScope::new(&limits, w::OwnerResourceScopeParts { connector: 7, pattern }).expect("scope");
    let mut writer = Writer::new(usize::try_from(resource.measure()).expect("scope size"));
    resource.encode(&mut writer).expect("scope bytes");
    let bytes = writer.finish();
    let note_resources = piece(&bytes);
    let mut kinds = List::with_capacity(1);
    kinds
        .push(w::OwnerAuthorityExecutor::Role(
            w::OwnerAuthorityExecutorRole::new(&limits, w::OwnerAuthorityExecutorRoleParts { value: u32::MAX })
                .expect("executor"),
        ))
        .expect("room");
    let authority = w::OwnerAuthority::new(
        &limits,
        w::OwnerAuthorityParts {
            tools: u64::MAX,
            grants,
            delegation: w::OwnerDelegation::new(&limits, w::OwnerDelegationParts { kinds, tasks: 9, depth: 10 })
                .expect("delegation"),
            spend: u64::MAX,
            deadline: Some(99),
            notes: 255,
            note_resources,
        },
    )
    .expect("authority");
    let request = w::Ask::ChangePolicy(
        w::ChangePolicy::new(
            &limits,
            w::ChangePolicyParts {
                project: 3,
                edit: w::PolicyEdit::Role(
                    w::PolicyEditRoleValue::new(
                        &limits,
                        w::PolicyEditRoleValueParts {
                            value: w::PolicyEditRole::new(
                                &limits,
                                w::PolicyEditRoleParts {
                                    number: 11,
                                    authority,
                                    period_spend: 12,
                                    requests: 13,
                                    decides: 14,
                                },
                            )
                            .expect("policy role"),
                        },
                    )
                    .expect("role edit"),
                ),
            },
        )
        .expect("policy"),
    );
    (request, expected_authority())
}
fn piece(bytes: &[u8]) -> Box<[u8]> {
    let mut writer = Writer::new(bytes.len().checked_add(4).expect("piece size"));
    skein_codec::piece::write(&mut writer, bytes).expect("piece");
    writer.finish()
}
fn expected_authority() -> people::Ask {
    use jig_domain_people as p;
    p::Ask::ChangePolicy {
        project: 3,
        change: p::PolicyChange::Role(p::PolicyRole {
            number: 11,
            period_spend: 12,
            requests: 13,
            decides: 14,
            authority: p::Authority {
                tools: u64::MAX,
                grants: Box::new([p::Grant {
                    connector: 5,
                    kind: 6,
                    pattern: p::Pattern { segments: Box::new([]), last: p::Last::Open(Box::new([255, 0])) },
                }]),
                delegation: p::Delegation { kinds: Box::new([p::Executor::Role(u32::MAX)]), tasks: 9, depth: 10 },
                spend: u64::MAX,
                deadline: Some(Wall::from_nanos(99)),
                notes: 255,
                note_resources: Box::new([p::ResourceScope {
                    connector: 7,
                    pattern: p::Pattern { segments: Box::new([]), last: p::Last::Open(Box::new([255, 0])) },
                }]),
            },
        }),
    }
}
pub fn malformed_piece() -> w::Ask {
    let limits = wire();
    w::Ask::ChangePolicy(
        w::ChangePolicy::new(
            &limits,
            w::ChangePolicyParts {
                project: 3,
                edit: w::PolicyEdit::Projections(
                    w::PolicyEditProjections::new(
                        &limits,
                        w::PolicyEditProjectionsParts { grants: Box::new([0, 0, 0, 2, 0]) },
                    )
                    .expect("opaque piece"),
                ),
            },
        )
        .expect("policy"),
    )
}

#[path = "fixtures/answers.rs"]
mod answers;
pub use answers::*;
impl Exchange {
    pub fn ask(&mut self) -> (Token, ReplyTo) {
        let body = keyed(w::Owner::Jig, 8, encode_ask(&start(&self.env.limits.wire)));
        let owner = self.post(body);
        self.authenticate();
        let Some(Request::Core(event)) = self.out.pop() else {
            panic!("core ask");
        };
        let core::Event::People(people::Event::Ask { reply_to, .. }) = *event else {
            panic!("expected ask");
        };
        (owner, reply_to)
    }
    pub fn reply(&mut self, to: ReplyTo, reply: people::Reply) {
        self.component.down(&self.env, Event::Reply { to, reply }, &mut self.out);
    }
    pub fn answer(&self, owner: Token) -> w::Answer {
        let (status, bytes) = self.response(owner);
        assert_eq!(status, 200);
        w::Answer::decode(&self.env.limits.wire, &mut skein_lib::Reader::new(bytes)).expect("wire answer")
    }
    pub fn final_answer(&self, owner: Token) -> w::Final {
        match self.answer(owner) {
            w::Answer::FinalAnswer(answer) => {
                assert_eq!(answer.owner(), &w::Owner::Jig);
                assert_eq!(answer.version(), 8);
                w::Final::decode(&self.env.limits.wire, &mut skein_lib::Reader::new(answer.record()))
                    .expect("jig final")
            }
            other @ (w::Answer::Busy | w::Answer::NotReady | w::Answer::SignedOut) => {
                panic!("expected final, got {other:?}")
            }
        }
    }
    pub fn application(&mut self) -> (Token, ReplyTo) {
        let family =
            w::Owner::Family(w::Family::new(&self.env.limits.wire, w::FamilyParts { number: 9 }).expect("family"));
        let owner = self.post(keyed(family, 2, Box::from(b"abc".as_slice())));
        self.authenticate();
        let Some(Request::Application(crate::ToApplication::Request { reply_to, .. })) = self.out.pop() else {
            panic!("application ask");
        };
        (owner, reply_to)
    }
}

#[path = "fixtures/client.rs"]
mod client;
pub use client::client_limits;

#[path = "fixtures/services.rs"]
mod services;
pub use services::*;

#[path = "fixtures/documents.rs"]
mod documents;
pub use documents::*;

#[path = "fixtures/live.rs"]
mod live;
pub use live::*;

#[path = "fixtures/objects.rs"]
mod objects;
pub use objects::*;

#[path = "fixtures/pages.rs"]
mod pages;
pub use pages::*;

#[path = "fixtures/reads.rs"]
mod reads;
pub use reads::*;
#[path = "fixtures/watches.rs"]
mod watches;
pub use watches::*;
#[path = "fixtures/closing.rs"]
mod closing;
pub use closing::*;

/// An amendment retaining every field, with the supplied explanation.
#[must_use]
pub fn person_unchanged(reason: &[u8]) -> jig_domain_people::Amendment {
    jig_domain_people::Amendment {
        tracked: None,
        addressed: None,
        instructions: None,
        spec: None,
        wake: None,
        dependencies: None,
        authority: None,
        reason: reason.into(),
    }
}
