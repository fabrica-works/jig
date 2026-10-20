//! Exact request translation (client/domain/contract.md, 3).
//! Whole supplied fields preserve independent revisions and optional edits.
use super::request_arguments;
mod compound;
mod create;
mod policy;
use jig_client_domain::contract as client;
use jig_domain_people as people;
use policy::policy_edit;
use skein_lib::List;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Invalid {
    Decision,
    NoteRevision,
    NoteProject,
    CompoundProject,
    /// A choice of models, which the joint world configures none of.
    Llm,
}

/// # Errors
/// Refuses an inapplicable closed decision, an overflowing recalled revision, a mismatched note project
/// or a model choice.
#[expect(
    clippy::too_many_lines,
    reason = "one exhaustive closed request match keeps every supported and pending wire arm explicit"
)]
pub fn ask(value: &client::Ask) -> Result<people::Ask, Invalid> {
    Ok(match value {
        client::Ask::Propose { project, action, reason } => create::propose(*project, action, reason)?,
        client::Ask::Withdraw { project, proposal } => people::Ask::Withdraw { project: *project, proposal: *proposal },
        client::Ask::Create { project, tasks } => create::ask(*project, tasks)?,
        client::Ask::Compound { project, operations } => compound::ask(*project, operations)?,
        client::Ask::Message { project, task, words } => {
            people::Ask::Say { project: *project, task: *task, words: Box::from(words.bytes()) }
        }
        client::Ask::Answer { project, on: client::AnswerOn::Question { task, message, words } } => {
            people::Ask::AnswerQuestion {
                project: *project,
                task: *task,
                question: *message,
                words: Box::from(words.bytes()),
            }
        }
        client::Ask::Answer { project, on: client::AnswerOn::Person { task, revision, answer } } => {
            people::Ask::AnswerPerson {
                project: *project,
                task: *task,
                revision: revision.value,
                result: request_arguments::person_answer(answer),
            }
        }
        client::Ask::Take { project, task } => people::Ask::TakePerson { project: *project, task: *task },
        client::Ask::HandBack { project, task } => people::Ask::HandBackPerson { project: *project, task: *task },
        client::Ask::Decide { project, on: client::DecideOn::Proposal { proposal, decision } } => {
            people::Ask::DecideProposal {
                project: *project,
                proposer: match proposal.proposer {
                    client::Proposer::Task(number) => people::Proposer::Task(number),
                    client::Proposer::Party(number) => people::Proposer::Party(number),
                },
                proposal: proposal.number,
                decision: proposal_decision(decision)?,
            }
        }
        client::Ask::Decide { project, on: client::DecideOn::Escalation { task, revision, decision } } => {
            people::Ask::DecideEscalation {
                project: *project,
                task: *task,
                revision: revision.value,
                decision: escalation_decision(decision)?,
            }
        }
        client::Ask::Release { project, task, revision, again } => {
            people::Ask::Release { project: *project, task: *task, revision: revision.value, again: *again }
        }
        client::Ask::Stop { project, task } => people::Ask::Stop { project: *project, task: *task },
        client::Ask::Cancel { project, task, reason, grace } => {
            people::Ask::Cancel { project: *project, task: *task, reason: Box::from(reason.bytes()), grace: *grace }
        }
        client::Ask::Move { project, task, reason } => {
            people::Ask::Move { project: *project, task: *task, reason: Box::from(reason.bytes()) }
        }
        client::Ask::Seen { through: seen } => people::Ask::MarkSeen { seen: request_arguments::seen(*seen) },
        client::Ask::SetPool { project, party, budget } => {
            people::Ask::SetPool { project: *project, person: *party, budget: *budget }
        }
        client::Ask::EditNote { project, note, scope, revision, description, body, references } => {
            people::Ask::EditNote {
                project: *project,
                name: *note,
                scope: Box::new(note_scope(scope, *project)?),
                change: Box::new(people::NoteChange::Correct {
                    description: Box::from(description.bytes()),
                    body: Box::from(body.bytes()),
                    references: references.iter().copied().collect(),
                    recalled: u32::try_from(revision.value).map_err(|_| Invalid::NoteRevision)?,
                }),
            }
        }
        client::Ask::DeleteNote { project, note, scope, revision } => people::Ask::EditNote {
            project: *project,
            name: *note,
            scope: Box::new(note_scope(scope, *project)?),
            change: Box::new(people::NoteChange::Delete {
                recalled: u32::try_from(revision.value).map_err(|_| Invalid::NoteRevision)?,
            }),
        },
        client::Ask::Amend { change, .. } if change.llm.is_some() => return Err(Invalid::Llm),
        client::Ask::Amend { project, task, revision, change, reason } => people::Ask::Amend {
            project: *project,
            task: *task,
            revision: revision.map_or(0, |value| value.value),
            amendment: people::Amendment {
                tracked: change.tracked.map(request_arguments::tracking),
                addressed: change.addressed.map(|value| people::AddressChange {
                    revision: value.revision.value,
                    address: match value.address {
                        client::PersonAddress::Party(number) => people::PersonAddress::Person(number),
                        client::PersonAddress::Role(number) => people::PersonAddress::Role(number),
                    },
                }),
                instructions: change.instructions.as_ref().map(|value| Box::from(value.bytes())),
                spec: change.spec.as_ref().map(specification),
                wake: change.wake.map(request_arguments::wake),
                dependencies: change.dependencies.as_ref().map(|values| values.iter().copied().collect()),
                authority: change.authority.as_ref().map(authority),
                llm: None,
                reason: Box::from(reason.bytes()),
            },
            release: None,
        },
        client::Ask::ChangePolicy { project, edit } => {
            people::Ask::ChangePolicy { project: *project, change: policy_edit(edit) }
        }
        client::Ask::SetRoles { project, holdings } => people::Ask::SetRoles {
            project: *project,
            holdings: holdings
                .iter()
                .map(|value| people::Holding { person: value.party, role: role(value.role) })
                .collect(),
        },
        client::Ask::MakeService { project, name, role: held } => {
            people::Ask::MakeService { project: *project, name: Box::from(name.bytes()), role: role(*held) }
        }
    })
}

