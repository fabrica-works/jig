//! Completed calls show the worker's job start and end on the person's page.
use super::*;

use jig_client_test_domain::DIALECT;
use skein_fake_person::Target;
use smith_transcript::v6 as source;

fn call(id: &[u8], name: &[u8]) -> source::Block {
    source::Block::Call(
        source::Call::new(
            &source::CEILINGS,
            source::CallParts { id: id.into(), name: name.into(), input: b"{}".as_slice().into(), replay: None },
        )
        .expect("job story record fits the transcript schema"),
    )
}

fn result(id: &[u8], value: source::Outcome) -> source::Block {
    let returned = source::Returned::Outcome(
        source::OwnedOutcome::new(&source::CEILINGS, source::OwnedOutcomeParts { value })
            .expect("job story record fits the transcript schema"),
    );
    source::Block::Result(
        source::ToolResult::new(&source::CEILINGS, source::ToolResultParts { id: id.into(), returned })
            .expect("job story record fits the transcript schema"),
    )
}

fn turn() -> Box<[u8]> {
    let limits = source::CEILINGS;
    let exit = source::Exit::Code(
        source::ExitCode::new(&limits, source::ExitCodeParts { code: 0 })
            .expect("job story record fits the transcript schema"),
    );
    let end = source::JobEnd::Ended(
        source::JobExit::new(&limits, source::JobExitParts { exit })
            .expect("job story record fits the transcript schema"),
    );
    let ended = source::Outcome::Job(
        source::JobRead::new(
            &limits,
            source::JobReadParts {
                job: 1,
                head: Box::from([]),
                last: Box::from([]),
                dropped: 0,
                output: source::BytesClamp::Within,
                wait: source::TimeoutClamp::Within,
                end,
            },
        )
        .expect("job story record fits the transcript schema"),
    );
    let running = source::Outcome::Running(
        source::CommandRunning::new(
            &limits,
            source::CommandRunningParts {
                job: 2,
                head: Box::from([]),
                last: Box::from([]),
                dropped: 0,
                timeout: source::TimeoutClamp::Within,
                output: source::BytesClamp::Within,
                after: source::TimeoutClamp::Within,
            },
        )
        .expect("job story record fits the transcript schema"),
    );
    let messages = crate::test::list([
        source::Message::new(
            &limits,
            source::MessageParts {
                role: source::Role::Assistant,
                blocks: crate::test::list([call(b"read-job", b"job"), call(b"start-job", b"shell")]),
            },
        )
        .expect("job story record fits the transcript schema"),
        source::Message::new(
            &limits,
            source::MessageParts {
                role: source::Role::User,
                blocks: crate::test::list([result(b"read-job", ended), result(b"start-job", running)]),
            },
        )
        .expect("job story record fits the transcript schema"),
    ]);
    let usage = source::Usage::new(
        &limits,
        source::UsageParts {
            input: Some(1),
            output: Some(1),
            cache_read: Some(0),
            cache_write: Some(0),
            reasoning: None,
        },
    )
    .expect("job story record fits the transcript schema");
    let value = source::Turn::new(
        &limits,
        source::TurnParts {
            model: b"model".as_slice().into(),
            effort: None,
            kind: source::TurnKind::Ordinary,
            opening: None,
            endpoint: b"fixture".as_slice().into(),
            dialect: DIALECT,
            place: 1,
            usage,
            spent: 1,
            jobs: 1,
            messages,
        },
    )
    .expect("job story record fits the transcript schema");
    let mut writer = skein_lib::Writer::new(usize::try_from(value.measure()).expect("measured turn fits usize"));
    value.encode(&mut writer).expect("writer holds the measured job turn");
    writer.finish()
}

fn in_call(words: &[u8], tool: &[u8]) -> Find {
    Find::text(words).within(Box::new([Target::Role { role: Role::Region, name: tool.into() }]))
}

fn receiving() -> jig_client_test_domain::Limits {
    let mut receiving = crate::test::limits();
    receiving.client.domain.text = 256;
    receiving.client.domain.turn_bytes = 256;
    receiving
}

#[test]
fn a_job_reads_as_its_calls_outcome() {
    let mut world = super::world(turn());
    // Text is a whole-turn budget, including both calls and repeated result summaries.
    world.front().session.transcript_text_limit(256);
    let client =
        world.client_with(1, crate::stories::conversation(), receiving(), |seed| crate::test::new(&receiving(), seed));
    let within = Duration::from_secs(1);
    world.person(
        client,
        vec![
            Step::See { find: Find::named(Role::Button, b"Sign in with Test"), within },
            Step::Press { find: Find::named(Role::Button, b"Sign in with Test") },
            Step::See { find: Find::named(Role::TextArea, b"First words"), within },
            Step::Type { find: Find::named(Role::TextArea, b"First words"), words: b"Hello".as_slice().into() },
            Step::Press { find: Find::named(Role::Button, b"Start a conversation") },
            Step::See { find: in_call(b"Call completed", b"shell"), within },
            Step::See { find: in_call(b"Started as job 2", b"shell"), within },
            Step::See { find: in_call(b"Job 1 ended: exit 0", b"job"), within },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
    assert!(outcome.judged.0 > 0);
}
