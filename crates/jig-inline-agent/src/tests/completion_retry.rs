use super::*;
use crate::{Completion, Provisional, reclaim, resume, terminal};
use alloc::vec::Vec;

const CLIENT: Token = Token::new(7);
fn drain(out: &mut Queue<Request>) -> Vec<Request> {
    let mut requests = Vec::new();
    while let Some(request) = out.pop() {
        requests.push(request);
    }
    requests
}
fn named(requests: &[Request]) -> (Token, smith::llm::CompletionName) {
    requests
        .iter()
        .find_map(|request| match request {
            Request::Lower { request: smith::Request::Complete { owner, name, .. }, .. } => Some((*owner, *name)),
            _ => None,
        })
        .expect("actual emitted completion")
}
fn spawn(agent: &mut Agent, env: &Env<Limits>, start: host::Start, out: &mut Queue<Request>) -> Vec<Request> {
    step(agent, env, host::Event::Spawn { client: CLIENT, start }, out);
    drain(out)
}
fn retry(agent: &mut Agent, env: &mut Env<Limits>, owner: Token, out: &mut Queue<Request>) -> Vec<Request> {
    terminal(
        agent,
        env,
        CLIENT,
        Completion::Failed {
            owner,
            failure: smith::llm::Failure::Unavailable,
            evidence: smith::llm::Evidence::Unknown,
            detail: Box::from(&b"provider failed"[..]),
        },
        out,
    );
    assert!(out.is_empty(), "failed whole completion supplies no partial text or synthetic start");
    env.now = agent.next_deadline().expect("actual Smith retry deadline");
    fire(agent, env, out);
    drain(out)
}
fn completed(
    agent: &mut Agent,
    env: &Env<Limits>,
    owner: Token,
    content: Box<[smith::llm::Said]>,
    out: &mut Queue<Request>,
) -> Vec<Request> {
    terminal(
        agent,
        env,
        CLIENT,
        Completion::Completed {
            owner,
            completion: smith::llm::Completion {
                content,
                stop: smith::llm::Stop::EndTurn,
                usage: smith::llm::Usage {
                    input_tokens: 1,
                    output_tokens: 1,
                    cache_read_tokens: 0,
                    cache_write_tokens: 0,
                },
            },
        },
        out,
    );
    drain(out)
}
fn settle(agent: &mut Agent, env: &Env<Limits>, out: &mut Queue<Request>, requests: &mut Vec<Request>) {
    for _ in 0..16 {
        reclaim(agent);
        if !agent.is_ready() {
            return;
        }
        resume(agent, env, out);
        requests.extend(drain(out));
    }
    assert!(!agent.is_ready(), "finite fixture reaches its external boundary");
}
fn said(text: &[u8]) -> smith::llm::Said {
    smith::llm::Said::Text { text: Box::from(text), replay: None }
}

#[test]
fn mandatory_no_text_retry_starts_survive_zero_observation_capacity() {
    let (mut agent, limits, start) = fixture_with(Some(0), false);
    let mut env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut out = Queue::with_capacity(max_out(&limits));
    let initial = spawn(&mut agent, &env, start, &mut out);
    let (owner, first) = named(&initial);
    assert!(initial.contains(&Request::CompletionStarted { client: CLIENT, turn: 1, retry: 0 }));
    assert_eq!(first, smith::llm::CompletionName { sequence: 1, retry: 0 });
    assert_ne!(owner, CLIENT, "the opaque main session is not the opener token");
    for retry_number in 1..=2 {
        let requests = retry(&mut agent, &mut env, owner, &mut out);
        assert_eq!(named(&requests), (owner, smith::llm::CompletionName { sequence: 1, retry: retry_number }));
        assert_eq!(
            requests.first(),
            Some(&Request::CompletionStarted { client: CLIENT, turn: 1, retry: retry_number })
        );
        assert!(!requests.iter().any(|request| matches!(request, Request::Content { .. })));
    }
    let requests = completed(&mut agent, &env, owner, Box::new([said(b"dropped observation")]), &mut out);
    assert!(!requests.iter().any(|request| matches!(request, Request::Content { .. })));
    assert!(
        requests
            .iter()
            .any(|request| matches!(request, Request::Host(host::Request::Turn { turn, .. }) if turn.number == 1))
    );
}