fn specification(value: &client::Specification) -> people::Spec {
    people::Spec {
        words: Box::from(value.words.bytes()),
        parameters: value
            .parameters
            .iter()
            .map(|value| match value {
                client::Parameter::Number { name, value } => people::Parameter::Number { name: *name, value: *value },
                client::Parameter::Bytes { name, value } => {
                    people::Parameter::Bytes { name: *name, value: Box::from(value.bytes()) }
                }
                client::Parameter::Resource { name, connector, resource } => {
                    people::Parameter::Resource { name: *name, connector: *connector, resource: *resource }
                }
            })
            .collect(),
        inputs: value.inputs.iter().copied().collect(),
    }
}

fn role(value: client::HeldRole) -> people::Role {
    match value {
        client::HeldRole::Owner => people::Role::Owner,
        client::HeldRole::Maintainer => people::Role::Maintainer,
        client::HeldRole::Member => people::Role::Member,
        client::HeldRole::Observer => people::Role::Observer,
        client::HeldRole::Policy { role } => people::Role::Policy { role },
    }
}

fn pattern(value: &client::OwnerPattern) -> people::Pattern {
    people::Pattern {
        segments: value.segments.iter().cloned().collect(),
        last: match &value.last {
            client::OwnerPatternEnd::Exact(bytes) => people::Last::Exact(bytes.clone()),
            client::OwnerPatternEnd::Open(bytes) => people::Last::Open(bytes.clone()),
        },
    }
}

fn grant(value: &client::OwnerGrant) -> people::Grant {
    people::Grant { connector: value.connector, kind: value.kind, pattern: pattern(&value.pattern) }
}

