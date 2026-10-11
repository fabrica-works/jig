#![expect(
    clippy::disallowed_methods,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary proofs inspect original read tokens and whole keyed requests"
)]
use super::spine::{LIMITS, delivered, drive, frame, start};
use crate::{
    Domain, Event, Request,
    action::Act,
    address::{Address, Place, ProjectSection},
    contract::*,
    pages::project::owner_form,
};
use skein_lib::{List, Time, Token, bytes::copy_of};
fn list<T>(cap: u32, rows: impl IntoIterator<Item = T>) -> List<T> {
    let mut out = List::with_capacity(cap);
    for row in rows {
        assert!(out.push(row).is_ok());
    }
    out
}
fn pattern() -> OwnerPattern {
    OwnerPattern {
        segments: list(2, [copy_of(&[255, 0, b'/']), copy_of(b"")]),
        last: OwnerPatternEnd::Open(copy_of(&[254, 0])),
    }
}
fn source(number: u32) -> OwnerProject {
    let authority = OwnerAuthorityDocument::complete(OwnerAuthority {
        tools: u64::MAX,
        grants: list(2, [OwnerGrant { connector: u16::MAX, kind: u16::MAX, pattern: pattern() }]),
        delegation: OwnerDelegation {
            kinds: list(2, [OwnerAuthorityExecutor::Role(u32::MAX)]),
            tasks: u32::MAX,
            depth: u32::MAX,
        },
        spend: u64::MAX,
        deadline: None,
        notes: u8::MAX,
        note_resources: List::with_capacity(2),
    });
    OwnerProject {
        number,
        acts: list(2, [Offered { act: ActKind::ChangePolicy, admission: Admission::Allowed }]),
        resources: List::with_capacity(2),
        policy: OwnerProjectPolicy {
            ceiling: authority.clone(),
            period_spend: u64::MAX,
            escalation_role: Some(u32::MAX),
            projections: list(2, [OwnerGrant { connector: 19, kind: 23, pattern: pattern() }]),
            roles: list(
                2,
                [OwnerProjectPolicyRole {
                    number: u32::MAX,
                    authority,
                    period_spend: 17,
                    requests: u16::MAX,
                    decides: u8::MAX,
                }],
            ),
            requirements: list(
                2,
                [PolicyRequirement {
                    connector: u16::MAX,
                    kind: u16::MAX,
                    pattern: pattern(),
                    judge: PolicyJudge { connector: u16::MAX, requirement: u16::MAX, parameters: u32::MAX },
                    guard: PolicyGuard::Guarded,
                    must_be_guarded: true,
                }],
            ),
            permissions: None,
            permissions_cut: false,
            cut: false,
        },
        members: List::with_capacity(2),
        current_period: None,
        pools: List::with_capacity(2),
        configured_period: 0,
        configured_pools: List::with_capacity(2),
        configured_pools_cut: false,
        spend: List::with_capacity(2),
        recurring: List::with_capacity(2),
        cut: false,
    }
}
fn read(out: &[Request]) -> Token {
    out.iter()
        .find_map(|r| match r {
            Request::Read { read, what: Read::Project { .. } } => Some(*read),
            _ => None,
        })
        .expect("original project read")
}
fn enter(domain: &mut Domain, number: u32) -> Box<[Request]> {
    drive(
        domain,
        Time::ZERO,
        Event::Went { place: Place::Jig(Address::Project { project: number, section: ProjectSection::Settings }) },
    )
}
fn live(number: u32) -> (Domain, Token) {
    let mut domain = Domain::new(&LIMITS, 37);
    let f = start(&mut domain, List::with_capacity(0));
    let mut value = frame(7);
    value.projects = list(2, [super::pages::project(number, false)]);
    delivered(&mut domain, Time::ZERO, f, value);
    let entered = enter(&mut domain, number);
    assert!(!entered.iter().any(|r| matches!(r, Request::Open { .. })), "settings does not acquire a Project watch");
    let token = read(&entered);
    (domain, token)
}
fn install(domain: &mut Domain, token: Token, value: OwnerProject) {
    drive(
        domain,
        Time::ZERO,
        Event::Read { read: token, result: ReadResult::Value(ReadValue::OwnerProject(Box::new(value))) },
    );
}
#[test]
fn owner_project_original_read_checks_full_identity_before_terminal_and_ignores_duplicate_late_and_reset_callbacks() {
    for number in [0, u32::MAX] {
        let (mut domain, token) = live(number);
        let value = source(number);
        let mut wrong = value.clone();
        wrong.number = if number == 0 { u32::MAX } else { 0 };
        install(&mut domain, token, wrong);
        assert!(domain.project_page().owner.is_none());
        install(&mut domain, token, value.clone());
        assert_eq!(domain.project_page().owner.as_deref(), Some(&value));
        assert!(domain.project_page().document.is_none());
        let mut duplicate = value.clone();
        duplicate.policy.period_spend = 0;
        install(&mut domain, token, duplicate.clone());
        assert_eq!(domain.project_page().owner.as_deref(), Some(&value));
        drive(&mut domain, Time::ZERO, Event::Went { place: Place::Elsewhere });
        let next = read(&enter(&mut domain, number));
        install(&mut domain, token, duplicate);
        assert!(domain.project_page().owner.is_none());
        install(&mut domain, next, value.clone());
        assert_eq!(domain.project_page().owner.as_deref(), Some(&value));
        drive(&mut domain, Time::ZERO, Event::SignedOut);
        assert!(domain.project_page().owner.is_none());
        install(&mut domain, next, value);
        assert!(domain.project_page().owner.is_none());
    }
}
#[test]
fn owner_project_whole_role_review_keeps_raw_flags_authority_and_other_edited_channels_in_one_keyed_request() {
    let (mut domain, token) = live(u32::MAX);
    let value = source(u32::MAX);
    install(&mut domain, token, value.clone());
    let role = value.policy.roles.get(0).unwrap();
    let before = Ask::ChangePolicy {
        project: value.number,
        edit: PolicyEdit::Role(PolicyEditRole {
            number: role.number,
            authority: role.authority.whole().unwrap(),
            period_spend: role.period_spend,
            requests: role.requests,
            decides: role.decides,
        }),
    };
    let mut chosen = before.clone();
    if let Ask::ChangePolicy { edit: PolicyEdit::Role(role), .. } = &mut chosen {
        role.period_spend = 0;
        role.requests = 0x8041;
    }
    assert!(drive(&mut domain, Time::ZERO, Event::Act { act: owner_form::action(chosen.clone()) }).is_empty());
    assert_eq!(domain.project_page().settings.before.as_deref(), Some(&before));
    assert_eq!(owner_form::draft(&domain.project_page().settings, &before), chosen);
    let out = drive(&mut domain, Time::ZERO, Event::Act { act: Act::Confirm });
    let Request::Send { request, scope, ask } = &out[0] else { panic!("one keyed form request") };
    assert_eq!(*scope, 7);
    assert_eq!(**ask, chosen);
    let request = *request;
    assert!(drive(&mut domain, Time::ZERO, Event::Act { act: Act::Confirm }).is_empty());
    drive(&mut domain, Time::ZERO, Event::Answered { request, answer: Answer::Done(Outcome::Changed) });
    assert!(domain.project_page().settings.pending.is_none());
    assert_eq!(domain.project_page().owner.as_deref(), Some(&value));
}
#[test]
fn owner_project_policy_scalar_and_complete_lists_are_offered_without_inferred_permissions_or_cut_replacements() {
    let mut value = source(0);
    for edit in [
        PolicyEdit::ProjectSpend { period_spend: 0 },
        PolicyEdit::Requirements { requirements: value.policy.requirements.clone() },
        PolicyEdit::Projections { grants: value.policy.projections.clone() },
    ] {
        assert!(owner_form::supported(&value, &Ask::ChangePolicy { project: 0, edit }));
    }
    let absent = Ask::ChangePolicy { project: 0, edit: PolicyEdit::Permissions { mappings: List::with_capacity(2) } };
    assert!(!owner_form::supported(&value, &absent));
    assert_eq!(owner_form::baseline(&value, &absent), None);
    value.policy.cut = true;
    assert!(!owner_form::supported(
        &value,
        &Ask::ChangePolicy { project: 0, edit: PolicyEdit::Projections { grants: value.policy.projections.clone() } }
    ));
    assert!(owner_form::supported(
        &value,
        &Ask::ChangePolicy { project: 0, edit: PolicyEdit::ProjectSpend { period_spend: 0 } }
    ));
    let (mut domain, token) = live(0);
    install(&mut domain, token, value);
    assert!(drive(&mut domain, Time::ZERO, Event::Act { act: owner_form::action(absent) }).is_empty());
    assert!(drive(&mut domain, Time::ZERO, Event::Act { act: Act::Confirm }).is_empty());
}
