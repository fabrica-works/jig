//! Assemble outputs in the decision that owns them. A copying application
//! replaces its tools and connector results; charter conversion stays shared
//! (domain/root.md, 7; domain/hosts.md, 8).
use crate::domain::Work;
use crate::{Domain, Limits};
use alloc::boxed::Box;
use jig_domain_accounts as accounts;
use jig_domain_brief as brief;
use jig_domain_core as core;
use jig_domain_host as host;
use jig_domain_tasks as tasks;
use jig_ops_domain_connectors::Connector;
use jig_protocol_charter as charter;
use skein_lib::{Decimal, Env, List, Queue, ReplyTo, Token, Writer};

pub(crate) fn brief(order: Box<[brief::GatherPlaced]>) -> Box<[charter::Section]> {
    let mut sections = List::with_capacity(u32::try_from(order.len()).expect("bounded sections"));
    for placed in order {
        let section = match placed {
            brief::GatherPlaced::Core { kind, text } => charter::Section { title: Box::from(title(kind)), text },
            brief::GatherPlaced::CoreMissing { kind, .. } => {
                charter::Section { title: Box::from(title(kind)), text: Box::from(&b"[missing]"[..]) }
            }
            brief::GatherPlaced::Missing { .. } => {
                charter::Section { title: Box::from(&b"Context"[..]), text: Box::from(&b"[missing]"[..]) }
            }
            brief::GatherPlaced::Connector { .. } => unreachable!("ops offers no connector brief sections"),
        };
        sections.push(section).expect("one section per placement");
    }
    sections.into_boxed()
}
const fn title(kind: brief::Core) -> &'static [u8] {
    match kind {
        brief::Core::Task => b"Task",
        brief::Core::Lineage => b"Requesters",
        brief::Core::Results => b"Results",
        brief::Core::Plan => b"Plan",
        brief::Core::Attempts => b"Attempts",
        brief::Core::Calls => b"Calls",
        brief::Core::Waiting => b"Waiting",
        brief::Core::NotesIndex => b"Notes",
        brief::Core::TranscriptTail => b"Conversation",
    }
}
#[expect(clippy::too_many_arguments, reason = "assemble the owners parts without a shared sibling type")]
pub(crate) fn assignment(
    domain: &Domain,
    env: &Env<Limits>,
    task: u64,
    attempt: u64,
    conversation: u64,
    run: core::RunCharter,
    sections: Box<[charter::Section]>,
    transcript: Box<[Box<[u8]>]>,
    messages: Box<[tasks::Word]>,
    grant: accounts::Grant,
) -> Option<host::Assignment> {
    let policy = run.policy;
    let mut models = List::with_capacity(u32::try_from(policy.alternatives.len()).ok()?);
    for value in policy.alternatives {
        models.push(model(value)).ok()?;
    }
    let value = charter::charter(
        charter::Charter {
            instructions: policy.instructions,
            tools: domain.tools.clone(),
            wait: true,
            agents: policy.agents,
            workspace: charter::WorkspaceTools { inspect: false, modify: false, shell: false },
            conventions: None,
            contract: contract(run.contract)?,
            budget: charter::Budget {
                turns: policy.turns,
                spend: run.budget,
                time: policy.time,
                reserve: charter::Reserve { turns: policy.reserve_turns, time: policy.reserve_time },
            },
            model: model(policy.model),
            models: models.into_boxed(),
            waiting: policy.waiting,
            resumes: policy.resume,
        },
        sections,
        !transcript.is_empty(),
    )?;
    let encoded = charter::encode(value, &domain.charter_endpoints, &env.limits.charter)?;
    let mut answered = List::with_capacity(env.limits.core.call_records);
    for value in domain.core.settled_calls(task, attempt) {
        let answer = match value.answer {
            core::SettledAnswer::Host { error, body } => host::SettledAnswer::Host { error, body },
            core::SettledAnswer::Delivery { outcome, evidence } => {
                host::SettledAnswer::Delivery { outcome: delivery(outcome), evidence }
            }
        };
        answered.push(host::AnsweredCall { name: value.name, tool: value.tool, answer }).ok()?;
    }
    let mut start_messages = List::with_capacity(env.limits.core.tasks.run_inbox);
    for word in messages {
        start_messages
            .push(host::StartMessage {
                name: Token::new(word.number),
                sender: sender(word.from),
                words: inbox_words(&word),
            })
            .ok()?;
    }
    Some(host::Assignment {
        messages: start_messages.into_boxed(),
        assignment: host::RunAssignment {
            identity: charter::run_identity(&domain.deployment, task, conversation),
            run: Token::new(task),
            attempt: Token::new(attempt),
            workspace: None,
            save: false,
            charter: encoded,
            grants: Box::new([host::Grant {
                table: jig_domain_host::GrantTable::Accounts,
                account: grant.account,
                generation: grant.generation,
                valid: grant.valid,
            }]),
        },
        turns: transcript,
        answered: answered.into_boxed(),
    })
}
fn model(value: core::Model) -> charter::Model {
    charter::Model {
        prices: charter::Prices {
            input: value.input_price,
            cached: value.cached_price,
            cache_write: value.cache_write_price,
            output: value.output_price,
            unit: value.price_unit,
        },
        dialect: value.dialect,
        account: value.account,
        endpoint: value.endpoint,
        name: value.name,
        window: value.window,
        output: value.output,
    }
}
fn contract(value: tasks::Contract) -> Option<charter::Contract> {
    Some(match value {
        tasks::Contract::Approval { .. } => return None,
        tasks::Contract::Report { words } => charter::Contract {
            report: Some(charter::TextRule { max: words, fields: Box::new([]) }),
            failure: Some(charter::TextRule { max: words, fields: Box::new([]) }),
            verdicts: Box::new([]),
            change: None,
        },
        tasks::Contract::Verdict { choices } => {
            let mut verdicts = List::with_capacity(u32::try_from(choices.len()).expect("verdict choices"));
            for choice in choices {
                let mut fields = List::with_capacity(u32::try_from(choice.fields.len()).ok()?);
                for field in choice.fields {
                    fields.push(charter::FieldRule { name: field.name, max: field.max }).ok()?;
                }
                verdicts
                    .push(charter::VerdictRule {
                        name: Box::from(Decimal::of(u64::from(choice.code)).as_bytes()),
                        text_max: choice.words,
                        fields: fields.into_boxed(),
                        items: charter::Items { min: 0, max: 0, kinds: Box::new([]) },
                    })
                    .expect("one verdict rule");
            }
            charter::Contract {
                report: None,
                failure: Some(charter::TextRule { max: 1024, fields: Box::new([]) }),
                verdicts: verdicts.into_boxed(),
                change: None,
            }
        }
        tasks::Contract::Change { words, .. } => charter::Contract {
            report: None,
            failure: Some(charter::TextRule { max: words, fields: Box::new([]) }),
            verdicts: Box::new([]),
            change: Some(charter::ChangeRule {
                checks_must_pass: true,
                fields: Box::new([
                    charter::FieldRule { name: Box::from(&b"title"[..]), max: words },
                    charter::FieldRule { name: Box::from(&b"body"[..]), max: words },
                ]),
            }),
        },
    })
}
const fn delivery(value: core::DeliveryOutcome) -> host::DeliveryOutcome {
    match value {
        core::DeliveryOutcome::Delivered => host::DeliveryOutcome::Delivered,
        core::DeliveryOutcome::Nothing => host::DeliveryOutcome::Nothing,
        core::DeliveryOutcome::Stale => host::DeliveryOutcome::Stale,
        core::DeliveryOutcome::Refused | core::DeliveryOutcome::Invalid => host::DeliveryOutcome::Refused,
        core::DeliveryOutcome::Failed => host::DeliveryOutcome::Failed,
    }
}
/// Preserve the task-owned message kind when translating its opaque words.
pub(crate) fn inbox_words(word: &tasks::Word) -> Box<[u8]> {
    let (label, number) = match word.kind {
        tasks::MessageKind::Result(_) => (
            b"Result from task ".as_slice(),
            match word.from {
                tasks::Party::Task(task) => task,
                tasks::Party::Person(person) => person,
                tasks::Party::Deployment { .. } => 0,
            },
        ),
        tasks::MessageKind::ProposalDecision { proposal, accepted } => {
            (if accepted { b"Proposal accepted ".as_slice() } else { b"Proposal rejected ".as_slice() }, proposal)
        }
        tasks::MessageKind::PersonStalled { task } => (b"Person task past its waiting deadline ".as_slice(), task),
        tasks::MessageKind::EffectNotMade { entry } => (b"Effect not made ".as_slice(), entry),
        tasks::MessageKind::Words
        | tasks::MessageKind::Question
        | tasks::MessageKind::Answer { .. }
        | tasks::MessageKind::Notice { .. }
        | tasks::MessageKind::News { .. }
        | tasks::MessageKind::Timer { .. }
        | tasks::MessageKind::Amendment { .. }
        | tasks::MessageKind::Proposal { .. }
        | tasks::MessageKind::Escalation { .. } => return word.words.clone(),
    };
    let number = Decimal::of(number);
    let bytes = label.len().checked_add(number.as_bytes().len()).expect("bounded message label");
    let bytes = bytes.checked_add(2).expect("bounded message separator");
    let bytes = bytes.checked_add(word.words.len()).expect("bounded message envelope");
    let mut writer = Writer::new(bytes);
    writer.put(label).expect("measured label");
    writer.put(number.as_bytes()).expect("measured source");
    writer.put(b":\n").expect("measured separator");
    writer.put(&word.words).expect("measured words");
    writer.finish()
}

