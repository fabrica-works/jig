use crate::family::{self, Kind, Scope, Subject};
use jig_client::contract::{OwnerPart, OwnerPartKey, OwnerPartKind, Part, ProposalKey, Proposer};
use skein_lib::List;

#[test]
fn a_native_effect_reference_keeps_its_connector_number_and_separate_namespace() {
    for connector in [0, crate::FIRST, crate::SECOND, u16::MAX] {
        for proposal in [0, u64::MAX] {
            let native = OwnerPart { connector, kind: OwnerPartKind::Effect, key: OwnerPartKey::Number(proposal) };
            let part = family::owner_effect(&native).unwrap();
            assert_eq!(part, Part::OwnerEffect { connector, proposal });
            assert_eq!(crate::parts::connector(&part), connector);
            for scope in [Scope::Project(0), Scope::Project(u32::MAX), Scope::Deployment] {
                let subject = family::subject(&part, scope).unwrap();
                assert_eq!(subject, Subject { scope, kind: Kind::OwnerEffect(proposal) });
                for proposer in [Proposer::Task(0), Proposer::Party(u64::MAX)] {
                    let key = ProposalKey { proposer, number: proposal };
                    assert_ne!(subject, family::subject(&Part::Effect { connector, proposal: key }, scope).unwrap());
                }
            }
            assert_eq!(family::owner_hold(&native), None);
        }
    }
    for kind in [
        OwnerPartKind::Effect,
        OwnerPartKind::Hold,
        OwnerPartKind::Resource,
        OwnerPartKind::Pool,
        OwnerPartKind::Result,
        OwnerPartKind::Procedure,
        OwnerPartKind::Outbox,
    ] {
        let mut path = List::with_capacity(2);
        path.push(Box::<[u8]>::from(*b"number/17")).unwrap();
        path.push(Box::<[u8]>::from([255, 0])).unwrap();
        assert_eq!(
            family::owner_effect(&OwnerPart { connector: crate::FIRST, kind, key: OwnerPartKey::Path(path) }),
            None
        );
        if kind != OwnerPartKind::Effect {
            assert_eq!(
                family::owner_effect(&OwnerPart { connector: crate::FIRST, kind, key: OwnerPartKey::Number(17) }),
                None
            );
        }
    }
}
