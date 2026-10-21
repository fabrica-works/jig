use jig_channel_world::program_runs::{self, Case, Run};
use serde_json::json;
use smith_fake_claude::{Decision, End, Seen, Step, Story, Turn};
fn finish() -> Turn {
    Turn {
        steps: vec![],
        end: End::Success { structured_output: Some(json!({"form":"report","text":"Hello","fields":{}})) },
    }
}
fn story(turns: Vec<Turn>) -> Story {
    Story { turns, ..Story::default() }
}
fn finished(run: &Run) {
    assert!(
        run.terminals.iter().any(|answer| matches!(
            &answer.end,
            jig_domain_tasks::End::Finished {
                result: jig_domain_tasks::TaskResult::Report { words }, ..
            } if words.as_ref() == b"Hello"
        )),
        "{run:?}"
    );
}
#[test]
fn a_program_run_answers_through_claude() {
    let run = program_runs::run(Case::Answer, &story(vec![finish()]));
    assert!(run.observations.iter().any(|row| row.arguments.first().is_some_and(|arg| arg == "-p")), "{run:?}");
    assert!(!run.turns.is_empty());
    assert_eq!(run.grants, vec![(1, 1, 2, b"alice".to_vec())]);
    finished(&run);
}
fn limited(reset: Option<u64>) -> Run {
    let run = program_runs::run(
        Case::Limited,
        &story(vec![Turn { steps: vec![Step::RateLimit { resets_at: reset }], end: End::default() }]),
    );
    assert_eq!(run.limited.len(), 1, "{run:?}");
    assert_eq!(run.limited[0].account(), 1);
    assert_eq!(run.authenticated_failures, vec![1], "notice reaches the recorded core binding");
    run
}
#[test]
fn a_usage_limit_reaches_the_core_as_limited() {
    let reset = skein_sim::Config::calm().wall.as_nanos() / 1_000_000_000 + 123;
    let run = limited(Some(reset));
    let remaining = run.limited[0].reset().expect("remaining reset");
    assert!(remaining > skein_lib::Duration::from_secs(120) && remaining <= skein_lib::Duration::from_secs(123));
    assert!(matches!(run.held.as_slice(), [(1, Some(jig_domain_accounts::Held::Limited { reset: Some(_) }))]));
}
#[test]
fn a_usage_limit_without_a_reset_reaches_the_core_as_limited() {
    let run = limited(None);
    assert_eq!(*run.limited[0].reset(), None);
    assert_eq!(run.held, vec![(1, Some(jig_domain_accounts::Held::Limited { reset: None }))]);
}
#[test]
fn a_login_lost_mid_run_reaches_the_core_as_unauthorized() {
    let run = program_runs::run(
        Case::Unauthorized,
        &story(vec![Turn { steps: vec![Step::LoseLogin, Step::Text("continue".into())], end: End::default() }]),
    );
    assert_eq!(run.unauthorized.len(), 1, "{run:?}");
    assert_eq!(run.unauthorized[0].account(), 1);
    assert_eq!(run.unauthorized[0].generation(), 2);
    assert_eq!(run.authenticated_failures, vec![1], "notice crosses the core's source authentication");
    assert!(run.terminals.iter().any(|answer| matches!(answer.end, jig_domain_tasks::End::Failed(_))));
}
#[test]
fn a_source_signed_out_mid_run_ends_its_run_at_the_next_instruction() {
    let run = program_runs::run(
        Case::SignOut,
        &story(vec![Turn { steps: vec![Step::Call { tool: "wait".into(), input: json!({}) }], end: End::default() }]),
    );
    assert!(run.signed_out, "{run:?}");
    assert!(run.observations.iter().any(|row| row.arguments == ["auth", "logout"]));
    assert!(run.terminals.iter().all(|answer| answer.end == jig_domain_tasks::End::Recheck), "{run:?}");
    assert_eq!(run.unauthorized.len(), 1, "the signed-out login ends the live run");
    assert_eq!(run.authenticated_failures, vec![1]);
    assert!(
        run.observations.iter().any(|row| row.arguments.first().is_some_and(|arg| arg == "-p")),
        "Claude was running"
    );
}
#[test]
fn a_waiting_program_run_resumes_on_its_source() {
    let run = program_runs::run(
        Case::Waiting,
        &story(vec![
            Turn { steps: vec![Step::Call { tool: "wait".into(), input: json!({}) }], end: End::default() },
            finish(),
        ]),
    );
    assert!(run.terminals.iter().any(|answer| answer.end == jig_domain_tasks::End::Parked), "{run:?}");
    assert_eq!(run.starts, 2);
    assert_eq!(run.grants.len(), 2);
    assert_eq!(&run.grants[0].1, &run.grants[1].1);
    assert_eq!((&run.grants[0].2, &run.grants[0].3), (&run.grants[1].2, &run.grants[1].3));
    assert!(
        run.observations.iter().any(|row| row.arguments.iter().any(|arg| arg == "--resume")),
        "the committed conversation resumes"
    );
    assert!(run.received.iter().any(|line| line.to_string().contains("After waiting")));
    finished(&run);
}
#[test]
fn a_granted_built_in_call_asks_smith_s_policy() {
    let mut turn = finish();
    turn.steps = vec![Step::Builtin {
        tool: "Read".into(),
        input: json!({"file_path":"/etc/passwd"}),
        result: "must not run".into(),
    }];
    let run = program_runs::run(Case::Builtin, &story(vec![turn]));
    assert!(run.seen.iter().any(|seen| matches!(seen,Seen::Settings { ask,.. } if ask.iter().any(|tool|tool=="Read"))));
    assert!(
        run.seen.iter().any(|seen| matches!(
            seen,
            Seen::Prompted { tool, decision: Decision::Deny(reason) } if tool == "Read" && !reason.is_empty()
        )),
        "granted family still asks policy: {run:?}"
    );
    let mut told = false;
    for turn in &run.turns {
        let transcript =
            smith_transcript::Turn::decode(&smith_transcript::CEILINGS, &mut skein_lib::Reader::new(&turn.transcript))
                .expect("committed transcript");
        for message in transcript.messages().as_slice() {
            told |= message
                .blocks()
                .iter()
                .any(|block| matches!(block,smith_transcript::Block::Call(call) if call.name()==b"Read"));
        }
    }
    assert!(told, "the core's transcript shows the call");
    finished(&run);
}
