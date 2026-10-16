use crate::family;
use jig_client_domain::contract::{OwnerPart, OwnerPartKey, OwnerPartKind, Part};
use skein_lib::List;

#[test]
fn a_native_hold_account_uses_only_its_supplied_connector_and_number() {
    for connector in [0, crate::FIRST, crate::SECOND, u16::MAX] {
        for task in [0, u64::MAX] {
            let native = OwnerPart { connector, kind: OwnerPartKind::Hold, key: OwnerPartKey::Number(task) };
            assert_eq!(family::owner_hold(&native), Some(Part::Hold { connector, task }));
        }
    }
    for kind in [
        OwnerPartKind::Hold,
        OwnerPartKind::Resource,
        OwnerPartKind::Pool,
        OwnerPartKind::Result,
        OwnerPartKind::Procedure,
        OwnerPartKind::Effect,
        OwnerPartKind::Outbox,
    ] {
        let mut path = List::with_capacity(2);
        path.push(Box::<[u8]>::from(*b"same/number")).unwrap();
        path.push(Box::<[u8]>::from([255, 0])).unwrap();
        assert_eq!(
            family::owner_hold(&OwnerPart { connector: crate::FIRST, kind, key: OwnerPartKey::Path(path) }),
            None
        );
        if kind != OwnerPartKind::Hold {
            assert_eq!(
                family::owner_hold(&OwnerPart { connector: crate::FIRST, kind, key: OwnerPartKey::Number(7) }),
                None
            );
        }
    }
}
