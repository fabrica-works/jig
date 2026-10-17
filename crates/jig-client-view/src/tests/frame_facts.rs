#![expect(clippy::disallowed_methods, reason = "ordinary tests inspect public trees and exact supplied source values")]
use super::cards::{CLIENT, contains, document, drive, limits, start, text, words};
use crate::tests::fixtures::{Fixture, Watched};
use crate::{JigBindings, Owner, phrases::jig::JigPhrase};
use skein_ui::{Builder, Role, SlotRef, Tree};

use crate::Event;
use jig_client::contract::*;
use skein_lib::{List, Token, Wall};
fn tree(domain: &Fixture) -> Tree {
    frame_tree(domain, words())
}
fn frame_tree(domain: &Fixture, words: &crate::words::Words) -> Tree {
    let mut sized = limits();
    sized.nodes = crate::banner_nodes(&CLIENT)
        .unwrap()
        .checked_add(crate::navigation_nodes(&CLIENT).unwrap())
        .unwrap()
        .checked_add(1)
        .unwrap();
    let mut builder = Builder::new(&sized);
    let mut bindings = JigBindings::new(sized.bindings, CLIENT.text);
    builder.open(Role::Page);
    crate::frame::banner(&mut builder, &domain.view(), words, &mut bindings);
    crate::frame::navigation(
        &mut builder,
        &domain.view(),
        SlotRef { owner: (Owner::Application { number: 1 }).opaque(), token: Token::new(1) },
        words,
        &mut bindings,
    );
    builder.close();
    let tree = builder.finish();
    assert!(tree.len() <= sized.nodes);
    super::cards_access::check(&tree);
    tree
}
fn ready(source: Me) -> Fixture {
    let (mut domain, watch) = start();
    drive(&mut domain, Watched { watch, event: WatchEvent::Delivered(Delivery::Me(Box::new(source))) });
    domain
}
#[test]
fn frame_tree_keeps_absent_empty_and_partial_names_separate_from_identity_loss() {
    for name in
        [None, Some(DocumentText::complete(text(b""))), Some(DocumentText { bytes: text(b"partial-name"), cut: true })]
    {
        let mut source = document();
        source.party.kind = PartyKind::Service;
        source.party.login = DocumentText { bytes: text(b"supplied-login"), cut: true };
        source.party.name = DocumentText::complete(text(b"supplied-name"));
        source.projects.get_mut(0).unwrap().name = name.clone();
        source.projects.get_mut(0).unwrap().pool = None;
        let domain = ready(source.clone());
        let shown = tree(&domain);
        assert!(contains(&shown, b"Login: supplied-login"));
        assert!(contains(&shown, b"supplied-name"));
        assert!(contains(&shown, b"Service"));
        assert!(contains(&shown, b"Pool ledger not supplied"));
        let expected = match name {
            None => b"Project 4".as_slice(),
            Some(ref value) if value.bytes().is_empty() => b"Project: ".as_slice(),
            Some(_) => b"Project: partial-name".as_slice(),
        };
        assert!(shown.nodes().iter().any(|node| node.role == Role::Button && node.name.as_deref() == Some(expected)));
        assert_eq!(
            shown
                .nodes()
                .iter()
                .filter(|node| node.text.iter().any(|run| run.text.as_ref() == b"Only part of this text was supplied"))
                .count(),
            if name.as_ref().is_some_and(|value| value.cut) { 2 } else { 1 }
        );
        assert_eq!(domain.frame().document, Some(&source));
    }
}
#[test]
fn frame_tree_uses_supplied_totals_goal_list_loss_and_all_pool_ledger_facts() {
    let mut source = document();
    source.needs_total = 17;
    source.results_seen = u64::MAX;
    source.cut = true;
    source.charters.get_mut(0).unwrap().project = u32::MAX;
    source.charters.get_mut(0).unwrap().cut = true;
    source.projects.get_mut(0).unwrap().role = HeldRole::Policy { role: u32::MAX };
    let ledger = PoolLedger {
        source: Funder::Pool { project: u32::MAX, person: u64::MAX, period: u64::MAX },
        parent: Some(Funder::Period { project: 18, period: 19 }),
        numbers: OwnerNumbers { budget: 101, spent: 7, spent_below: 11, reserved: 13 },
        left: 17,
        closed: true,
        began: Some(Wall::from_nanos(60_000_000_000)),
        next: Some(Wall::from_nanos(120_000_000_000)),
    };
    source.projects.get_mut(0).unwrap().pool = Some(ledger.clone());
    let domain = ready(source.clone());
    let shown = tree(&domain);
    for bytes in [
        b"17 need you".as_slice(),
        b"Questions: 1; proposals: 2; escalations: 3; person tasks: 4",
        b"Results read through 18446744073709551615",
        b"More project memberships than shown",
        b"More goal charters in project 4294967295",
        b"Budget: 101",
        b"Spent directly: 7",
        b"Settled spend below: 11",
        b"Reserved for funded work: 13",
        b"Left: 17",
    ] {
        assert!(contains(&shown, bytes), "missing {bytes:?}");
    }
    assert!(!contains(&shown, b"Pool ledger not supplied"));
    assert_eq!(domain.frame().document.unwrap().projects.get(0).unwrap().pool.as_ref(), Some(&ledger));
    source.projects.get_mut(0).unwrap().pool.as_mut().unwrap().left = 0;
    source.projects.get_mut(0).unwrap().pool.as_mut().unwrap().closed = false;
    let zero = tree(&ready(source));
    assert!(contains(&zero, b"Left: 0"));
    assert!(!contains(&zero, b"Pool ledger not supplied"));
}
#[test]
fn chat_forms_join_membership_only_by_supplied_project_and_preserve_optional_charter_names() {
    let mut source = document();
    let mut chats = List::with_capacity(2);
    chats.push(Charter { number: u32::MAX, name: None }).unwrap();
    chats.push(Charter { number: 0, name: Some(DocumentText { bytes: text(b"actual-charter"), cut: true }) }).unwrap();
    source.charters.get_mut(0).unwrap().chats = chats;
    source.projects.get_mut(0).unwrap().chat_budget = 123;
    let mut domain = ready(source.clone());
    drive(
        &mut domain,
        Event::Went { place: crate::address::Place::Jig(crate::address::Address::Conversations { project: None }) },
    );
    let shown = super::pages::build(&domain, 1, words()).0;
    for bytes in
        [b"Charter 4294967295".as_slice(), b"actual-charter", b"Budget 123", b"Only part of this text was supplied"]
    {
        assert!(contains(&shown, bytes));
    }
    source.charters.get_mut(0).unwrap().project = 9;
    let mut mismatched = ready(source);
    drive(
        &mut mismatched,
        Event::Went { place: crate::address::Place::Jig(crate::address::Address::Conversations { project: None }) },
    );
    let shown = super::pages::build(&mismatched, 1, words()).0;
    assert!(!contains(&shown, b"Budget 123"));
    assert!(!contains(&shown, b"actual-charter"));
}

