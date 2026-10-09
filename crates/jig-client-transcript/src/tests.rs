use super::*;
use smith::{Block as SBlock, CEILINGS, MessageParts, TurnParts, UsageParts};

fn limits() -> Limits {
    Limits { smith: CEILINGS, bytes: 1_048_576, turn: TurnLimits { messages: 64, blocks: 16, text: 65_536 } }
}
fn text(value: &[u8]) -> smith::Text {
    smith::Text::new(&CEILINGS, smith::TextParts { text: Box::from(value), replay: None }).unwrap()
}
fn call(id: &[u8]) -> SBlock {
    SBlock::Call(
        smith::Call::new(
            &CEILINGS,
            smith::CallParts {
                id: Box::from(id),
                name: Box::from(&b"read"[..]),
                input: Box::from(&b"{ \"path\": \"a\" }"[..]),
                replay: None,
            },
        )
        .unwrap(),
    )
}
fn result(id: &[u8], returned: smith::Returned) -> SBlock {
    SBlock::Result(smith::ToolResult::new(&CEILINGS, smith::ToolResultParts { id: Box::from(id), returned }).unwrap())
}
fn owned(value: smith::Outcome) -> smith::Returned {
    smith::Returned::Outcome(smith::OwnedOutcome::new(&CEILINGS, smith::OwnedOutcomeParts { value }).unwrap())
}
fn message(role: smith::Role, blocks: &[SBlock]) -> smith::Message {
    let mut list = List::with_capacity(u32::try_from(blocks.len()).unwrap());
    for block in blocks {
        list.push(block.clone()).unwrap();
    }
    smith::Message::new(&CEILINGS, MessageParts { role, blocks: list }).unwrap()
}
fn source(messages: &[smith::Message]) -> smith::Turn {
    let mut list = List::with_capacity(u32::try_from(messages.len()).unwrap());
    for message in messages {
        list.push(message.clone()).unwrap();
    }
    smith::Turn::new(
        &CEILINGS,
        TurnParts {
            endpoint: Box::from(&b"endpoint"[..]),
            dialect: Box::from(&b"dialect"[..]),
            place: 1,
            usage: smith::Usage::new(&CEILINGS, UsageParts { input: 11, output: 12, cache_read: 13, cache_write: 14 })
                .unwrap(),
            spent: 15,
            messages: list,
        },
    )
    .unwrap()
}
fn encode(turn: &smith::Turn) -> Box<[u8]> {
    let mut writer = skein_lib::Writer::new(usize::try_from(turn.measure()).unwrap());
    turn.encode(&mut writer).unwrap();
    writer.finish()
}

#[test]
fn whole_turn_preserves_blocks_and_joins_results_to_calls() {
    let source = source(&[
        message(
            smith::Role::Assistant,
            &[
                SBlock::Text(text(b"hello")),
                SBlock::Refusal(text(b"cannot")),
                call(b"id"),
                SBlock::Opaque(
                    smith::Opaque::new(
                        &CEILINGS,
                        smith::OpaqueParts { dialect: Box::from(&b"secret"[..]), bytes: Box::from(&[0xff, 0x00][..]) },
                    )
                    .unwrap(),
                ),
            ],
        ),
        message(
            smith::Role::User,
            &[result(
                b"id",
                owned(smith::Outcome::Read(
                    smith::Read::new(
                        &CEILINGS,
                        smith::ReadParts {
                            content: Box::from(&b"a\nb"[..]),
                            skipped: 1,
                            lines: 2,
                            total: 9,
                            cut: true,
                        },
                    )
                    .unwrap(),
                )),
            )],
        ),
    ]);
    let turn = decode(&limits(), &encode(&source)).unwrap();
    assert_eq!(turn.used, Usage { input: 11, output: 12, cached: 13, cache_written: 14 });
    assert_eq!(turn.spent, 15);
    assert_eq!(turn.read_through, None);
    assert!(!turn.cut);
    assert_eq!(turn.messages.len(), 2);
    assert_eq!(turn.messages.get(0).unwrap().speaker, Speaker::Assistant);
    assert_eq!(turn.messages.get(1).unwrap().speaker, Speaker::Person);
    assert_eq!(
        turn.messages.get(0).unwrap().blocks.get(0).unwrap(),
        &Block::Text(Text::new(Box::from(&b"hello"[..]), 5).unwrap())
    );
    assert_eq!(
        turn.messages.get(0).unwrap().blocks.get(1).unwrap(),
        &Block::Text(Text::new(Box::from(&b"cannot"[..]), 6).unwrap())
    );
    match &turn.messages.get(0).unwrap().blocks.get(2).unwrap() {
        Block::Call(call) => {
            assert_eq!(call.number, 3);
            assert_eq!(call.tool.bytes(), b"read");
            assert_eq!(call.input.bytes(), b"{ \"path\": \"a\" }");
            assert_eq!(call.outcome.as_ref().unwrap().bytes(), b"Read 2 of 9 lines; skipped 1; content cut");
            assert!(!call.failed);
            assert!(call.objects.is_empty());
            assert!(call.parts.is_empty());
        }
        Block::Text(_) | Block::Opaque { .. } | Block::Result { .. } => panic!("call expected"),
    }
    match &turn.messages.get(0).unwrap().blocks.get(3).unwrap() {
        Block::Opaque { dialect, size } => {
            assert_eq!(dialect.bytes(), b"secret");
            assert_eq!(*size, 2);
        }
        Block::Text(_) | Block::Call(_) | Block::Result { .. } => panic!("opaque expected"),
    }
    match &turn.messages.get(1).unwrap().blocks.get(0).unwrap() {
        Block::Result { call, summary, failed } => {
            assert_eq!(*call, 3);
            assert_eq!(summary.bytes(), b"Read 2 of 9 lines; skipped 1; content cut");
            assert!(!failed);
        }
        Block::Text(_) | Block::Call(_) | Block::Opaque { .. } => panic!("result expected"),
    }
}

