//! The runs' agents, scripted for the alert stories at the hub's agent
//! capability (domain/hosts.md, section 6). Ops runs no agent in its engine,
//! so the world answers the hub as an agent host would: a run's script is
//! cued by the earliest of the stories' cues in its charter (a triage's batch
//! of alerts is noise unless it holds alert 12, the incident), and each
//! completion plays the script's step for the completions its conversation
//! has had, resumed turns included. A run that has delegated, and the
//! incident lead or the environment chat once a result has reached it, takes
//! its own steps instead.
//!
//! A run's first completion waits until the hub has taken its start, so the
//! messages the hub held for it have been passed on, as they reached smith's
//! inline agent before its first model answer. Later completions follow the
//! answer or the message they wait for.
//!
//! A resumed run whose last turn finished, with its answer lost, finishes
//! again with the same result, as smith accepts an unanswered finish it
//! finds in the transcript.
//!
//! Each completion streams as it goes, for the run's followers: its start,
//! then the text it yields with or the call it makes.
//!
//! Each completion reads the messages that came since the last, tells one
//! turn (a smith transcript turn, for the client to decode), and then calls a
//! host tool and waits for its answer, yields, or finishes. A run that yields
//! parks once the charter's waiting time passes without a message. A stop
//! ends a run that has not finished as cancelled, and the agent goes.
//! Messages that come once its run has finished, before the stop, are not
//! read: smith's inline agent heard the stop in the same step.
use jig_conformance::{Clock, Input};
use jig_domain_host as host;
use jig_ops_domain as root;
use skein_lib::{Duration, List, Token, Writer};
use smith_transcript::v3 as transcript;
use std::collections::BTreeMap;

pub struct Agents {
    client_backed: bool,
    environment_chat: bool,
    waiting: Duration,
    call_timeout: Duration,
    live: BTreeMap<Token, Agent>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Role {
    Watch,
    Noise,
    Incident,
    Lead,
    Environment,
    Uncued,
}

enum Step {
    Call { tool: &'static str, input: String },
    Yield,
    Finish(Box<[u8]>),
}

#[expect(clippy::struct_excessive_bools, reason = "independent facts of one scripted run")]
struct Agent {
    role: Role,
    activation: u64,
    /// Completions the conversation has had, resumed ones included.
    said: u32,
    /// Turns told, and spend, in this activation.
    turns: u32,
    spent: u64,
    acted: bool,
    result: bool,
    /// The result of the last resumed turn's finish, accepted again first.
    refinish: Option<Box<[u8]>>,
    unread: Vec<(Token, Box<[u8]>)>,
    pending: Option<Box<[u8]>>,
    ready: bool,
    parks_at: Option<u64>,
    finished: bool,
}

impl Agents {
    pub fn new(client_backed: bool, environment_chat: bool, waiting: Duration, call_timeout: Duration) -> Self {
        Self { client_backed, environment_chat, waiting, call_timeout, live: BTreeMap::new() }
    }

    /// Every agent goes with the engine that hosted it.
    pub fn clear(&mut self) {
        self.live.clear();
    }

    pub fn answer(&mut self, request: host::ToAgent) -> Vec<Input<root::Event, root::Record>> {
        let mut out = Vec::new();
        match request {
            host::ToAgent::Start { owner, start } => {
                let host::Start { charter, activation, turns, .. } = *start;
                let mut agent = Agent {
                    role: self.role(&charter),
                    activation,
                    said: u32::try_from(turns.len()).expect("bounded transcript"),
                    turns: 0,
                    spent: 0,
                    acted: false,
                    result: false,
                    refinish: None,
                    unread: Vec::new(),
                    pending: None,
                    ready: false,
                    parks_at: None,
                    finished: false,
                };
                for turn in &turns {
                    agent.remember(turn);
                }
                self.live.insert(owner, agent);
                out.push(host::FromAgent::Started { owner, agent: owner });
            }
            host::ToAgent::Message { agent: owner, name, words, .. } => {
                // A run that has said how it finishes reads nothing more; the
                // core keeps what it did not read for the next activation.
                if let Some(agent) = self.live.get_mut(&owner)
                    && !agent.finished
                {
                    agent.unread.push((name, words));
                    if agent.parks_at.take().is_some() {
                        agent.ready = true;
                    }
                }
            }
            host::ToAgent::Answer { agent: owner, call, .. } => {
                if let Some(agent) = self.live.get_mut(&owner)
                    && agent.pending.as_deref() == Some(&call[..])
                {
                    agent.pending = None;
                    agent.ready = true;
                }
            }
            host::ToAgent::Cancel { agent: owner } => {
                if let Some(agent) = self.live.remove(&owner) {
                    if !agent.finished {
                        out.push(host::FromAgent::Finished {
                            owner,
                            turns: agent.turns,
                            spent: agent.spent,
                            finish: host::Finish::Failed { failure: host::RunFailure::Cancelled },
                        });
                    }
                    out.push(host::FromAgent::Gone { owner, detail: Box::new([]) });
                }
            }
            host::ToAgent::Acknowledge { .. } | host::ToAgent::Grant { .. } => {}
        }
        out.into_iter().map(|report| Input::Event(root::Event::Agent(report))).collect()
    }