#[test]
fn a_service_frame_offers_no_sign_out() {
    let mut domain = ready(sign_out_frame(PartyKind::Service));
    let shown = sign_out_tree(&domain);
    assert!(!offers_sign_out(&shown));
    let out = choose_sign_out(&mut domain);
    assert!(out.is_empty());
}

#[test]
fn choosing_sign_out_keeps_the_frame_until_the_shell_answers() {
    let source = sign_out_frame(PartyKind::Person);
    let mut domain = ready(source.clone());
    let out = choose_sign_out(&mut domain);
    assert_eq!(out.as_ref(), &[crate::Request::Client { asked: jig_client::Event::SignOut }]);
    assert_eq!(domain.frame().document, Some(&source));
    assert!(offers_sign_out(&sign_out_tree(&domain)));
    drive(&mut domain, jig_client::Up::SignedOut);
    assert!(domain.frame().document.is_none());
    assert_eq!(domain.page(), &crate::page::Page::SigningIn);
}

fn offers_sign_out(shown: &Tree) -> bool {
    shown.nodes().iter().any(|node| node.role == Role::Button && node.name.as_deref() == Some(b"Sign out"))
}
fn choose_sign_out(domain: &mut Fixture) -> Box<[crate::Request]> {
    drive(domain, Event::Act { action: crate::action::Act::SignOut }).into_boxed_slice()
}