#[test]
fn outcomes_summarize_successes_command_endings_and_tool_refusals() {
    let cases: [(smith::Outcome, &[u8], bool); 32] = [
        (
            smith::Outcome::Listed(
                smith::Listed::new(&CEILINGS, smith::ListedParts { entries: List::with_capacity(0), more: 4 }).unwrap(),
            ),
            b"Listed 0 entries; more 4",
            false,
        ),
        (
            smith::Outcome::Found(
                smith::Found::new(
                    &CEILINGS,
                    smith::FoundParts { hits: List::with_capacity(0), more: 3, timed_out: true },
                )
                .unwrap(),
            ),
            b"Found 0 hits; more 3; search timed out",
            false,
        ),
        (
            smith::Outcome::Written(smith::Written::new(&CEILINGS, smith::WrittenParts { created: true }).unwrap()),
            b"Created file",
            false,
        ),
        (
            smith::Outcome::Written(smith::Written::new(&CEILINGS, smith::WrittenParts { created: false }).unwrap()),
            b"Wrote file",
            false,
        ),
        (
            smith::Outcome::Edited(smith::Edited::new(&CEILINGS, smith::EditedParts { replaced: 7 }).unwrap()),
            b"Edited 7 matches",
            false,
        ),
        (
            exited(smith::Exit::Code(smith::ExitCode::new(&CEILINGS, smith::ExitCodeParts { code: 0 }).unwrap())),
            b"Command ended: exit 0; output bytes 2; dropped 9",
            false,
        ),
        (
            exited(smith::Exit::Code(smith::ExitCode::new(&CEILINGS, smith::ExitCodeParts { code: 1 }).unwrap())),
            b"Command ended: exit 1; output bytes 2; dropped 9",
            true,
        ),
        (
            exited(smith::Exit::Signal(
                smith::ExitSignal::new(&CEILINGS, smith::ExitSignalParts { signal: 9 }).unwrap(),
            )),
            b"Command ended: signal 9; output bytes 2; dropped 9",
            true,
        ),
        (exited(smith::Exit::TimedOut), b"Command ended: timed out; output bytes 2; dropped 9", true),
        (smith::Outcome::NotGranted, b"Tool refused: not granted", true),
        (smith::Outcome::Outside, b"Tool refused: outside workspace", true),
        (smith::Outcome::ReadOnly, b"Tool refused: read only", true),
        (smith::Outcome::TooLong, b"Tool refused: path too long", true),
        (smith::Outcome::NotFound, b"Tool refused: not found", true),
        (smith::Outcome::NotFile, b"Tool refused: not a file", true),
        (smith::Outcome::Linked, b"Tool refused: linked path", true),
        (smith::Outcome::Protected, b"Tool refused: protected path", true),
        (smith::Outcome::NotDirectory, b"Tool refused: not a directory", true),
        (
            smith::Outcome::TooLarge(smith::TooLarge::new(&CEILINGS, smith::TooLargeParts { size: u64::MAX }).unwrap()),
            b"Tool refused: too large, 18446744073709551615 bytes",
            true,
        ),
        (smith::Outcome::NotRead, b"Tool refused: not read", true),
        (smith::Outcome::Stale, b"Tool refused: stale read", true),
        (smith::Outcome::NoMatch, b"Tool refused: no match", true),
        (
            smith::Outcome::Ambiguous(
                smith::Ambiguous::new(&CEILINGS, smith::AmbiguousParts { count: 2, lines: List::with_capacity(0) })
                    .unwrap(),
            ),
            b"Tool refused: ambiguous, 2 matches",
            true,
        ),
        (smith::Outcome::Unchanged, b"Unchanged", false),
        (failed(smith::Fault::Denied), b"Tool failed: denied", true),
        (failed(smith::Fault::NoSpace), b"Tool failed: no space", true),
        (failed(smith::Fault::Other), b"Tool failed: other fault", true),
        (smith::Outcome::TimedOut, b"Tool timed out", true),
        (smith::Outcome::Cancelled, b"Tool cancelled", true),
        (smith::Outcome::Busy, b"Tool busy", true),
        (smith::Outcome::NulByte, b"Tool refused: NUL byte", true),
        (
            smith::Outcome::Read(
                smith::Read::new(
                    &CEILINGS,
                    smith::ReadParts { content: Box::from(&b""[..]), skipped: 0, lines: 0, total: 0, cut: false },
                )
                .unwrap(),
            ),
            b"Read 0 of 0 lines; skipped 0",
            false,
        ),
    ];
    for (outcome, expected, failed) in cases {
        assert_result(owned(outcome), expected, failed);
    }
}
fn exited(exit: smith::Exit) -> smith::Outcome {
    smith::Outcome::Exited(
        smith::CommandEnd::new(
            &CEILINGS,
            smith::CommandEndParts { exit, head: Box::from(&b"h"[..]), last: Box::from(&b"l"[..]), dropped: 9 },
        )
        .unwrap(),
    )
}
fn failed(fault: smith::Fault) -> smith::Outcome {
    smith::Outcome::Failed(smith::Failed::new(&CEILINGS, smith::FailedParts { fault }).unwrap())
}
fn assert_result(returned: smith::Returned, expected: &[u8], failed: bool) {
    let turn =
        decode(&limits(), &encode(&source(&[message(smith::Role::User, &[result(b"unpaired", returned)])]))).unwrap();
    match &turn.messages.get(0).unwrap().blocks.get(0).unwrap() {
        Block::Result { call, summary, failed: value } => {
            assert_eq!(*call, 1);
            assert_eq!(summary.bytes(), expected);
            assert_eq!(*value, failed);
        }
        Block::Text(_) | Block::Call(_) | Block::Opaque { .. } => panic!("result expected"),
    }
}

