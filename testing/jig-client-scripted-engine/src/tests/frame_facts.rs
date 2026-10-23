use super::{frame, list, text};
use crate::{
    Bounds,
    payload::{Payload, fits},
};
use jig_client_domain::contract::*;
use skein_lib::{List, Wall};
#[test]
fn me_prices_memberships_offers_and_required_project_names() {
    let mut row = frame(1);
    row.party.login = DocumentText { bytes: text(&[b'l'; 32]), cut: true };
    row.party.name = DocumentText::complete(text(&[b'n'; 32]));
    row.projects.get_mut(0).unwrap().name = DocumentText::complete(text(&[b'p'; 32]));
    let mut bounds = Bounds { rows: 2, text: 32, document_bytes: u64::MAX, ..Bounds::default() };
    let expected = u64::try_from(core::mem::size_of::<Me>()).unwrap() + List::<MeProject>::worst_case(1).unwrap() + 96;
    assert_eq!(row.heap(&bounds).unwrap() + u64::try_from(core::mem::size_of::<Me>()).unwrap(), expected);
    bounds.document_bytes = expected;
    assert!(fits(&row, &bounds));
    bounds.document_bytes -= 1;
    assert!(!fits(&row, &bounds));
    bounds.document_bytes += 2;
    assert!(fits(&row, &bounds));
    bounds.document_bytes = u64::MAX;
    row.projects.get_mut(0).unwrap().name = DocumentText::complete(text(&[b'p'; 33]));
    assert!(!fits(&row, &bounds));
    bounds.text = 33;
    assert!(fits(&row, &bounds));
}
#[test]
fn builtin_roles_remain_memberships_without_becoming_numeric_policy_recipients() {
    let roles = [
        HeldRole::Owner,
        HeldRole::Maintainer,
        HeldRole::Member,
        HeldRole::Observer,
        HeldRole::Policy { role: u32::MAX },
    ];
    for role in roles {
        let mut row = frame(1);
        row.projects.get_mut(0).unwrap().role = role;
        row.projects.get_mut(0).unwrap().pool = None;
        let mut truth = crate::Truth::default();
        truth.frames.insert(1, row.clone());
        assert!(truth.visible(1, 1));
        assert!(!truth.visible(1, 2));
        assert_eq!(
            crate::requests::recipient(&truth, 1, &Recipient::Role { project: 1, role: u32::MAX }),
            role == HeldRole::Policy { role: u32::MAX }
        );
        assert!(!crate::requests::recipient(&truth, 1, &Recipient::Role { project: 1, role: 1 }));
        let request = Ask::SetRoles { project: u32::MAX, holdings: list(vec![Holding { party: u64::MAX, role }]) };
        assert!(fits(&request, &Bounds::default()));
        let service = Ask::MakeService { project: u32::MAX, name: text(b"actual-service"), role };
        assert!(fits(&service, &Bounds::default()));
        row.projects.get_mut(0).unwrap().pool = Some(PoolLedger {
            source: Funder::Pool { project: u32::MAX, person: u64::MAX, period: u64::MAX },
            parent: Some(Funder::Period { project: u32::MAX, period: u64::MAX }),
            numbers: OwnerNumbers { budget: u64::MAX, spent: u64::MAX, spent_below: u64::MAX, reserved: u64::MAX },
            left: 0,
            closed: true,
            began: Some(Wall::EPOCH),
            next: None,
        });
        assert!(fits(&row, &Bounds::default()));
    }
}
