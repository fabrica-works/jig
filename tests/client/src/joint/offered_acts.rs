//! Offered Task acts are checked against the actual admitted source packet and
//! the public card, not private domain state (client/domain/testing.md, 6).
use super::*;
use jig_fake_person::{Face, Found, TreeFace};

pub(super) fn task(world: &mut World<Test, Joint>, client: crate::ClientId, number: u64) {
    use jig_domain_core::acts::{Act, Admission};
    let (source, title) = {
        let packet = world.front().session.task_offers(client, number).expect("actual Task packet");
        assert_eq!(packet.source.len(), packet.native.len());
        (packet.source.clone(), packet.native_task.chip.title.bytes().to_vec())
    };
    let words = jig_test_client_view::connector_values_words::english(32).expect("fixture English");
    let title = words.say(
        jig_client_view::phrases::jig::JigPhrase::ChipNumber,
        &[jig_client_view::phrases::Hole::Number(number), jig_client_view::words::name(&title)],
    );
    let tree = world.tree(client);
    let card = match TreeFace::new(tree).find(&Find::exactly_named(Role::Region, &title)) {
        Found::Node(id) => id,
        Found::Absent | Found::Pending => panic!("the current source Task has a public card"),
    };
    let mut face = TreeFace::subtree(tree, card).expect("whole public Task card");
    let captions: [(Act, &[u8]); 9] = [
        (Act::Write, b"Write"),
        (Act::Amend, b"Amend"),
        (Act::TakeOver, b"Take over"),
        (Act::Close, b"Close"),
        (Act::AmendRelease, b"Amend and release"),
        (Act::Release, b"Release"),
        (Act::Stop, b"Stop"),
        (Act::Cancel, b"Cancel"),
        (Act::MarkSeen, b"Mark read"),
    ];
    assert!(source.iter().all(|offer| captions.iter().any(|(act, _)| *act == offer.act)));
    for (act, caption) in captions {
        let offered = source.iter().find(|offer| offer.act == act);
        let button = face.find(&Find::exactly_named(Role::Button, caption));
        let actionable = offered.is_some_and(|offer| match &offer.admission {
            Admission::Allowed | Admission::Proposed { .. } | Admission::Bounded { .. } => true,
            Admission::Refused(_) => false,
        });
        assert_eq!(matches!(button, Found::Node(_)), actionable, "source act {act:?}");
        if offered.is_some() {
            assert!(matches!(face.find(&Find::text(caption)), Found::Node(_)));
        }
    }
    // The card has no unrelated navigation controls; every button names one
    // of this source packet's actionable acts.
    let start = tree.nodes().iter().position(|node| node.id == card).expect("card index");
    let end = start + usize::try_from(tree.nodes()[start].size).expect("card size");
    let count = tree.nodes()[start..end].iter().filter(|node| node.role == Role::Button).count();
    let expected = source
        .iter()
        .filter(|offer| match &offer.admission {
            Admission::Allowed | Admission::Proposed { .. } | Admission::Bounded { .. } => true,
            Admission::Refused(_) => false,
        })
        .count();
    assert_eq!(count, expected);
}

pub(super) fn card(world: &mut World<Test, Joint>, client: crate::ClientId, key: client::ObjectKey, title: &[u8]) {
    use jig_domain_core::acts::{Act, Admission};
    let source = {
        let packet = world.front().session.card_offers(client, key).expect("actual admitted card packet");
        assert_eq!(packet.source.len(), packet.native.len());
        packet.source.clone()
    };
    let tree = world.tree(client);
    let card = match TreeFace::new(tree).find(&Find::exactly_named(Role::Region, title)) {
        Found::Node(id) => id,
        Found::Absent | Found::Pending => panic!("the supplied card is publicly shown: {key:?}"),
    };
    let mut face = TreeFace::subtree(tree, card).expect("whole public card");
    let captions: [(Act, &[u8]); 15] = [
        (Act::Write, b"Write"),
        (Act::Answer, b"Answer"),
        (Act::Take, b"Take it"),
        (Act::HandBack, b"Hand it back"),
        (Act::Accept, b"Accept"),
        (Act::Reject, b"Reject"),
        (Act::PassUp, b"Pass up"),
        (Act::Release, b"Release"),
        (Act::LeaveHeld, b"Leave it held"),
        (Act::Amend, b"Amend"),
        (Act::AmendRelease, b"Amend and release"),
        (Act::Cancel, b"Cancel"),
        (Act::MarkSeen, b"Mark read"),
        (Act::EditNote, b"Edit note"),
        (Act::DeleteNote, b"Delete note"),
    ];
    assert!(source.iter().all(|offer| captions.iter().any(|(act, _)| *act == offer.act)));
    for (act, caption) in captions {
        let offered = source.iter().find(|offer| offer.act == act);
        let actionable = offered.is_some_and(|offer| {
            matches!(&offer.admission, Admission::Allowed | Admission::Proposed { .. } | Admission::Bounded { .. })
        });
        assert_eq!(
            matches!(face.find(&Find::exactly_named(Role::Button, caption)), Found::Node(_)),
            actionable,
            "source act {act:?}"
        );
        if offered.is_some() {
            assert!(matches!(face.find(&Find::text(caption)), Found::Node(_)));
        }
    }
    let start = tree.nodes().iter().position(|node| node.id == card).expect("card index");
    let end = start + usize::try_from(tree.nodes()[start].size).expect("card size");
    let actual = tree.nodes()[start..end].iter().filter(|node| node.role == Role::Button).count();
    let expected = source
        .iter()
        .filter(|offer| {
            matches!(&offer.admission, Admission::Allowed | Admission::Proposed { .. } | Admission::Bounded { .. })
        })
        .count();
    assert_eq!(actual, expected, "no unoffered card buttons");
}