    /// The root is about to take `event`: a start it takes readies the run's
    /// first completion.
    pub fn stepped(&mut self, event: &root::Event) {
        if let root::Event::Agent(host::FromAgent::Started { owner, .. }) = event
            && let Some(agent) = self.live.get_mut(owner)
        {
            agent.ready = true;
        }
    }

    /// Take each ready run's next completion, and park the runs whose wait ran out.
    pub fn poll(&mut self, clock: Clock) -> Vec<Input<root::Event, root::Record>> {
        let mut out = Vec::new();
        let owners: Vec<Token> = self.live.keys().copied().collect();
        for owner in owners {
            let agent = self.live.get_mut(&owner).expect("listed agent");
            if agent.finished {
                continue;
            }
            if agent.parks_at.is_some_and(|at| clock.now >= at) {
                agent.parks_at = None;
                agent.finished = true;
                out.push(root::Event::Agent(host::FromAgent::Finished {
                    owner,
                    turns: agent.turns,
                    spent: agent.spent,
                    finish: host::Finish::Parked,
                }));
            } else if agent.ready
                && let Some(outcome) = agent.refinish.take()
            {
                agent.ready = false;
                agent.finished = true;
                out.push(root::Event::Agent(host::FromAgent::Finished {
                    owner,
                    turns: agent.turns,
                    spent: agent.spent,
                    finish: host::Finish::Ended { outcome },
                }));
            } else if agent.ready && agent.pending.is_none() {
                agent.ready = false;
                let role = agent.role;
                let mut read = None;
                let mut texts = Vec::new();
                for (name, words) in agent.unread.drain(..) {
                    if contains(&words, b"Result from task ") {
                        agent.result = true;
                    }
                    read = Some(name);
                    texts.push(words.get(..64).unwrap_or(&words).to_vec());
                }
                let step = self.step(role, self.live.get(&owner).expect("listed agent"));
                let agent = self.live.get_mut(&owner).expect("listed agent");
                agent.said += 1;
                agent.turns += 1;
                agent.spent += 1;
                for content in stream(owner, agent, &step) {
                    out.push(root::Event::AgentContent { owner, content });
                }
                let body = turn(agent, &texts, &step);
                out.push(root::Event::Agent(host::FromAgent::Turn {
                    owner,
                    turn: host::Turn { turn: agent.turns, spent: agent.spent, read, body },
                }));
                match step {
                    Step::Call { tool, input } => {
                        if tool == "delegate" || (tool == "propose" && matches!(role, Role::Lead | Role::Environment)) {
                            agent.acted = true;
                        }
                        let call = name(agent.activation, agent.turns);
                        agent.pending = Some(call.clone());
                        out.push(root::Event::Agent(host::FromAgent::Called {
                            owner,
                            call,
                            ask: host::Ask::Relay {
                                tool: tool.as_bytes().into(),
                                writes: !tool.starts_with("read_"),
                                input: input.into_bytes().into(),
                                deadline: self.call_timeout,
                            },
                        }));
                    }
                    Step::Yield => {
                        agent.parks_at = Some(clock.now + self.waiting.as_nanos());
                        out.push(root::Event::Agent(host::FromAgent::Yielded { owner }));
                    }
                    Step::Finish(outcome) => {
                        agent.finished = true;
                        out.push(root::Event::Agent(host::FromAgent::Finished {
                            owner,
                            turns: agent.turns,
                            spent: agent.spent,
                            finish: host::Finish::Ended { outcome },
                        }));
                    }
                }
            }
        }
        out.into_iter().map(Input::Event).collect()
    }