fn authority(value: &client::OwnerAuthority) -> people::Authority {
    people::Authority {
        tools: value.tools,
        models: 0,
        grants: value.grants.iter().map(grant).collect(),
        delegation: people::Delegation {
            kinds: value
                .delegation
                .kinds
                .iter()
                .map(|kind| match kind {
                    client::OwnerAuthorityExecutor::Charter(number) => people::Executor::Charter(*number),
                    client::OwnerAuthorityExecutor::Procedure(number) => people::Executor::Procedure(*number),
                    client::OwnerAuthorityExecutor::Role(number) => people::Executor::Role(*number),
                })
                .collect(),
            tasks: value.delegation.tasks,
            depth: value.delegation.depth,
        },
        spend: value.spend,
        deadline: value.deadline,
        notes: value.notes,
        note_resources: value
            .note_resources
            .iter()
            .map(|scope| people::ResourceScope { connector: scope.connector, pattern: pattern(&scope.pattern) })
            .collect(),
    }
}

fn note_scope(value: &client::NoteScope, project: u32) -> Result<people::NoteScope, Invalid> {
    Ok(match value {
        client::NoteScope::Deployment => people::NoteScope::Deployment,
        client::NoteScope::Project(named) => {
            if *named != project {
                return Err(Invalid::NoteProject);
            }
            people::NoteScope::Project
        }
        client::NoteScope::Task { project: named, task: goal } => {
            if *named != project {
                return Err(Invalid::NoteProject);
            }
            people::NoteScope::Goal { goal: *goal }
        }
        client::NoteScope::Resource { project: named, connector, pattern: value } => {
            if *named != project {
                return Err(Invalid::NoteProject);
            }
            people::NoteScope::Resources { connector: *connector, pattern: pattern(value) }
        }
    })
}

fn proposal_decision(value: &client::Decision) -> Result<people::ProposalDecision, Invalid> {
    match value {
        client::Decision::Accepted => Ok(people::ProposalDecision::Accept),
        client::Decision::Rejected { reason } => {
            Ok(people::ProposalDecision::Reject { reason: Box::from(reason.bytes()) })
        }
        client::Decision::PassedUp => Ok(people::ProposalDecision::Pass),
        client::Decision::Released { .. } | client::Decision::LeftHeld { .. } => Err(Invalid::Decision),
    }
}

fn escalation_decision(value: &client::Decision) -> Result<people::EscalationDecision, Invalid> {
    match value {
        client::Decision::Released { again: None } => Ok(people::EscalationDecision::Release),
        client::Decision::LeftHeld { reason } | client::Decision::Rejected { reason } => {
            Ok(people::EscalationDecision::Reject { reason: Box::from(reason.bytes()) })
        }
        client::Decision::PassedUp => Ok(people::EscalationDecision::Pass),
        client::Decision::Accepted | client::Decision::Released { again: Some(_) } => Err(Invalid::Decision),
    }
}

#[cfg(test)]
mod tests {
    const UTF8_TEXTS: [&[u8]; 3] = [b"~/\0", b"~\0", b"~/"];

    use super::*;
    fn text(value: &[u8]) -> client::Text {
        client::Text::new(Box::from(value), 128).expect("the complete supplied fixture is admitted")
    }