pub(crate) fn sender(value: tasks::Party) -> Box<[u8]> {
    match value {
        tasks::Party::Task(number) | tasks::Party::Person(number) => Box::from(Decimal::of(number).as_bytes()),
        tasks::Party::Deployment { .. } => Box::from(&b"deployment"[..]),
    }
}
pub(crate) fn host_answer(value: core::SettledAnswer) -> host::Relayed {
    match value {
        core::SettledAnswer::Host { error, body } => host::Relayed::Answer { error, text: body },
        core::SettledAnswer::Delivery { .. } => unreachable!("engine charter has no delivery tool"),
    }
}

pub(crate) fn ending(value: host::Ending) -> tasks::End {
    match value {
        host::Ending::Refused(_) => tasks::End::Refused,
        host::Ending::Parked { .. } => tasks::End::Parked,
        host::Ending::Failed { failure, .. } => tasks::End::Failed(match failure {
            host::Failure::Unprepared(host::Preparation::Transient) => tasks::Class::Transient,
            host::Failure::Unprepared(host::Preparation::Permanent { .. }) => tasks::Class::Permanent,
            host::Failure::Run(_) => tasks::Class::Run,
            host::Failure::Agent(_) | host::Failure::Cancelled(_) => tasks::Class::Agent,
        }),
        host::Ending::Ended { .. } => unreachable!("ended results leave for protocol reading"),
    }
}