#[test]
fn tool_text_and_invalid_calls_remain_distinct() {
    for error in [false, true] {
        assert_result(
            smith::Returned::Said(
                smith::Said::new(&CEILINGS, smith::SaidParts { text: Box::from(&b"tool words"[..]), error }).unwrap(),
            ),
            b"tool words",
            error,
        );
    }
    let field =
        smith::FieldProblem::new(&CEILINGS, smith::FieldProblemParts { field: Box::from(&b"path"[..]) }).unwrap();
    for (problem, expected) in [
        (smith::CallProblem::UnknownTool, &b"Invalid call: unknown tool"[..]),
        (smith::CallProblem::NotAnObject, b"Invalid call: input is not an object"),
        (smith::CallProblem::Missing(field.clone()), b"Invalid call: missing path"),
        (smith::CallProblem::WrongType(field.clone()), b"Invalid call: wrong type for path"),
        (smith::CallProblem::BadValue(field), b"Invalid call: bad value for path"),
        (smith::CallProblem::TooLarge, b"Invalid call: input too large"),
    ] {
        assert_result(
            smith::Returned::Invalid(smith::Invalid::new(&CEILINGS, smith::InvalidParts { problem }).unwrap()),
            expected,
            true,
        );
    }
    assert_result(smith::Returned::NotRun, b"Not run", true);
    assert_result(smith::Returned::Withdrawn, b"Withdrawn", true);
}