    #[test]
    fn amendment_requests_keep_supplied_empty_or_opaque_reason_and_independent_address_spec_and_release_revisions() {
        for reason in [text(b""), text(UTF8_TEXTS[0])] {
            for address in [client::PersonAddress::Party(u64::MAX), client::PersonAddress::Role(u32::MAX)] {
                let request = client::Ask::Amend {
                    project: u32::MAX,
                    task: 0,
                    revision: Some(client::Revision { value: 6 }),
                    change: Box::new(client::Amendment {
                        llm: None,
                        addressed: Some(client::AddressChange { revision: client::Revision { value: 12 }, address }),
                        instructions: Some(text(b"")),
                        spec: None,
                        wake: None,
                        dependencies: Some(List::with_capacity(7)),
                        authority: None,
                        tracked: None,
                    }),
                    reason: reason.clone(),
                };
                let client::Ask::Amend { project, task, revision, change, reason: supplied_reason } = request else {
                    panic!("amendment");
                };
                let mut operations = List::with_capacity(2);
                operations
                    .push(client::Operation::Amend { project, task, revision, change, reason: supplied_reason })
                    .expect("two operations fit");
                operations
                    .push(client::Operation::Release {
                        project,
                        task,
                        revision: client::Revision { value: 4 },
                        again: None,
                    })
                    .expect("two operations fit");
                let request = client::Ask::Compound { project, operations };
                let before = request.clone();
                let Ok(people::Ask::Compound { project, operations }) = ask(&request) else {
                    panic!("whole compound");
                };
                let [
                    people::Operation::Amend { task, revision, amendment, release: None },
                    people::Operation::Release { revision: release, .. },
                ] = operations.as_slice()
                else {
                    panic!("amend and release");
                };
                let amendment = amendment.clone();
                assert_eq!((project, *task, *revision, Some(*release)), (u32::MAX, 0, 6, Some(4)));
                let addressed = amendment.addressed.expect("selected address");
                assert_eq!(addressed.revision, 12);
                assert_eq!(
                    addressed.address,
                    match address {
                        client::PersonAddress::Party(n) => people::PersonAddress::Person(n),
                        client::PersonAddress::Role(n) => people::PersonAddress::Role(n),
                    }
                );
                assert_eq!(amendment.reason.as_ref(), reason.bytes());
                assert_eq!(amendment.instructions.as_deref(), Some([].as_slice()));
                assert_eq!(amendment.dependencies.as_deref(), Some([].as_slice()));
                assert_eq!((amendment.spec, amendment.wake, amendment.authority), (None, None, None));
                assert_eq!(request, before);
            }
        }
    }

    #[test]
    fn amendment_whole_selected_specification_authority_and_wake_keep_every_parameter_channel_and_optional_absence() {
        let mut parameters = List::with_capacity(7);
        parameters.push(client::Parameter::Number { name: u32::MAX, value: u64::MAX }).expect("room");
        parameters.push(client::Parameter::Bytes { name: 0, value: text(UTF8_TEXTS[1]) }).expect("room");
        parameters.push(client::Parameter::Resource { name: 3, connector: u16::MAX, resource: 0 }).expect("room");
        let mut inputs = List::with_capacity(5);
        inputs.push(u64::MAX).expect("room");
        inputs.push(0).expect("room");
        let selected = client::Specification { words: text(b""), parameters, inputs: inputs.clone() };
        let source = super::super::owner::tests::amendment_source();
        let limits = jig_client::DomainLimits { rows: 4, text: 128, ..crate::test::limits().client.domain };
        let authority = super::super::owner::authority(&source.authority, &limits).expect("whole authority").authority;
        let wake = client::OwnerWakePolicy {
            words: client::WakeRule::Batch { count: u32::MAX, age: skein_lib::Duration::ZERO },
            notices: client::WakeRule::Immediate,
            news: client::WakeRule::Never,
            results: client::ResultsWake::LastOrFailure,
            questions: true,
            answers: false,
            timers: true,
        };
        let request = client::Ask::Amend {
            project: 0,
            task: u64::MAX,
            revision: Some(client::Revision { value: u64::MAX }),
            change: Box::new(client::Amendment {
                llm: None,
                addressed: None,
                instructions: None,
                spec: Some(selected),
                wake: Some(wake),
                dependencies: Some(inputs),
                authority: Some(authority.clone()),
                tracked: None,
            }),
            reason: text(b"supplied"),
        };
        let Ok(people::Ask::Amend { project, task, revision, amendment, release }) = ask(&request) else {
            panic!("whole amendment")
        };
        assert_eq!((project, task, revision, release), (0, u64::MAX, u64::MAX, None));
        assert_eq!((amendment.addressed, amendment.instructions), (None, None));
        let spec = amendment.spec.expect("selected whole spec");
        assert_eq!(spec.words.as_ref(), b"");
        assert_eq!(spec.inputs.as_ref(), &[u64::MAX, 0]);
        assert_eq!(
            spec.parameters.as_ref(),
            &[
                people::Parameter::Number { name: u32::MAX, value: u64::MAX },
                people::Parameter::Bytes { name: 0, value: Box::from(UTF8_TEXTS[1]) },
                people::Parameter::Resource { name: 3, connector: u16::MAX, resource: 0 },
            ]
        );
        assert_eq!(amendment.dependencies.as_deref(), Some([u64::MAX, 0].as_slice()));
        assert_eq!(amendment.authority, Some(super::authority(&authority)));
        assert_eq!(
            amendment.wake,
            Some(people::WakePolicy {
                words: people::WakeRule::Batch { count: u32::MAX, age: skein_lib::Duration::ZERO },
                notices: people::WakeRule::Immediate,
                news: people::WakeRule::Never,
                results: people::ResultsWake::LastOrFailure,
                questions: true,
                answers: false,
                timers: true,
            })
        );
        assert_eq!(amendment.reason.as_ref(), b"supplied");
    }

