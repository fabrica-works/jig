use crate::{
    Builder, Modifier, Role, Shortcut, Tree,
    outline::{Fault, Rule, check},
    regions::JigRegion,
};

fn page() -> Builder {
    let mut b = Builder::new(&super::tree::LIMITS);
    b.open(Role::Page);
    b
}
fn finish(mut b: Builder) -> Tree {
    b.close();
    b.finish()
}
fn banner(b: &mut Builder) {
    b.open(Role::Banner);
    for region in [JigRegion::Link, JigRegion::Needs, JigRegion::Party] {
        b.open(Role::Region);
        b.region(region.region());
        b.close();
    }
    b.close();
}
#[test]
fn a_page_keeping_its_outline_has_no_fault() {
    let mut b = page();
    banner(&mut b);
    for role in [Role::Navigation, Role::Main, Role::Complementary, Role::ContentInfo] {
        b.open(role);
        b.close();
    }
    assert_eq!(check(&finish(b)), None);
}
#[test]
fn a_page_without_its_banner_first_is_a_fault() {
    let mut b = page();
    b.open(Role::Main);
    b.close();
    assert_eq!(check(&finish(b)), Some(Fault { rule: Rule::BannerFirst, node: 1 }));
}
#[test]
fn a_banner_without_the_party_region_is_a_fault() {
    let mut b = page();
    b.open(Role::Banner);
    for region in [JigRegion::Link, JigRegion::Needs] {
        b.open(Role::Region);
        b.region(region.region());
        b.close();
    }
    b.close();
    b.open(Role::Main);
    b.close();
    assert_eq!(check(&finish(b)), Some(Fault { rule: Rule::BannerHolds { region: JigRegion::Party }, node: 0 }));
}
#[test]
fn a_page_with_two_mains_is_a_fault() {
    let mut b = page();
    banner(&mut b);
    b.open(Role::Main);
    b.close();
    b.open(Role::Main);
    b.close();
    assert_eq!(check(&finish(b)), Some(Fault { rule: Rule::OneMain, node: 6 }));
}
#[test]
fn a_region_hanging_from_the_page_is_a_fault() {
    let mut b = page();
    banner(&mut b);
    b.open(Role::Main);
    b.close();
    b.open(Role::Region);
    b.close();
    assert_eq!(check(&finish(b)), Some(Fault { rule: Rule::InsideLandmark, node: 6 }));
}
#[test]
fn a_main_after_the_notices_is_a_fault() {
    let mut b = page();
    banner(&mut b);
    b.open(Role::Region);
    b.region(JigRegion::Notices.region());
    b.close();
    b.open(Role::Main);
    b.close();
    assert_eq!(check(&finish(b)), Some(Fault { rule: Rule::AfterPageLast, node: 6 }));
}
#[test]
fn a_shortcut_named_twice_is_a_fault_at_its_second_node() {
    let mut b = page();
    banner(&mut b);
    b.open(Role::Main);
    for _ in 0_u8..2 {
        b.open(Role::Button);
        b.bind(crate::Binding {
            owner: (crate::Owner::Application { number: 1 }).opaque(),
            token: skein_lib::Token::new(1),
        });
        b.shortcut(Shortcut { key: b'a', modifier: Modifier::Primary });
        b.close();
    }
    b.close();
    assert_eq!(check(&finish(b)), Some(Fault { rule: Rule::ShortcutOnce, node: 7 }));
}
