const LOCAL_WORDS: jig_client::Key = jig_client::Key { asker: jig_client::Asker::JIG, number: 1 };
use super::spine::{LIMITS, delivered, frame, start};
use crate::tests::fixtures::Fixture;
use crate::{
    address::Address,
    page::navigation::{Destination, Navigation},
};
use jig_client::contract::*;
use skein_lib::{List, Time};
fn text(bytes: &[u8]) -> DocumentText {
    DocumentText::complete(Text::new(bytes.into(), 32).unwrap())
}
fn member(role: HeldRole) -> MeProject {
    MeProject {
        templates: List::with_capacity(0),
        creating: None,
        project: u32::MAX,
        name: None,
        role,
        pool: None,
        models: List::with_capacity(0),
    }
}
#[test]
fn me_preserves_roles_pools_and_independent_owner_counts() {
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
        let mut domain = Fixture::new(&LIMITS);
        let watch = start(&mut domain, List::with_capacity(0));
        delivered(&mut domain, Time::ZERO, watch, source.clone());
        assert_eq!(domain.frame().document, Some(&source));
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
fn pending_words_keep_the_local_name_loss_without_inventing_historical_loss_or_blocking_send() {
    let mut words = jig_client::thread::words::KeptWords::new(3);
    let mut author = frame(7).party;
    author.name = DocumentText { bytes: Text::new(Box::from(*b"partial"), 32).unwrap(), cut: true };
    assert!(words.begin_frame(LOCAL_WORDS, Some(author.clone()), Text::new(Box::from(*b"send"), 32).unwrap(), 0));
    let local = words.rows.get(0).unwrap();
    assert_eq!(local.by.as_ref().unwrap().number, 7);
    assert_eq!(local.by.as_ref().unwrap().name.bytes(), b"partial");
    assert_eq!(local.by_name_cut, Some(true));
    author.name.cut = false;
    assert!(words.begin_frame(LOCAL_WORDS, Some(author), Text::new(Box::from(*b"later"), 32).unwrap(), 0));
    assert_eq!(words.rows.get(0).unwrap().by_name_cut, Some(true));
    assert_eq!(words.rows.get(0).unwrap().words.bytes(), b"send");
    assert!(words.restore(super::spine::party(8), 2, Text::new(Box::from(*b"history"), 32).unwrap()));
    assert_eq!(words.rows.get(1).unwrap().by_name_cut, None);
}