    #[test]
    fn request_translation_keeps_named_question_person_hold_and_proposal_identities_and_empty_words() {
        assert_eq!(
            ask(&client::Ask::Answer {
                project: u32::MAX,
                on: client::AnswerOn::Question { task: u64::MAX, message: 42, words: text(b"") }
            }),
            Ok(people::Ask::AnswerQuestion { project: u32::MAX, task: u64::MAX, question: 42, words: Box::new([]) })
        );
        for again in [None, Some(false), Some(true)] {
            assert_eq!(
                ask(&client::Ask::Release {
                    project: 4,
                    task: 99,
                    revision: client::Revision { value: u64::MAX },
                    again
                }),
                Ok(people::Ask::Release { project: 4, task: 99, revision: u64::MAX, again })
            );
        }
        for proposer in [client::Proposer::Task(u64::MAX), client::Proposer::Party(u64::MAX)] {
            let owner = match proposer {
                client::Proposer::Task(n) => people::Proposer::Task(n),
                client::Proposer::Party(n) => people::Proposer::Party(n),
            };
            assert_eq!(
                ask(&client::Ask::Decide {
                    project: 4,
                    on: client::DecideOn::Proposal {
                        proposal: client::ProposalKey { proposer, number: 7 },
                        decision: client::Decision::Rejected { reason: text(UTF8_TEXTS[0]) }
                    }
                }),
                Ok(people::Ask::DecideProposal {
                    project: 4,
                    proposer: owner,
                    proposal: 7,
                    decision: people::ProposalDecision::Reject { reason: Box::from(UTF8_TEXTS[0]) }
                })
            );
        }
    }

    #[test]
    fn note_request_translation_checks_recalled_width_and_supplied_project_without_dropping_references() {
        let mut references = List::with_capacity(5);
        references.push(u64::MAX).expect("the complete supplied fixture is admitted");
        references.push(13).expect("the complete supplied fixture is admitted");
        let value = client::Ask::EditNote {
            project: 4,
            note: 99,
            scope: client::NoteScope::Project(4),
            revision: client::Revision { value: u64::from(u32::MAX) },
            description: text(b""),
            body: text(UTF8_TEXTS[2]),
            references,
        };
        assert_eq!(
            ask(&value),
            Ok(people::Ask::EditNote {
                project: 4,
                name: 99,
                scope: Box::new(people::NoteScope::Project),
                change: Box::new(people::NoteChange::Correct {
                    description: Box::new([]),
                    body: Box::from(UTF8_TEXTS[2]),
                    references: Box::from([u64::MAX, 13]),
                    recalled: u32::MAX
                })
            })
        );
        assert_eq!(
            ask(&client::Ask::DeleteNote {
                project: 4,
                note: 99,
                scope: client::NoteScope::Project(5),
                revision: client::Revision { value: 1 }
            }),
            Err(Invalid::NoteProject)
        );
        assert_eq!(
            ask(&client::Ask::DeleteNote {
                project: 4,
                note: 99,
                scope: client::NoteScope::Task { project: 4, task: u64::MAX },
                revision: client::Revision { value: u64::from(u32::MAX) + 1 }
            }),
            Err(Invalid::NoteRevision)
        );
    }

