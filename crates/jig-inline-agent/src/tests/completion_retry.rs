use super::*;
use crate::{Completion, Provisional, reclaim, resume, terminal};

const CLIENT: Token = Token::new(7);
fn drain(out: &mut Queue<Request>) -> Box<[Request]> {
    let mut requests = List::with_capacity(out.len());
    for _ in 0..out.len() {
        requests.push(out.pop().expect("bounded queued request")).expect("bounded drain");
    }
    requests.into_boxed()
}
fn named(requests: &[Request]) -> (Token, smith::llm::CompletionName) {
    for request in requests {
        match request {
            Request::Lower { request: smith::Request::Complete { owner, name, .. }, .. } => return (*owner, *name),
            Request::Lower { .. } | Request::Host(..) | Request::Content { .. } | Request::CompletionStarted { .. } => {
            }
        }
    }
    panic!("actual emitted completion")
}
fn content_count(requests: &[Request]) -> u32 {
    let mut count = 0_u32;
    for request in requests {
        match request {
            Request::Content { .. } => count = count.checked_add(1).expect("bounded content"),
            Request::Lower { .. } | Request::Host(..) | Request::CompletionStarted { .. } => {}
        }
    }
    count
}
fn has_turn(requests: &[Request], number: u32) -> bool {
    for request in requests {
        match request {
            Request::Host(host::Request::Turn { turn, .. }) if turn.number == number => return true,
            Request::Lower { .. } | Request::Host(..) | Request::Content { .. } | Request::CompletionStarted { .. } => {
            }
        }
    }
    false
}
fn has_start(requests: &[Request]) -> bool {
    for request in requests {
        match request {
            Request::CompletionStarted { .. } => return true,
            Request::Lower { .. } | Request::Host(..) | Request::Content { .. } => {}
        }
    }
    false
}
fn has_call(requests: &[Request], owner: Token, name: smith::llm::CompletionName, id: &[u8]) -> bool {
    for request in requests {
        match request {
            Request::Content { content: smith::Content::Call { owner: seen, completion, id: seen_id, .. }, .. }
                if *seen == owner && *completion == name && seen_id.as_ref() == id =>
            {
                return true;
            }
            Request::Lower { .. } | Request::Host(..) | Request::Content { .. } | Request::CompletionStarted { .. } => {
            }
        }
    }
    false
}
fn has_text(
    requests: &[Request],
    owner: Token,
    name: smith::llm::CompletionName,
    provisional: Option<Provisional>,
    text: &[u8],
) -> bool {
    for request in requests {
        match request {
            Request::Content {
                provisional: seen_provisional,
                content: smith::Content::Text { owner: seen, completion, text: seen_text },
                ..
            } if *seen == owner
                && *completion == name
                && *seen_provisional == provisional
                && seen_text.as_ref() == text =>
            {
                return true;
            }
            Request::Lower { .. } | Request::Host(..) | Request::Content { .. } | Request::CompletionStarted { .. } => {
            }
        }
    }
    false
}
fn spawn(agent: &mut Agent, env: &Env<Limits>, start: host::Start, out: &mut Queue<Request>) -> Box<[Request]> {
    step(agent, env, host::Event::Spawn { client: CLIENT, start }, out);
    drain(out)
}
fn retry(agent: &mut Agent, env: &mut Env<Limits>, owner: Token, out: &mut Queue<Request>) -> Box<[Request]> {
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
    env.now = crate::next_deadline(agent).expect("actual Smith retry deadline");
    fire(agent, env, out);
    drain(out)
}
fn completed(
    agent: &mut Agent,
    env: &Env<Limits>,
    owner: Token,
    content: Box<[smith::llm::Said]>,
    out: &mut Queue<Request>,
) -> Box<[Request]> {
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
fn settle(agent: &mut Agent, env: &Env<Limits>, out: &mut Queue<Request>, requests: &mut Box<[Request]>) {
    for _ in 0_u32..16_u32 {
        reclaim(agent);
        if !agent.is_ready() {
            return;
        }
        resume(agent, env, out);
        let next = drain(out);
        let capacity = u32::try_from(requests.len())
            .expect("bounded old requests")
            .checked_add(u32::try_from(next.len()).expect("bounded next requests"))
            .expect("bounded fixture output");
        let mut combined = List::with_capacity(capacity);
        for request in core::mem::replace(requests, Box::new([])) {
            combined.push(request).expect("bounded old output");
        }
        for request in next {
            combined.push(request).expect("bounded resumed output");
        }
        *requests = combined.into_boxed();
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
    for retry_number in 1_u32..=2_u32 {
        let requests = retry(&mut agent, &mut env, owner, &mut out);
        assert_eq!(named(&requests), (owner, smith::llm::CompletionName { sequence: 1, retry: retry_number }));
        assert_eq!(
            requests.first(),
            Some(&Request::CompletionStarted { client: CLIENT, turn: 1, retry: retry_number })
        );
        assert_eq!(content_count(&requests), 0);
    }
    let requests = completed(&mut agent, &env, owner, Box::new([said(b"dropped observation")]), &mut out);
    assert_eq!(content_count(&requests), 0);
    assert!(has_turn(&requests, 1));
}

#[test]
fn captured_text_call_and_usage_keep_their_issued_retry_before_the_next_start() {
    let (mut agent, limits, start) = fixture();
    let mut env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut out = Queue::with_capacity(max_out(&limits));
    let (owner, _) = named(&spawn(&mut agent, &env, start, &mut out));
    let (_, issued) = named(&retry(&mut agent, &mut env, owner, &mut out));
    let call = smith::llm::Said::ToolCall {
        id: Box::from(&[0xff, 0, 0x80][..]),
        name: Box::from(&b"unknown"[..]),
        input: Box::from(&b"{}"[..]),
        call: smith::llm::Decoded::Invalid { problem: smith::llm::Problem::UnknownTool },
        replay: None,
    };
    let mut captured = completed(&mut agent, &env, owner, Box::new([said(b"earlier"), call]), &mut out);
    settle(&mut agent, &env, &mut out, &mut captured);
    let mut kinds = List::with_capacity(3);
    for request in captured.as_ref() {
        match request {
            Request::Content { client, provisional, content } => {
                assert_eq!(*client, CLIENT);
                match content {
                    smith::Content::Text { owner: seen, completion, text } => {
                        assert_eq!((*seen, *completion, text.as_ref()), (owner, issued, &b"earlier"[..]));
                        assert_eq!(*provisional, Some(Provisional { turn: 1, retry: 1, offset: 0 }));
                        kinds.push("text").expect("three captured kinds");
                    }
                    smith::Content::Call { owner: seen, completion, id, name, input } => {
                        assert_eq!((*seen, *completion), (owner, issued));
                        assert_eq!(
                            (id.as_ref(), name.as_ref(), input.as_ref()),
                            (&[0xff, 0, 0x80][..], &b"unknown"[..], &b"{}"[..])
                        );
                        assert_eq!(*provisional, Some(Provisional { turn: 1, retry: 1, offset: 7 }));
                        kinds.push("call").expect("three captured kinds");
                    }
                    smith::Content::Usage { owner: seen, completion, .. } => {
                        assert_eq!((*seen, *completion), (owner, issued));
                        assert_eq!(*provisional, Some(Provisional { turn: 1, retry: 1, offset: 7 }));
                        kinds.push("usage").expect("three captured kinds");
                    }
                    smith::Content::Tool { .. } => assert!(provisional.is_none()),
                }
            }
            Request::Lower { .. } | Request::Host(..) | Request::CompletionStarted { .. } => {}
        }
    }
    assert_eq!(kinds.as_slice(), ["text", "call", "usage"]);
    let mut before_turn = 0_u32;
    let mut found_turn = false;
    for request in captured.as_ref() {
        match request {
            Request::Host(host::Request::Turn { turn, .. }) if turn.number == 1 => {
                found_turn = true;
                break;
            }
            Request::Content { .. } => before_turn = before_turn.checked_add(1).expect("bounded captured content"),
            Request::Lower { .. } | Request::Host(..) | Request::CompletionStarted { .. } => {}
        }
    }
    assert!(found_turn, "actual first turn");
    assert!(before_turn >= 3);
    let next = &captured;
    assert_eq!(named(next), (owner, smith::llm::CompletionName { sequence: 2, retry: 0 }));
    assert!(next.contains(&Request::CompletionStarted { client: CLIENT, turn: 2, retry: 0 }));
    let (_, second_name) = named(next);
    let second = completed(
        &mut agent,
        &env,
        owner,
        Box::new([smith::llm::Said::ToolCall {
            id: Box::from(&[0xff, 0, 0x80][..]),
            name: Box::from(&b"unknown"[..]),
            input: Box::from(&b"{}"[..]),
            call: smith::llm::Decoded::Invalid { problem: smith::llm::Problem::UnknownTool },
            replay: None,
        }]),
        &mut out,
    );
    assert_ne!(issued, second_name, "one opaque provider id in another completion names another call");
    assert!(has_call(&second, owner, second_name, &[0xff, 0, 0x80]));
}

#[test]
fn restored_source_sequence_and_retry_do_not_renumber_the_activation_turn() {
    let (mut agent, limits, start) = fixture();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut out = Queue::with_capacity(max_out(&limits));
    let (owner, _) = named(&spawn(&mut agent, &env, start, &mut out));
    let mut saved = None;
    for request in completed(&mut agent, &env, owner, Box::new([said(b"historical turn")]), &mut out) {
        match request {
            Request::Host(host::Request::Turn { turn, .. }) => saved = Some(turn.body),
            Request::Lower { .. } | Request::Host(..) | Request::Content { .. } | Request::CompletionStarted { .. } => {
            }
        }
    }
    let (mut restored, limits, mut start) = fixture_resumed(None, false, true);
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
    assert!(has_text(
        &requests,
        owner,
        smith::llm::CompletionName { sequence: 2, retry: 1 },
        Some(Provisional { turn: 1, retry: 1, offset: 0 }),
        b"new activation"
    ));
    assert!(has_turn(&requests, 1));
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
    assert_ne!(child, main, "actual typed admission and provider requests: {requests:?}");
    assert_eq!(first, smith::llm::CompletionName { sequence: 1, retry: 0 });
    assert!(!has_start(&requests));
    let mut parent_call = false;
    for request in requests.as_ref() {
        match request {
            Request::Content {
                provisional: Some(Provisional { turn: 1, retry: 0, .. }),
                content: smith::Content::Call { owner, completion, .. },
                ..
            } if *owner == main && *completion == parent => parent_call = true,
            Request::Lower { .. } | Request::Host(..) | Request::Content { .. } | Request::CompletionStarted { .. } => {
            }
        }
    }
    assert!(parent_call);
    let requests = retry(&mut agent, &mut env, child, &mut out);
    let (_, child_retry) = named(&requests);
    assert!(!has_start(&requests));
    let requests = completed(&mut agent, &env, child, Box::new([said(b"child answer")]), &mut out);
    assert!(has_text(&requests, child, child_retry, None, b"child answer"));
}
