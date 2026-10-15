//! The page's landmark outline (client/domain/view.md, section 3.1).
use crate::regions::JigRegion;
use crate::ui::{Role, Shortcut, Tree};
use skein_lib::List;

/// The first node in preorder breaking the outline, or zero for a missing node.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Fault {
    pub rule: Rule,
    pub node: u32,
}

/// One rule of the page's outline.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Rule {
    /// The page's first child is not the banner.
    BannerFirst,
    /// The banner lacks a required region.
    BannerHolds { region: JigRegion },
    /// A second main, or no main.
    OneMain,
    /// A second content info.
    OneContentInfo,
    /// A page child outside a landmark and `after_page`'s parts.
    InsideLandmark,
    /// A landmark after `after_page`, or its parts out of order.
    AfterPageLast,
    /// A shortcut already named by an earlier node.
    ShortcutOnce,
}

/// Check roles, region names and shortcuts, without reading words.
#[must_use]
pub fn check(tree: &Tree) -> Option<Fault> {
    let fault = landmarks(tree);
    let length = u32::try_from(tree.nodes().len()).expect("tree length fits u32");
    let mut seen: List<Shortcut> = List::with_capacity(length);
    for index in 0..length {
        let node = tree.get(index).expect("index within tree");
        if let Some(shortcut) = node.shortcut {
            for previous in &seen {
                if *previous == shortcut {
                    if let Some(earlier) = fault
                        && earlier.node <= index
                    {
                        return Some(earlier);
                    }
                    return Some(Fault { rule: Rule::ShortcutOnce, node: index });
                }
            }
            seen.push(shortcut).expect("at most one shortcut per node");
        }
    }
    fault
}

fn landmarks(tree: &Tree) -> Option<Fault> {
    let Some(banner) = tree.get(1) else {
        return Some(Fault { rule: Rule::BannerFirst, node: 0 });
    };
    if banner.role != Role::Banner {
        return Some(Fault { rule: Rule::BannerFirst, node: 1 });
    }
    for region in [JigRegion::Link, JigRegion::Needs, JigRegion::Party] {
        let mut found = false;
        for index in 2..banner.size.checked_add(1).expect("banner end fits") {
            let node = tree.get(index).expect("banner descendant exists");
            if node.role == Role::Region && node.region == Some(region.region()) {
                found = true;
            }
        }
        if !found {
            return Some(Fault { rule: Rule::BannerHolds { region }, node: 0 });
        }
    }
    let mut main = false;
    let mut content_info = false;
    let mut after = 0_u8;
    let mut next = banner.size.checked_add(1).expect("banner end fits");
    let mut fault = None;
    let length = u32::try_from(tree.nodes().len()).expect("tree length fits u32");
    for index in 1..length {
        if index != next {
            continue;
        }
        let node = tree.get(index).expect("page child exists");
        next = index.checked_add(node.size).expect("child end fits");
        let part = if node.role == Role::Complementary && node.region == Some(JigRegion::Owed.region()) {
            1
        } else if node.role == Role::Dialog && node.region == Some(JigRegion::Confirmation.region()) {
            2
        } else if node.role == Role::Region && node.region == Some(JigRegion::Notices.region()) {
            3
        } else {
            0
        };
        let rule = if part != 0 {
            let wrong = part <= after;
            after = part;
            if wrong { Some(Rule::AfterPageLast) } else { None }
        } else if node.role == Role::Main
            || node.role == Role::Navigation
            || node.role == Role::Complementary
            || node.role == Role::ContentInfo
        {
            let duplicate = if node.role == Role::Main {
                let duplicate = main;
                main = true;
                if duplicate { Some(Rule::OneMain) } else { None }
            } else if node.role == Role::ContentInfo {
                let duplicate = content_info;
                content_info = true;
                if duplicate { Some(Rule::OneContentInfo) } else { None }
            } else {
                None
            };
            if after != 0 { Some(Rule::AfterPageLast) } else { duplicate }
        } else {
            Some(Rule::InsideLandmark)
        };
        if fault.is_none()
            && let Some(rule) = rule
        {
            fault = Some(Fault { rule, node: index });
        }
    }
    if !main {
        return Some(Fault { rule: Rule::OneMain, node: 0 });
    }
    fault
}