    #[test]
    fn decision_translation_refuses_an_inapplicable_kind_or_uncertain_choice_instead_of_reinterpreting_the_ask() {
        for again in [Some(false), Some(true)] {
            assert_eq!(escalation_decision(&client::Decision::Released { again }), Err(Invalid::Decision));
        }
        assert_eq!(
            escalation_decision(&client::Decision::Released { again: None }),
            Ok(people::EscalationDecision::Release)
        );
        assert_eq!(proposal_decision(&client::Decision::Released { again: None }), Err(Invalid::Decision));
        assert_eq!(proposal_decision(&client::Decision::LeftHeld { reason: text(b"") }), Err(Invalid::Decision));
        assert_eq!(escalation_decision(&client::Decision::Accepted), Err(Invalid::Decision));
        assert_eq!(
            escalation_decision(&client::Decision::LeftHeld { reason: text(b"") }),
            Ok(people::EscalationDecision::Reject { reason: Box::new([]) })
        );
    }

    fn literal_pattern(open: bool) -> client::OwnerPattern {
        let mut segments = List::with_capacity(4);
        segments.push(Box::from(&b"one/two"[..])).expect("the complete supplied fixture is admitted");
        segments.push(Box::from(&[255, 0][..])).expect("the complete supplied fixture is admitted");
        client::OwnerPattern {
            segments,
            last: if open {
                client::OwnerPatternEnd::Open(Box::new([]))
            } else {
                client::OwnerPatternEnd::Exact(Box::from(&[b'/', 254][..]))
            },
        }
    }

    #[test]
    fn policy_request_translation_keeps_literal_patterns_guard_judge_and_full_width_permission_fields() {
        for open in [false, true] {
            let expected = people::Pattern {
                segments: Box::from([Box::from(&b"one/two"[..]), Box::from(&[255, 0][..])]),
                last: if open {
                    people::Last::Open(Box::new([]))
                } else {
                    people::Last::Exact(Box::from(&[b'/', 254][..]))
                },
            };
            let mut requirements = List::with_capacity(5);
            for guard in [
                client::PolicyGuard::Guarded,
                client::PolicyGuard::Observed { freshness: skein_lib::Duration::from_nanos(u64::MAX) },
            ] {
                requirements
                    .push(client::PolicyRequirement {
                        connector: u16::MAX,
                        kind: 7,
                        pattern: literal_pattern(open),
                        judge: client::PolicyJudge { connector: 3, requirement: u16::MAX, parameters: u32::MAX },
                        guard,
                        must_be_guarded: true,
                    })
                    .expect("the complete supplied fixture is admitted");
            }
            let Ok(people::Ask::ChangePolicy { project, change: people::PolicyChange::Requirements { requirements } }) =
                ask(&client::Ask::ChangePolicy {
                    project: u32::MAX,
                    edit: client::PolicyEdit::Requirements { requirements },
                })
            else {
                panic!("one whole policy request");
            };
            assert_eq!(project, u32::MAX);
            assert_eq!(requirements.len(), 2);
            for value in &requirements {
                assert_eq!(value.pattern, expected);
                assert_eq!((value.connector, value.kind), (u16::MAX, 7));
                assert_eq!(value.judge, people::Judge { connector: 3, requirement: u16::MAX, parameters: u32::MAX });
                assert!(value.must_be_guarded);
            }
            assert_eq!(requirements[0].guard, people::Guard::Guarded);
            assert_eq!(
                requirements[1].guard,
                people::Guard::Observed { freshness: skein_lib::Duration::from_nanos(u64::MAX) }
            );
            let mut mappings = List::with_capacity(3);
            mappings
                .push(client::PolicyPermission { connector: u16::MAX, permission: u16::MAX, role: u32::MAX })
                .expect("the complete supplied fixture is admitted");
            assert_eq!(
                ask(&client::Ask::ChangePolicy { project: 4, edit: client::PolicyEdit::Permissions { mappings } }),
                Ok(people::Ask::ChangePolicy {
                    project: 4,
                    change: people::PolicyChange::Permissions {
                        mappings: Box::from([people::PermissionRole {
                            connector: u16::MAX,
                            permission: u16::MAX,
                            role: u32::MAX
                        }])
                    }
                })
            );
        }
    }