fn sign_out_frame(kind: PartyKind) -> Me {
    let mut source = document();
    source.party.kind = kind;
    source.projects = List::with_capacity(0);
    source.charters = List::with_capacity(0);
    source.updates = Count { value: 0, limited: false };
    source
}
const SIGN_OUT_PHRASES: [JigPhrase; 21] = [
    JigPhrase::Me,
    JigPhrase::NothingKnown,
    JigPhrase::LinkPart,
    JigPhrase::NeedsPart,
    JigPhrase::PartyPart,
    JigPhrase::ProjectsPart,
    JigPhrase::GoToTaskPart,
    JigPhrase::Starting,
    JigPhrase::Live,
    JigPhrase::SignedIn,
    JigPhrase::FramePerson,
    JigPhrase::FrameService,
    JigPhrase::FrameLogin,
    JigPhrase::FrameNeedsKinds,
    JigPhrase::FrameResultsSeen,
    JigPhrase::SignOut,
    JigPhrase::Updates,
    JigPhrase::Needs,
    JigPhrase::TaskNumber,
    JigPhrase::Navigation,
    JigPhrase::LoadingNavigation,
];

fn sign_out_tree(domain: &Fixture) -> Tree {
    frame_tree(domain, &sign_out_words())
}
fn sign_out_words() -> crate::words::Words {
    use skein_ui::phrases::{PhraseLimits, Phrasebook};
    let count = u32::try_from(SIGN_OUT_PHRASES.len()).unwrap();
    let mut shapes = List::with_capacity(count);
    for phrase in SIGN_OUT_PHRASES {
        shapes.push(phrase.shape(CLIENT.text)).unwrap();
    }
    let mut language = crate::phrases::english::language();
    let mut phrases = List::with_capacity(count);
    for phrase in language.phrases {
        if SIGN_OUT_PHRASES.iter().any(|wanted| wanted.id() == phrase.id) {
            phrases.push(phrase).unwrap();
        }
    }
    language.phrases = phrases.into_boxed();
    let book = Phrasebook::load(
        shapes.into_boxed(),
        Box::new([language]),
        &PhraseLimits {
            languages: 1,
            phrases: count,
            holes: 4,
            segments: 128,
            text_bytes: 2048,
            hole_bytes: CLIENT.text,
        },
    )
    .unwrap();
    crate::words::Words { book, language: 0, offset_minutes: 60 }
}

#[test]
fn the_banner_holds_link_needs_party_projects_and_task_in_order() {
    let shown = tree(&ready(document()));
    assert_eq!(shown.get(1).unwrap().role, Role::Banner);
    let mut regions = List::with_capacity(5);
    let mut at = 2;
    let end = shown.get(1).unwrap().size + 1;
    while at < end {
        let node = shown.get(at).unwrap();
        assert_eq!(node.role, Role::Region);
        let region = node.region.unwrap();
        assert_eq!(region.owner, Owner::Jig.opaque());
        regions.push(crate::regions::JigRegion::of(region.number).unwrap().name()).unwrap();
        at += node.size;
    }
    assert_eq!(
        regions.as_slice(),
        [b"jig-link".as_slice(), b"jig-needs", b"jig-party", b"jig-projects", b"jig-go-to-task"]
    );
}

#[test]
fn the_navigation_landmark_ends_with_the_applications_slot() {
    let shown = tree(&ready(document()));
    let navigation = shown.nodes().iter().position(|node| node.role == Role::Navigation).unwrap();
    let node = &shown.nodes()[navigation];
    assert_eq!(node.region, Some(crate::regions::JigRegion::Navigation.region()));
    let mut at = navigation + 1;
    let end = navigation + usize::try_from(node.size).unwrap();
    let mut last = &shown.nodes()[at];
    while at < end {
        last = &shown.nodes()[at];
        at += usize::try_from(last.size).unwrap();
    }
    assert_eq!(last.role, Role::Slot);
    assert_eq!(last.slot, Some(SlotRef { owner: (Owner::Application { number: 1 }).opaque(), token: Token::new(1) }));
}

#[test]
fn the_needs_part_says_nothing_is_known_before_the_frame() {
    let (domain, _) = start();
    let shown = tree(&domain);
    let at =
        shown.nodes().iter().position(|node| node.region == Some(crate::regions::JigRegion::Needs.region())).unwrap();
    assert_eq!(shown.nodes()[at + 1].role, Role::Status);
    assert_eq!(shown.nodes()[at + 1].name.as_deref(), Some(b"Nothing known yet".as_slice()));
}
