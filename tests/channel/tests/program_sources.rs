use jig_channel_world::program_sources::{self, SignOutAnswer};
use jig_domain_fleet::SignOutResult;
use jig_domain_host::{LoginMethod, LoginState, SourceFailure};
use smith_fake_claude::{Logout, Status, Story};
#[test]
fn auth_status_reaches_the_engine_through_the_worker_channel() {
    for (login, status, expected) in [
        (Some(b"claude.ai".as_slice()), Status::Truthful, LoginState::Usable),
        (None, Status::Truthful, LoginState::NotSignedIn),
        (
            Some(b"api-key".as_slice()),
            Status::Truthful,
            LoginState::SignedInOtherwise(LoginMethod::new(b"api-key").expect("method")),
        ),
        (None, Status::Unreadable, LoginState::Failed { reason: SourceFailure::Unreadable, detail: Box::new([]) }),
    ] {
        let result = program_sources::run(login, &Story { status, ..Story::default() }, false, false);
        assert_eq!(result.states.first(), Some(&expected));
        assert_eq!(result.observations.len(), 1);
        assert_eq!(result.observations[0].arguments, ["auth", "status"]);
        assert!(result.observations[0].environment.contains(&"CLAUDE_CONFIG_DIR".into()));
        assert!(
            !result.observations[0]
                .environment
                .iter()
                .any(|name| name.starts_with("ANTHROPIC_") || name == "CLAUDE_CODE_OAUTH_TOKEN")
        );
    }
}
#[test]
fn logout_success_and_bounded_failure_reach_the_engine() {
    for logout in [Logout::Succeeds, Logout::Fails] {
        let succeeds = logout == Logout::Succeeds;
        let result = program_sources::run(Some(b"claude.ai"), &Story { logout, ..Story::default() }, true, false);
        if succeeds {
            assert_eq!(result.ended, Some(SignOutAnswer::Answered(SignOutResult::SignedOut)));
        } else {
            assert_eq!(
                result.ended,
                Some(SignOutAnswer::Answered(SignOutResult::Failed {
                    reason: jig_domain_fleet::SourceFailure::Exited,
                    detail: b"refused\n".as_slice().into()
                }))
            );
        }
        assert_eq!(result.observations.iter().filter(|row| row.arguments == ["auth", "logout"]).count(), 1);
    }
}
#[test]
fn a_lost_real_logout_reply_ends_unconfirmed_and_redial_only_checks() {
    let result = program_sources::run(Some(b"claude.ai"), &Story::default(), true, true);
    assert_eq!(result.ended, Some(SignOutAnswer::Unconfirmed));
    assert_eq!(result.states.last(), Some(&LoginState::NotSignedIn));
    assert_eq!(result.observations.iter().filter(|row| row.arguments == ["auth", "logout"]).count(), 1);
}

#[test]
fn an_unconfigured_worker_reports_failure_without_starting_a_program() {
    let result = program_sources::unconfigured();
    assert_eq!(
        result.states,
        [LoginState::Failed {
            reason: SourceFailure::Unconfigured,
            detail: b"no program configured".as_slice().into()
        }]
    );
    assert!(result.observations.is_empty());
}

#[test]
fn a_fake_login_then_check_publishes_the_fresh_status() {
    let result = program_sources::login_then_check();
    assert_eq!(result.states, [LoginState::NotSignedIn, LoginState::Usable]);
    assert_eq!(
        result.observations.iter().map(|row| row.arguments.clone()).collect::<Vec<_>>(),
        [["auth", "status"], ["auth", "login"], ["auth", "status"]]
    );
}

#[test]
fn sources_after_a_redial_are_checked_again_without_a_sign_out() {
    let result = program_sources::run(Some(b"claude.ai"), &Story::default(), false, true);
    assert_eq!(result.states, [LoginState::Usable, LoginState::Usable]);
    assert_eq!(result.observations.len(), 2);
    assert!(result.observations.iter().all(|row| row.arguments == ["auth", "status"]));
}