    fn role(&self, charter: &[u8]) -> Role {
        let alerts = b"Alerts: ".as_slice();
        let triage = match find(charter, alerts) {
            Some(at) => {
                let batch: Vec<u8> = charter[at + alerts.len()..]
                    .iter()
                    .copied()
                    .take_while(|byte| byte.is_ascii_digit() || *byte == b' ')
                    .collect();
                if batch.split(|byte| *byte == b' ').any(|alert| alert == b"12") { Role::Incident } else { Role::Noise }
            }
            None => Role::Uncued,
        };
        let mut cues = vec![
            (b"@watch_setup".as_slice(), Role::Watch),
            (alerts, triage),
            (b"@incident_lead".as_slice(), Role::Lead),
        ];
        if self.environment_chat {
            cues.push((b"@environment_chat".as_slice(), Role::Environment));
        }
        cues.into_iter()
            .filter_map(|(cue, role)| find(charter, cue).map(|at| (at, role)))
            .min_by_key(|(at, _)| *at)
            .map_or(Role::Uncued, |(_, role)| role)
    }

    fn step(&self, role: Role, agent: &Agent) -> Step {
        let index = agent.said;
        if role == Role::Environment && agent.result {
            return finish("staging/payments is ready for the requested week");
        }
        if role == Role::Lead && agent.result {
            return finish("checkout recovered after the accepted restart");
        }
        if matches!(role, Role::Watch | Role::Lead | Role::Environment) && agent.acted {
            return Step::Yield;
        }
        let steps: Vec<Step> = match role {
            Role::Watch => vec![call("delegate", format!(r#"{{"batch":[{}]}}"#, watch())), Step::Yield],
            Role::Noise => vec![logs(), Step::Finish(verdict(NOISE_CODE, NOISE_TEXT, &[]))],
            Role::Incident => vec![logs(), Step::Finish(verdict(INCIDENT_CODE, INCIDENT_TEXT, INCIDENT_FIELDS))],
            Role::Lead => {
                let remediate = remediate(self.client_backed);
                vec![
                    call(
                        "read_series",
                        r#"{"environment":"production","service":"checkout","from":0,"through":7200,"max_bytes":256}"#
                            .into(),
                    ),
                    if self.client_backed {
                        call("delegate", format!(r#"{{"batch":[{remediate}]}}"#))
                    } else {
                        call(
                            "propose",
                            format!(
                                r#"{{"action":"batch","reason":"restart checkout in production","batch":[{remediate}]}}"#
                            ),
                        )
                    },
                ]
            }
            Role::Environment => vec![call(
                "propose",
                format!(
                    r#"{{"action":"batch","reason":"staging for payments, a week; maximum cost 40","batch":[{}]}}"#,
                    provision()
                ),
            )],
            Role::Uncued => Vec::new(),
        };
        steps.into_iter().nth(usize::try_from(index).expect("small index")).unwrap_or(Step::Yield)
    }
}

impl Agent {
    /// What a resumed turn says: whether the run delegated, or read a result.
    fn remember(&mut self, body: &[u8]) {
        self.refinish = None;
        let Ok(turn) = transcript::Turn::decode(&transcript::CEILINGS, &mut skein_lib::Reader::new(body)) else {
            return;
        };
        for message in turn.messages() {
            for block in message.blocks() {
                match block {
                    transcript::Block::Call(call) => {
                        if call.name() == b"finish" {
                            self.refinish = Some(Box::from(call.input()));
                        }
                        if call.name() == b"delegate"
                            || (call.name() == b"propose" && matches!(self.role, Role::Lead | Role::Environment))
                        {
                            self.acted = true;
                        }
                    }
                    transcript::Block::Text(text) => {
                        if contains(text.text(), b"Result from task ") {
                            self.result = true;
                        }
                    }
                    transcript::Block::Refusal(_) | transcript::Block::Result(_) | transcript::Block::Opaque(_) => {}
                }
            }
        }
    }
}

/// A relayed call's name, as an agent's host renders it: activation,
/// completion and position, sixteen bytes, little-endian.
fn name(activation: u64, completion: u32) -> Box<[u8]> {
    let mut bytes = activation.to_le_bytes().to_vec();
    bytes.extend_from_slice(&completion.to_le_bytes());
    bytes.extend_from_slice(&0_u32.to_le_bytes());
    bytes.into()
}

/// What a completion streams: its start, then its text or its call.
fn stream(owner: Token, agent: &Agent, step: &Step) -> Vec<root::AgentContent> {
    let start = root::AgentContent::Text { turn: agent.turns, retry: 0, offset: 0, text: Box::new([]) };
    let call = |tool: &str, input: &[u8]| {
        root::AgentContent::Call(jig_domain_views::Call {
            identity: jig_domain_views::CallIdentity {
                owner,
                sequence: agent.turns,
                retry: 0,
                id: format!("call-{}", agent.said).into_bytes().into(),
            },
            tool: tool.as_bytes().into(),
            input: input.into(),
            cut: false,
            began: None,
        })
    };
    let said = match step {
        Step::Call { tool, input } => call(tool, input.as_bytes()),
        Step::Yield => root::AgentContent::Text {
            turn: agent.turns,
            retry: 0,
            offset: 0,
            text: b"waiting for the delegated result".as_slice().into(),
        },
        Step::Finish(outcome) => call("finish", outcome),
    };
    vec![start, said]
}

fn turn(agent: &Agent, texts: &[Vec<u8>], step: &Step) -> Box<[u8]> {
    let limits = transcript::CEILINGS;
    let text = |bytes: &[u8]| {
        transcript::Block::Text(
            transcript::Text::new(&limits, transcript::TextParts { text: bytes.into(), replay: None })
                .expect("bounded turn text"),
        )
    };
    let message = |role, blocks: Vec<transcript::Block>| {
        let mut list = List::with_capacity(u32::try_from(blocks.len()).expect("few blocks"));
        for block in blocks {
            list.push(block).expect("counted blocks");
        }
        transcript::Message::new(&limits, transcript::MessageParts { role, blocks: list }).expect("bounded message")
    };
    let call = |name: &str, input: &[u8]| {
        transcript::Block::Call(
            transcript::Call::new(
                &limits,
                transcript::CallParts {
                    id: format!("call-{}", agent.said).into_bytes().into(),
                    name: name.as_bytes().into(),
                    input: input.into(),
                    replay: None,
                },
            )
            .expect("bounded call"),
        )
    };
    let said = match step {
        Step::Call { tool, input } => call(tool, input.as_bytes()),
        Step::Yield => text(b"waiting for the delegated result"),
        // The finish's input is its result, so a resumed run can accept it again.
        Step::Finish(outcome) => call("finish", outcome),
    };
    let mut messages = List::with_capacity(u32::try_from(texts.len() + 1).expect("few messages"));
    for words in texts {
        messages.push(message(transcript::Role::User, vec![text(words)])).expect("counted messages");
    }
    messages.push(message(transcript::Role::Assistant, vec![said])).expect("counted messages");
    let value = transcript::Turn::new(
        &limits,
        transcript::TurnParts {
            kind: transcript::TurnKind::Ordinary,
            affinity: None,
            endpoint: b"fake".as_slice().into(),
            dialect: b"fake".as_slice().into(),
            place: agent.said,
            usage: transcript::Usage::new(
                &limits,
                transcript::UsageParts {
                    input: Some(1),
                    output: Some(1),
                    cache_read: Some(0),
                    cache_write: Some(0),
                    reasoning: None,
                },
            )
            .expect("usage"),
            spent: agent.spent,
            messages,
        },
    )
    .expect("bounded turn");
    let mut writer = Writer::new(usize::try_from(value.measure()).expect("turn size"));
    value.encode(&mut writer).expect("measured turn");
    writer.finish()
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|part| part == needle)
}
fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    find(haystack, needle).is_some()
}

fn call(tool: &'static str, input: String) -> Step {
    Step::Call { tool, input }
}
fn logs() -> Step {
    call(
        "read_logs",
        r#"{"environment":"production","service":"checkout","from":0,"through":7200,"max_bytes":256,"filter":""}"#
            .into(),
    )
}
fn finish(text: &str) -> Step {
    Step::Finish(result(smith_charter::Form::Report, None, text.as_bytes(), &[]))
}
fn verdict(code: &str, text: &str, fields: &[(&str, &str)]) -> Box<[u8]> {
    result(smith_charter::Form::Verdict, Some(code.as_bytes()), text.as_bytes(), fields)
}
/// The run's accepted result, in the record its outcome carries.
fn result(form: smith_charter::Form, label: Option<&[u8]>, text: &[u8], fields: &[(&str, &str)]) -> Box<[u8]> {
    let limits = smith_charter::v2::CEILINGS;
    let mut list = List::with_capacity(u32::try_from(fields.len()).expect("few fields"));
    for (name, value) in fields {
        list.push(
            smith_charter::Field::new(
                &limits,
                smith_charter::FieldParts { name: name.as_bytes().into(), text: value.as_bytes().into() },
            )
            .expect("bounded field"),
        )
        .expect("counted fields");
    }
    let value = smith_charter::RunResult::new(
        &limits,
        smith_charter::RunResultParts {
            form,
            label: label.map(Box::from),
            text: text.into(),
            fields: list,
            items: List::with_capacity(0),
        },
    )
    .expect("bounded result");
    let mut writer = Writer::new(usize::try_from(value.measure()).expect("result size"));
    value.encode(&mut writer).expect("measured result");
    writer.finish()
}

const NOISE_CODE: &str = "1";
const NOISE_TEXT: &str = "known harmless error; noise";
const INCIDENT_CODE: &str = "2";
const INCIDENT_TEXT: &str = "checkout errors";
const INCIDENT_FIELDS: &[(&str, &str)] =
    &[("alerts", "12"), ("environment", "production"), ("service", "checkout"), ("severity", "3")];

fn watch() -> String {
    delegate(
        "watch checkout",
        r#"{"kind":"procedure","connector":1,"code":1}"#,
        80,
        5,
        3,
        false,
        r#"{"name":1,"kind":"bytes","value":"production"},{"name":2,"kind":"bytes","value":"checkout"},{"name":3,"kind":"number","value":1}"#,
        r#", "wake":{"words":{"kind":"immediate"},"notices":{"kind":"immediate"},"news":{"kind":"batch","count":3,"age":1000000000},"results":"last_or_failure","questions":true,"answers":true,"timers":true}"#,
    )
}
fn remediate(client_backed: bool) -> String {
    delegate(
        "restart checkout",
        r#"{"kind":"procedure","connector":2,"code":2}"#,
        10,
        0,
        0,
        !client_backed,
        r#"{"name":1,"kind":"bytes","value":"production"},{"name":2,"kind":"bytes","value":"checkout"},{"name":3,"kind":"number","value":77},{"name":4,"kind":"number","value":7200}"#,
        "",
    )
}
fn provision() -> String {
    format!(
        r#"{{"executor":{{"kind":"procedure","connector":2,"code":{}}},"spec":{{"words":"staging for payments, a week","parameters":[{{"name":1,"kind":"bytes","value":"staging"}},{{"name":2,"kind":"bytes","value":"payments"}},{{"name":3,"kind":"number","value":604800}},{{"name":4,"kind":"number","value":40}},{{"name":5,"kind":"number","value":7200}}]}},"contract":{{"kind":"report","words":128}},"authority":{{"tools":1023,"grants":[{{"connector":2,"kind":4,"segments":["pool"],"terminal":"exact","last":"staging"}},{{"connector":2,"kind":4,"segments":["pool","staging","environment"],"terminal":"open","last":""}}],"delegation":{{"kinds":[],"tasks":0,"depth":0}},"budget":{{"spend":40}},"notes":0}}}}"#,
        jig_ops_domain_infrastructure::PROVISION_CODE,
    )
}

#[expect(clippy::too_many_arguments, reason = "the fixture spells out the finite delegate envelope")]
pub fn delegate(
    words: &str,
    executor: &str,
    spend: u64,
    tasks: u32,
    depth: u32,
    production: bool,
    parameters: &str,
    wake: &str,
) -> String {
    let environment = if production { "production" } else { "staging" };
    format!(
        r#"{{"executor":{executor},"spec":{{"words":"{words}","parameters":[{parameters}]}},"contract":{{"kind":"report","words":128}},"authority":{{"tools":1023,"grants":[{{"connector":2,"kind":1,"segments":["env","{environment}","service"],"terminal":"open","last":""}},{{"connector":2,"kind":2,"segments":["env","{environment}","service"],"terminal":"open","last":""}},{{"connector":1,"kind":2,"segments":[],"terminal":"open","last":""}},{{"connector":1,"kind":3,"segments":[],"terminal":"open","last":""}},{{"connector":1,"kind":4,"segments":[],"terminal":"open","last":""}}],"delegation":{{"kinds":[{{"kind":"agent","number":1}},{{"kind":"procedure","number":1}},{{"kind":"procedure","number":2}}],"tasks":{tasks},"depth":{depth}}},"budget":{{"spend":{spend}}},"notes":0}}{wake}}}"#
    )
}
pub fn triage() -> Box<[u8]> {
    let delegate = delegate("triage checkout", r#"{"kind":"agent","charter":1}"#, 60, 0, 0, false, "", "");
    let envelope = delegate.replace(r#""contract":{"kind":"report","words":128}"#, r#""contract":{"kind":"verdict","choices":[{"code":1,"words":128},{"code":2,"words":128,"fields":[{"name":"alerts","max":192},{"name":"environment","max":64},{"name":"service","max":64},{"name":"severity","max":3}]}]}"#);
    format!(r#"{{"batch":[{envelope}]}}"#).into_bytes().into()
}

pub fn lead() -> Box<[u8]> {
    format!(
        r#"{{"batch":[{}]}}"#,
        delegate("@incident_lead", r#"{"kind":"agent","charter":1}"#, 40, 1, 1, false, "", "")
    )
    .into_bytes()
    .into()
}