    #[test]
    fn policy_role_and_whole_replacements_keep_opaque_authority_codes_flags_and_explicit_empty_lists() {
        let mut kinds = List::with_capacity(5);
        kinds
            .push(client::OwnerAuthorityExecutor::Charter(u32::MAX))
            .expect("the complete supplied fixture is admitted");
        kinds
            .push(client::OwnerAuthorityExecutor::Procedure(u32::MAX))
            .expect("the complete supplied fixture is admitted");
        kinds.push(client::OwnerAuthorityExecutor::Role(u32::MAX)).expect("the complete supplied fixture is admitted");
        let grant = client::OwnerGrant { connector: u16::MAX, kind: u16::MAX, pattern: literal_pattern(true) };
        let mut grants = List::with_capacity(3);
        grants.push(grant.clone()).expect("the complete supplied fixture is admitted");
        let mut resources = List::with_capacity(3);
        resources
            .push(client::OwnerResourceScope { connector: u16::MAX, pattern: literal_pattern(false) })
            .expect("the complete supplied fixture is admitted");
        let role = client::PolicyEditRole {
            number: u32::MAX,
            authority: client::OwnerAuthority {
                tools: u64::MAX,
                grants: grants.clone(),
                delegation: client::OwnerDelegation { kinds, tasks: u32::MAX, depth: u32::MAX },
                spend: u64::MAX,
                deadline: None,
                notes: u8::MAX,
                note_resources: resources,
            },
            period_spend: u64::MAX,
            requests: u16::MAX,
            decides: u8::MAX,
        };
        let Ok(people::Ask::ChangePolicy { project: 4, change: people::PolicyChange::Role(now) }) =
            ask(&client::Ask::ChangePolicy { project: 4, edit: client::PolicyEdit::Role(role) })
        else {
            panic!("closed role replacement");
        };
        assert_eq!((now.number, now.period_spend, now.requests, now.decides), (u32::MAX, u64::MAX, u16::MAX, u8::MAX));
        assert_eq!((now.authority.tools, now.authority.spend, now.authority.notes), (u64::MAX, u64::MAX, u8::MAX));
        assert_eq!(now.authority.deadline, None);
        assert_eq!((now.authority.delegation.tasks, now.authority.delegation.depth), (u32::MAX, u32::MAX));
        assert_eq!(
            &*now.authority.delegation.kinds,
            &[
                people::Executor::Charter(u32::MAX),
                people::Executor::Procedure(u32::MAX),
                people::Executor::Role(u32::MAX)
            ]
        );
        assert_eq!(now.authority.grants[0], super::grant(&grant));
        assert_eq!(now.authority.note_resources[0].pattern, pattern(&literal_pattern(false)));
        assert_eq!(
            ask(&client::Ask::ChangePolicy { project: 4, edit: client::PolicyEdit::Projections { grants } }),
            Ok(people::Ask::ChangePolicy {
                project: 4,
                change: people::PolicyChange::Projections { grants: Box::from([super::grant(&grant)]) }
            })
        );
        assert_eq!(
            ask(&client::Ask::ChangePolicy { project: 4, edit: client::PolicyEdit::ProjectSpend { period_spend: 0 } }),
            Ok(people::Ask::ChangePolicy {
                project: 4,
                change: people::PolicyChange::ProjectSpend { period_spend: 0 }
            })
        );
        for edit in [
            client::PolicyEdit::Requirements { requirements: List::with_capacity(5) },
            client::PolicyEdit::Projections { grants: List::with_capacity(5) },
            client::PolicyEdit::Permissions { mappings: List::with_capacity(5) },
        ] {
            let Ok(people::Ask::ChangePolicy { change, .. }) = ask(&client::Ask::ChangePolicy { project: 4, edit })
            else {
                panic!("explicit empty replacement");
            };
            match change {
                people::PolicyChange::Requirements { requirements } => assert!(requirements.is_empty()),
                people::PolicyChange::Projections { grants } => assert!(grants.is_empty()),
                people::PolicyChange::Permissions { mappings } => assert!(mappings.is_empty()),
                people::PolicyChange::Role(_)
                | people::PolicyChange::ProjectSpend { .. }
                | people::PolicyChange::Templates { .. } => {
                    panic!("replacement kind retained")
                }
            }
        }
    }