#[test]
fn captured_text_call_and_usage_keep_their_issued_retry_before_the_next_start() {
    let (mut agent, limits, start) = fixture();
    let mut env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut out = Queue::with_capacity(max_out(&limits));
    let (owner, _) = named(&spawn(&mut agent, &env, start, &mut out));
    let (_, issued) = named(&retry(&mut agent, &mut env, owner, &mut out));
    let call = smith::llm::Said::ToolCall {
        id: Box::from(&b"opaque-id"[..]),
        name: Box::from(&b"unknown"[..]),
        input: Box::from(&b"{}"[..]),
        call: smith::llm::Decoded::Invalid { problem: smith::llm::Problem::UnknownTool },
        replay: None,
    };
    let mut captured = completed(&mut agent, &env, owner, Box::new([said(b"earlier"), call]), &mut out);
    settle(&mut agent, &env, &mut out, &mut captured);
    let mut kinds = Vec::new();
    for request in &captured {
        if let Request::Content { client, provisional, content } = request {
            assert_eq!(*client, CLIENT);
            match content {
                smith::Content::Text { owner: seen, completion, text } => {
                    assert_eq!((*seen, *completion, text.as_ref()), (owner, issued, &b"earlier"[..]));
                    assert_eq!(*provisional, Some(Provisional { turn: 1, retry: 1, offset: 0 }));
                    kinds.push("text");
                }
                smith::Content::Call { owner: seen, completion, id, name, input } => {
                    assert_eq!((*seen, *completion), (owner, issued));
                    assert_eq!(
                        (id.as_ref(), name.as_ref(), input.as_ref()),
                        (&b"opaque-id"[..], &b"unknown"[..], &b"{}"[..])
                    );
                    assert_eq!(*provisional, Some(Provisional { turn: 1, retry: 1, offset: 7 }));
                    kinds.push("call");
                }
                smith::Content::Usage { owner: seen, completion, .. } => {
                    assert_eq!((*seen, *completion), (owner, issued));
                    assert_eq!(*provisional, Some(Provisional { turn: 1, retry: 1, offset: 7 }));
                    kinds.push("usage");
                }
                smith::Content::Tool { .. } => assert!(provisional.is_none()),
            }
        }
    }
    assert_eq!(kinds, ["text", "call", "usage"]);
    let turn_at = captured
        .iter()
        .position(|r| matches!(r, Request::Host(host::Request::Turn { turn, .. }) if turn.number == 1))
        .unwrap();
    assert!(captured[..turn_at].iter().filter(|r| matches!(r, Request::Content { .. })).count() >= 3);
    let next = &captured;
    assert_eq!(named(&next), (owner, smith::llm::CompletionName { sequence: 2, retry: 0 }));
    assert!(next.contains(&Request::CompletionStarted { client: CLIENT, turn: 2, retry: 0 }));
}

#[test]
fn restored_source_sequence_and_retry_do_not_renumber_the_activation_turn() {
    let (mut agent, limits, start) = fixture();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut out = Queue::with_capacity(max_out(&limits));
    let (owner, _) = named(&spawn(&mut agent, &env, start, &mut out));
    let mut saved = None;
    for request in completed(&mut agent, &env, owner, Box::new([said(b"historical turn")]), &mut out) {
        if let Request::Host(host::Request::Turn { turn, .. }) = request {
            saved = Some(turn.body);
        }
    }
    let (mut restored, limits, mut start) = fixture();
    start.activation = 2;
    start.transcript = Some(Box::new([saved.expect("real encoded historical turn")]));
    let mut env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let initial = spawn(&mut restored, &env, start, &mut out);
    let (owner, name) = named(&initial);
    assert_eq!(name, smith::llm::CompletionName { sequence: 2, retry: 0 });
    assert!(initial.contains(&Request::CompletionStarted { client: CLIENT, turn: 1, retry: 0 }));
    let retried = retry(&mut restored, &mut env, owner, &mut out);
    assert_eq!(named(&retried), (owner, smith::llm::CompletionName { sequence: 2, retry: 1 }));
    assert!(retried.contains(&Request::CompletionStarted { client: CLIENT, turn: 1, retry: 1 }));
    let requests = completed(&mut restored, &env, owner, Box::new([said(b"new activation")]), &mut out);
    assert!(requests.iter().any(|r| matches!(
        r,
        Request::Content {
            provisional: Some(Provisional { turn: 1, retry: 1, offset: 0 }),
            content: smith::Content::Text { completion: smith::llm::CompletionName { sequence: 2, retry: 1 }, .. },
            ..
        }
    )));
    assert!(requests.iter().any(|r| matches!(r, Request::Host(host::Request::Turn { turn, .. }) if turn.number == 1)));
}

#[test]
fn nested_retry_keeps_its_opaque_owner_without_becoming_main_writing() {
    let (mut agent, limits, start) = fixture_with(None, true);
    let mut env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut out = Queue::with_capacity(max_out(&limits));
    let (main, parent) = named(&spawn(&mut agent, &env, start, &mut out));
    let call = smith::llm::Said::ToolCall {
        id: Box::from(&b"child"[..]),
        name: Box::from(&b"sub_agent"[..]),
        input: Box::from(&b"{}"[..]),
        call: smith::llm::Decoded::Served {
            ask: smith::run::Ask::SubAgent {
                brief: Box::from(&b"Find it"[..]),
                families: smith::run::charter::Families {
                    tools: smith::run::charter::Tools { inspect: false, modify: false, shell: false },
                    agents: true,
                },
                llm: None,
                share: None,
            },
        },
        replay: None,
    };
    let mut requests = completed(&mut agent, &env, main, Box::new([call]), &mut out);
    settle(&mut agent, &env, &mut out, &mut requests);
    let (child, first) = named(&requests);
    assert_ne!(child, main);
    assert_eq!(first, smith::llm::CompletionName { sequence: 1, retry: 0 });
    assert!(!requests.iter().any(|r| matches!(r, Request::CompletionStarted { .. })));
    assert!(requests.iter().any(|r| matches!(r, Request::Content {
        provisional: Some(Provisional { turn: 1, retry: 0, .. }),
        content: smith::Content::Call { owner, completion, .. }, ..
    } if *owner == main && *completion == parent)));
    let requests = retry(&mut agent, &mut env, child, &mut out);
    let (_, child_retry) = named(&requests);
    assert!(!requests.iter().any(|r| matches!(r, Request::CompletionStarted { .. })));
    let requests = completed(&mut agent, &env, child, Box::new([said(b"child answer")]), &mut out);
    assert!(requests.iter().any(|r| matches!(r, Request::Content {
        provisional: None, content: smith::Content::Text { owner, completion, text }, ..
    } if *owner == child && *completion == child_retry && text.as_ref() == b"child answer")));
}
