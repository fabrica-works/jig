use super::spine::{LIMITS, delivered, frame, start};
use crate::{
    Domain,
    address::Address,
    contract::*,
    pages::navigation::{Destination, Navigation},
};
use skein_lib::{List, Time};
fn text(bytes: &[u8]) -> DocumentText {
    DocumentText::complete(Text::new(bytes.into(), 32).unwrap())
}
fn member(role: HeldRole) -> ProjectRole {
    ProjectRole { project: u32::MAX, name: None, role, pool: None, chat_budget: u64::MAX }
}
#[test]
fn frame_delivery_keeps_exact_roles_absent_pools_and_independent_owner_counts() {
    let roles = [
        HeldRole::Owner,
        HeldRole::Maintainer,
        HeldRole::Member,
        HeldRole::Observer,
        HeldRole::Policy { role: u32::MAX },
    ];
    for role in roles {
        let mut source = frame(u64::MAX);
        source.party.login = DocumentText { bytes: Text::new(Box::from(*b"partial-login"), 32).unwrap(), cut: true };
        source.party.name = text(b"");
        source.projects = List::with_capacity(1);
        source.projects.push(member(role)).unwrap();
        source.needs =
            Needs { questions: u32::MAX, proposals: u32::MAX, escalations: u32::MAX, person_tasks: u32::MAX };
        source.needs_total = 3;
        source.results_seen = u64::MAX;
        source.cut = true;
        source.deployment_role = None;
        let mut domain = Domain::new(&LIMITS, 1);
        let watch = start(&mut domain, List::with_capacity(0));
        delivered(&mut domain, Time::ZERO, watch, source.clone());
        assert_eq!(domain.frame().document.as_ref(), Some(&source));
        let summary = domain.frame_summary();
        assert_eq!(summary.needs, u64::from(u32::MAX) * 4);
        assert_eq!(summary.needs_total, 3);
        let nav = Navigation::new(0);
        assert!(nav.offered(
            Destination::Jig(Address::Project { project: u32::MAX, section: crate::address::ProjectSection::Overview }),
            Some(&source)
        ));
        assert!(!nav.offered(Destination::Jig(Address::Deployment), Some(&source)));
    }
}
#[test]
fn charter_offers_keep_their_own_project_and_reserved_capacity_is_bounded() {
    let mut source = frame(7);
    source.projects = List::with_capacity(1);
    source.projects.push(member(HeldRole::Observer)).unwrap();
    source.charters = List::with_capacity(1);
    let mut chats = List::with_capacity(1);
    chats.push(Charter { number: u32::MAX, name: Some(text(b"")) }).unwrap();
    source.charters.push(OfferedCharters { project: 17, chats, goals: List::with_capacity(1), cut: true }).unwrap();
    let nav = Navigation::new(0);
    assert!(nav.offered(Destination::Jig(Address::Conversations { project: Some(17) }), Some(&source)));
    assert!(!nav.offered(Destination::Jig(Address::Conversations { project: Some(u32::MAX) }), Some(&source)));
    assert!(crate::frame::fits(&source, &LIMITS));
    source.projects = List::with_capacity(LIMITS.projects + 1);
    assert!(!crate::frame::fits(&source, &LIMITS));
    source.projects = List::with_capacity(0);
    source.charters.get_mut(0).unwrap().goals = List::with_capacity(LIMITS.charters + 1);
    assert!(!crate::frame::fits(&source, &LIMITS));
    source.charters = List::with_capacity(0);
    source.party.login = DocumentText::complete(
        Text::new(vec![0; LIMITS.text as usize + 1].into_boxed_slice(), LIMITS.text + 1).unwrap(),
    );
    assert!(!crate::frame::fits(&source, &LIMITS));
}
