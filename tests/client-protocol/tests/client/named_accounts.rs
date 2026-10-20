//! Named preferences through public client records. Payer/coverage matrices stay below this world.
use super::{PROOF_WAIT, Story, fixtures};
use jig_client::{Answered, Ask, Asker, Handle, Key, Latest, Outcome, Watchable, contract as c};
use skein_lib::{List, sha256};
const ACCOUNTS: Handle = Handle { asker: Asker::JIG, number: 30 };
const TASK: Handle = Handle { asker: Asker::JIG, number: 31 };
const FIRST_KEY: Key = Key { asker: Asker::JIG, number: 30 };
const SECOND_KEY: Key = Key { asker: Asker::JIG, number: 31 };
const CREATE: Key = Key { asker: Asker::JIG, number: 32 };
const AMEND: Key = Key { asker: Asker::JIG, number: 33 };
const RESUME: Key = Key { asker: Asker::JIG, number: 34 };
const STOP: Key = Key { asker: Asker::JIG, number: 35 };
const RELEASE: Key = Key { asker: Asker::JIG, number: 36 };
const MOVE: Key = Key { asker: Asker::JIG, number: 37 };
const USES: Key = Key { asker: Asker::JIG, number: 38 };
fn text(bytes: &[u8]) -> c::Text {
    c::Text::new(bytes.into(), 64).expect("bounded account story fixture")
}
fn answer(story: &Story, key: Key) -> c::Answer {
    let answers = story.answered(key);
    let [Outcome::Answered(Answered::Jig(answer))] = answers.as_slice() else { panic!("one answer: {answers:?}") };
    answer.clone()
}
fn accounts(story: &Story) -> Option<&c::OwnerAccounts> {
    let Some(Latest::Jig(c::Delivery::OwnerAccounts(value))) = &story.client.document(ACCOUNTS)?.latest else {
        return None;
    };
    Some(value)
}
fn task(story: &Story) -> Option<&c::Task> {
    let Some(Latest::Jig(c::Delivery::Task(value))) = &story.client.document(TASK)?.latest else { return None };
    Some(value)
}
fn add(story: &mut Story, key: Key, name: u8, value: &[u8]) -> (u32, u32) {
    story.carry(
        key,
        [name; 16],
        Ask::Jig(Box::new(c::Ask::AccountChange {
            holder: c::AccountHolder::Own,
            change: c::AccountChange::AddKey { provider: 1, join: None, digest: sha256::digest(value) },
        })),
        value,
    );
    assert!(story.run_until(|s| !s.answered(key).is_empty(), PROOF_WAIT));
    let c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::AccountKeyKept { source })) = answer(story, key) else {
        panic!("key kept")
    };
    assert!(story.run_until(
        |s| accounts(s).is_some_and(|d| d.accounts.iter().any(|a| a.sources.iter().any(|s| s.number == source))),
        PROOF_WAIT
    ));
    let account = accounts(story)
        .expect("bounded account story fixture")
        .accounts
        .iter()
        .find(|a| a.sources.iter().any(|s| s.number == source))
        .expect("bounded account story fixture")
        .number;
    (account, source)
}
fn choice(account: u32) -> c::LlmChoice {
    c::LlmChoice {
        account: Some(account),
        main: c::ModelEffort { model: text(b"scripted"), effort: Some(text(b"high")) },
        subagents: List::with_capacity(0),
    }
}
fn prepared() -> (Story, u64, (u32, u32), (u32, u32)) {
    let mut story = Story::named_accounts(fixtures::SEED);
    story.watch(ACCOUNTS, Watchable::Jig(c::Subject::Accounts));
    let first = add(&mut story, FIRST_KEY, 61, b"first LLM key");
    let second = add(&mut story, SECOND_KEY, 62, b"second LLM key");
    let mut tasks = fixtures::create_tasks();
    tasks.get_mut(0).expect("bounded account story fixture").llm = Some(choice(first.0));
    story.send(CREATE, Ask::Jig(Box::new(c::Ask::Create { project: 1, tasks })));
    assert!(story.run_until(|s| !s.answered(CREATE).is_empty(), PROOF_WAIT));
    let c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::Created { first: number, .. })) = answer(&story, CREATE)
    else {
        panic!("created")
    };
    story.watch(TASK, Watchable::Jig(c::Subject::Task { task: number }));
    assert!(story.run_until(|s| task(s).is_some_and(|t| t.kept_account == Some(first.0)), PROOF_WAIT));
    (story, number, first, second)
}
fn amend(story: &mut Story, number: u64, account: u32) {
    let revision = task(story).expect("bounded account story fixture").spec_revision;
    story.send(
        AMEND,
        Ask::Jig(Box::new(c::Ask::Amend {
            project: 1,
            task: number,
            revision: Some(revision),
            change: Box::new(c::Amendment {
                addressed: None,
                spec: None,
                wake: None,
                dependencies: None,
                authority: None,
                instructions: None,
                tracked: None,
                llm: Some(choice(account)),
            }),
            reason: text(b"use the chosen account"),
        })),
    );
    assert!(
        story.run_until(|s| !s.answered(AMEND).is_empty(), PROOF_WAIT),
        "phase {:?}, document {:?}",
        story.phase(number),
        task(story)
    );
}
#[test]
fn a_client_creates_with_an_explicit_account() {
    let (story, _, first, _) = prepared();
    let task = task(&story).expect("bounded account story fixture");
    assert_eq!(task.llm.as_ref().expect("bounded account story fixture").account, Some(first.0));
    assert_eq!(task.kept_account, Some(first.0));
}
#[test]
fn a_client_cannot_switch_a_limited_kept_account() {
    let (mut story, number, first, second) = prepared();
    story.limit_account_source(first.1);
    amend(&mut story, number, second.0);
    let c::Answer::Refused { refusal, .. } = answer(&story, AMEND) else { panic!("refused") };
    assert_eq!(refusal, c::Refusal::Account { account: second.0, why: c::AccountWhy::Limited });
    assert_eq!(task(&story).expect("bounded account story fixture").kept_account, Some(first.0));
}
#[test]
fn a_client_can_switch_after_resuming_the_kept_source() {
    let (mut story, number, first, second) = prepared();
    story.limit_account_source(first.1);
    story.send(
        RESUME,
        Ask::Jig(Box::new(c::Ask::AccountChange {
            holder: c::AccountHolder::Own,
            change: c::AccountChange::Resume { source: first.1 },
        })),
    );
    assert!(story.run_until(|s| !s.answered(RESUME).is_empty(), PROOF_WAIT));
    assert!(matches!(answer(&story, RESUME), c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::AccountChanged))));
    amend(&mut story, number, second.0);
    assert!(matches!(answer(&story, AMEND), c::Answer::Done(_)));
    assert!(story.run_until(
        |s| {
            task(s).is_some_and(|t| {
                t.llm.as_ref().is_some_and(|c| c.account == Some(second.0)) && t.kept_account.is_none()
            })
        },
        PROOF_WAIT
    ));
}