#[test]
fn admission_errors_are_explicit_and_whole_records_are_required() {
    assert_eq!(decode(&limits(), &[0, 3]), Err(Undecodable::Version { found: 3 }));
    assert_eq!(decode(&limits(), &[0]), Err(Undecodable::Malformed));
    assert_eq!(decode(&limits(), &[0, 2, 0, 0, 0, 65]), Err(Undecodable::TooLarge));
    let source = source(&[message(smith::Role::Assistant, &[SBlock::Text(text(b"hello"))])]);
    let mut bytes = encode(&source);
    let mut small = limits();
    small.bytes = 1;
    assert_eq!(decode(&small, &bytes), Err(Undecodable::TooLarge));
    small = limits();
    small.smith.text_text = 4;
    assert_eq!(decode(&small, &bytes), Err(Undecodable::TooLarge));
    let mut trailing = skein_lib::Writer::new(bytes.len().checked_add(1).unwrap());
    trailing.put(&bytes).unwrap();
    trailing.put(&[0]).unwrap();
    assert_eq!(decode(&limits(), &trailing.finish()), Err(Undecodable::Malformed));
    let role_at = bytes.len() - usize::try_from(source.messages().get(0).unwrap().measure()).unwrap();
    bytes[role_at] = 255;
    assert_eq!(decode(&limits(), &bytes), Err(Undecodable::Unknown));
}

#[test]
fn receiving_limits_cut_without_growing_or_losing_call_outcomes() {
    let source = source(&[
        message(smith::Role::Assistant, &[call(b"id")]),
        message(smith::Role::User, &[result(b"id", owned(smith::Outcome::NotGranted)), SBlock::Text(text(b"words"))]),
    ]);
    let bytes = encode(&source);
    let mut small = limits();
    small.turn.messages = 1;
    let turn = decode(&small, &bytes).unwrap();
    assert!(turn.cut);
    assert_eq!(turn.messages.len(), 1);
    match &turn.messages.get(0).unwrap().blocks.get(0).unwrap() {
        Block::Call(call) => {
            assert!(call.failed);
            assert_eq!(call.outcome.as_ref().unwrap().bytes(), b"Tool refused: not granted");
        }
        Block::Text(_) | Block::Result { .. } | Block::Opaque { .. } => panic!("call expected"),
    }
    small = limits();
    small.turn.blocks = 1;
    let turn = decode(&small, &bytes).unwrap();
    assert!(turn.cut);
    assert_eq!(turn.messages.get(1).unwrap().blocks.len(), 1);
    small = limits();
    small.turn.text = 2;
    let turn = decode(&small, &bytes).unwrap();
    assert!(turn.cut);
    match &turn.messages.get(0).unwrap().blocks.get(0).unwrap() {
        Block::Call(call) => {
            assert_eq!(call.tool.bytes(), b"re");
            assert!(call.input.bytes().is_empty());
            assert!(call.outcome.as_ref().unwrap().bytes().is_empty());
        }
        Block::Text(_) | Block::Result { .. } | Block::Opaque { .. } => panic!("call expected"),
    }
    small.turn.messages = 0;
    let turn = decode(&small, &bytes).unwrap();
    assert!(turn.cut);
    assert!(turn.messages.is_empty());
    assert_eq!(turn.spent, 15);
}

#[test]
#[expect(clippy::disallowed_macros, reason = "fixed wire fixtures belong to decoder unit tests")]
fn smith_smallest_golden_decodes_at_its_whole_length() {
    let bytes = include_bytes!("../tests/data/record_turn_smallest.bin");
    assert_eq!(bytes.len(), 58);
    let value = decode(&limits(), bytes).unwrap();
    assert_eq!(
        value,
        Turn {
            messages: List::with_capacity(0),
            used: Usage { input: 0, output: 0, cached: 0, cache_written: 0 },
            spent: 0,
            read_through: None,
            cut: false
        }
    );
}

#[test]
#[expect(clippy::disallowed_macros, reason = "fixed wire fixtures belong to decoder unit tests")]
fn smith_full_golden_preserves_maximum_usage_at_its_whole_length() {
    let bytes = include_bytes!("../tests/data/record_turn_full.bin");
    assert_eq!(bytes.len(), 92);
    let mut blocks = List::with_capacity(1);
    blocks.push(Block::Text(Text::new(Box::from(&b"abc"[..]), 3).unwrap())).unwrap();
    let mut messages = List::with_capacity(1);
    messages.push(Message { speaker: Speaker::Person, blocks }).unwrap();
    assert_eq!(
        decode(&limits(), bytes).unwrap(),
        Turn {
            messages,
            used: Usage { input: u64::MAX, output: u64::MAX, cached: u64::MAX, cache_written: u64::MAX },
            spent: u64::MAX,
            read_through: None,
            cut: false
        }
    );
}