    #[test]
    fn role_requests_preserve_all_builtins_full_policy_width_and_whole_named_holdings() {
        let cases = [
            (client::HeldRole::Owner, people::Role::Owner),
            (client::HeldRole::Maintainer, people::Role::Maintainer),
            (client::HeldRole::Member, people::Role::Member),
            (client::HeldRole::Observer, people::Role::Observer),
            (client::HeldRole::Policy { role: u32::MAX }, people::Role::Policy { role: u32::MAX }),
        ];
        let mut holdings = List::with_capacity(7);
        let mut expected = Vec::new();
        for (index, (supplied, actual)) in cases.into_iter().enumerate() {
            let party = u64::MAX - u64::try_from(index).expect("the complete supplied fixture is admitted");
            holdings
                .push(client::Holding { party, role: supplied })
                .expect("the complete supplied fixture is admitted");
            expected.push(people::Holding { person: party, role: actual });
            assert_eq!(
                ask(&client::Ask::MakeService { project: u32::MAX, name: text(UTF8_TEXTS[0]), role: supplied }),
                Ok(people::Ask::MakeService { project: u32::MAX, name: Box::from(UTF8_TEXTS[0]), role: actual })
            );
        }
        assert_eq!(
            ask(&client::Ask::SetRoles { project: u32::MAX, holdings }),
            Ok(people::Ask::SetRoles { project: u32::MAX, holdings: expected.into_boxed_slice() })
        );
        assert_eq!(
            ask(&client::Ask::SetRoles { project: 0, holdings: List::with_capacity(7) }),
            Ok(people::Ask::SetRoles { project: 0, holdings: Box::new([]) })
        );
    }
    #[test]
    fn note_mutation_scope_keeps_explicit_goal_project_and_whole_literal_pattern_without_scope_inference() {
        for open in [false, true] {
            let pattern = literal_pattern(open);
            let scope =
                client::NoteScope::Resource { project: u32::MAX, connector: u16::MAX, pattern: pattern.clone() };
            let request = client::Ask::DeleteNote {
                project: u32::MAX,
                note: u64::MAX,
                scope: scope.clone(),
                revision: client::Revision { value: u64::from(u32::MAX) },
            };
            let before = request.clone();
            assert_eq!(
                ask(&request),
                Ok(people::Ask::EditNote {
                    project: u32::MAX,
                    name: u64::MAX,
                    scope: Box::new(people::NoteScope::Resources {
                        connector: u16::MAX,
                        pattern: super::pattern(&pattern)
                    }),
                    change: Box::new(people::NoteChange::Delete { recalled: u32::MAX }),
                })
            );
            assert_eq!(request, before);
            assert_eq!(note_scope(&scope, 0), Err(Invalid::NoteProject));
        }
        let goal = client::NoteScope::Task { project: u32::MAX, task: u64::MAX };
        assert_eq!(note_scope(&goal, u32::MAX), Ok(people::NoteScope::Goal { goal: u64::MAX }));
        assert_eq!(note_scope(&goal, 0), Err(Invalid::NoteProject));
    }
}