#[test]
fn a_move_rechooses_after_its_financial_payer_changes() {
    const PROJECT_PAID: u64 = 100;
    const STOP_PROJECT: Key = Key { asker: Asker::JIG, number: 39 };
    let (mut story, original, first, _) = prepared();
    let mut uses = List::with_capacity(1);
    uses.push(c::OwnerAccountUse::Project(1)).expect("bounded account story fixture");
    story.send(
        USES,
        Ask::Jig(Box::new(c::Ask::AccountChange {
            holder: c::AccountHolder::Deployment,
            change: c::AccountChange::SetUses { account: 0, uses: Some(uses.into_boxed()) },
        })),
    );
    assert!(story.run_until(|s| !s.answered(USES).is_empty(), PROOF_WAIT));
    assert!(matches!(answer(&story, USES), c::Answer::Done(_)));
    story.send(STOP, Ask::Jig(Box::new(c::Ask::Stop { project: 1, task: original })));
    assert!(
        story.run_until(|s| s.phase(original) == Some(c::Phase::Held), PROOF_WAIT),
        "stop {:?}, phase {:?}, document {:?}",
        story.answered(STOP),
        story.phase(original),
        task(&story)
    );
    story.project_paid_copy(original, PROJECT_PAID);
    story.watch(TASK, Watchable::Jig(c::Subject::Task { task: PROJECT_PAID }));
    assert!(
        story.run_until(
            |s| task(s).is_some_and(|t| t.kept_account == Some(0)) && s.phase(PROJECT_PAID) == Some(c::Phase::Running),
            PROOF_WAIT
        ),
        "project phase {:?}, document {:?}",
        story.phase(PROJECT_PAID),
        task(&story)
    );
    story.send(
        MOVE,
        Ask::Jig(Box::new(c::Ask::Move { project: 1, task: PROJECT_PAID, reason: text(b"pay from my pool") })),
    );
    assert!(story.run_until(|s| !s.answered(MOVE).is_empty(), PROOF_WAIT));
    assert!(matches!(answer(&story, MOVE), c::Answer::Done(_)));
    story.send(STOP_PROJECT, Ask::Jig(Box::new(c::Ask::Stop { project: 1, task: PROJECT_PAID })));
    assert!(story.run_until(|s| s.phase(PROJECT_PAID) == Some(c::Phase::Held), PROOF_WAIT));
    story.send(
        RELEASE,
        Ask::Jig(Box::new(c::Ask::Release {
            project: 1,
            task: PROJECT_PAID,
            revision: story.hold_revision(PROJECT_PAID),
            again: None,
        })),
    );
    assert!(story.run_until(|s| !s.answered(RELEASE).is_empty(), PROOF_WAIT));
    assert!(matches!(answer(&story, RELEASE), c::Answer::Done(_)));
    assert!(story.run_until(|s| task(s).is_some_and(|t| t.kept_account == Some(first.0)), PROOF_WAIT));
}

#[test]
fn a_task_watch_tracks_an_unknown_account_wait() {
    let (mut story, number, first, _) = prepared();
    story.send(STOP, Ask::Jig(Box::new(c::Ask::Stop { project: 1, task: number })));
    assert!(story.run_until(|s| s.phase(number) == Some(c::Phase::Held), PROOF_WAIT));
    story.limit_account_source(first.1);
    story.send(
        RELEASE,
        Ask::Jig(Box::new(c::Ask::Release {
            project: 1,
            task: number,
            revision: story.hold_revision(number),
            again: None,
        })),
    );
    assert!(story.run_until(
        |s| task(s).is_some_and(|t| t.run_wait
            == Some(c::TaskRunWait::Account {
                account: Some(first.0),
                source: Some(first.1),
                why: c::TaskWaitWhy::Source(c::TaskSourceState::Limited { until: None }),
            })),
        PROOF_WAIT
    ));
    story.send(
        RESUME,
        Ask::Jig(Box::new(c::Ask::AccountChange {
            holder: c::AccountHolder::Own,
            change: c::AccountChange::Resume { source: first.1 },
        })),
    );
    assert!(story.run_until(
        |s| !s.answered(RESUME).is_empty()
            && task(s).is_some_and(|t| t.run_wait.is_none() && t.kept_account == Some(first.0))
            && s.phase(number) == Some(c::Phase::Running),
        PROOF_WAIT
    ));
}