pub(crate) fn answer_payload(domain: &mut Domain, run: Token, attempt: Token, payload: Token, work: &mut Queue<Work>) {
    let ending = match domain.payloads.get(&payload).expect("fenced answer payload") {
        crate::boundary::Payload::Answer { task, attempt: retained, ending, .. } => {
            assert!(*task == run.raw() && *retained == attempt.raw(), "fenced answer identity");
            ending.clone()
        }
        crate::boundary::Payload::Turn { .. }
        | crate::boundary::Payload::Word(_)
        | crate::boundary::Payload::Settled(_) => unreachable!("answer family"),
    };
    match ending {
        host::Ending::Ended { outcome, .. } => {
            crate::domain::now(domain, crate::Output::ReadResult { payload, outcome });
        }
        other @ (host::Ending::Refused(_) | host::Ending::Parked { .. } | host::Ending::Failed { .. }) => {
            result_read(domain, payload, self::ending(other), work);
        }
    }
}

pub(crate) fn result_read(domain: &Domain, payload: Token, end: tasks::End, work: &mut Queue<Work>) {
    let Some(crate::boundary::Payload::Answer { task, attempt, spent, read, refused, .. }) =
        domain.payloads.get(&payload)
    else {
        return;
    };
    work.push(Work::Core(core::Event::AnswerPayload {
        run: Token::new(*task),
        attempt: Token::new(*attempt),
        payload,
        cumulative: *spent,
        end,
        saved: None,
        invalid_saved: false,
        read: *read,
        refused: refusal_names(refused),
    }));
}

/// Take the owner's answer once, preserving the core's held or now mark.
pub(crate) fn relayed(
    domain: &mut Domain,
    run: Token,
    attempt: Token,
    call: Box<[u8]>,
    answer: Token,
) -> Option<crate::Released> {
    let value = match domain.payloads.remove(&answer).expect("settled call token") {
        crate::boundary::Payload::Settled(value) => value,
        crate::boundary::Payload::Turn { .. }
        | crate::boundary::Payload::Answer { .. }
        | crate::boundary::Payload::Word(_) => unreachable!("settled call family"),
    };
    let binding = domain.host_calls.remove(&call)?;
    assert!(binding.run == run && binding.attempt == attempt, "hub relay fence");
    Some(crate::Released::Host(host::Event::Down(host::LinkDown::Relayed {
        run,
        attempt,
        call: binding.delivery,
        answer: host_answer(value.answer),
    })))
}

pub(crate) fn read_answer(
    domain: &mut Domain,
    call: crate::domain::ReadCall,
    error: bool,
    body: Box<[u8]>,
    work: &mut Queue<Work>,
) {
    let answer = domain.payload(crate::boundary::Payload::Settled(core::SettledCall {
        serial: 0,
        name: call.name,
        tool: call.tool,
        answer: core::SettledAnswer::Host { error, body },
    }));
    work.push(Work::Core(core::Event::Fleet(jig_domain_fleet::Event::Relayed { to: ReplyTo::new(call.to), answer })));
}

pub(crate) use crate::message_refusals::{message_read, refusal_names, refusals};

#[expect(clippy::too_many_arguments, reason = "route the owners read and authenticated caller envelope")]
pub(crate) fn connector_read(
    domain: &mut Domain,
    env: &Env<Limits>,
    to: ReplyTo,
    task: u64,
    name: Box<[u8]>,
    tool: Box<[u8]>,
    read: jig_ops_domain_observability::Read,
    work: &mut Queue<Work>,
) {
    let binding = crate::domain::ReadCall { to: to.into_token(), name, tool };
    let number = jig_ops_domain_connectors::number(Connector::Observability);
    let description = crate::translate::read_description(number, &read);
    let mut findings =
        Queue::with_capacity(jig_domain_authority::max_out(&env.limits.core.authority).expect("findings"));
    if domain.core.connector_read_admit(task, description, &mut findings) != jig_domain_authority::Answer::Allow
        || domain.read_calls.len() == domain.read_calls.capacity()
    {
        read_answer(domain, binding, true, Box::from(&b"refused"[..]), work);
    } else {
        let token = domain.token();
        domain.read_calls.insert(token, binding).expect("admitted read continuation");
        work.push(Work::Observability(jig_ops_domain_observability::Event::Read { token, read }));
    }
}
