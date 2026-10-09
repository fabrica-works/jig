use crate::{Connector, connector, number};

#[test]
fn both_connectors_round_trip_through_their_fixed_numbers() {
    assert_eq!(number(Connector::Observability), 1);
    assert_eq!(number(Connector::Infrastructure), 2);
    for owner in [Connector::Observability, Connector::Infrastructure] {
        assert_eq!(connector(number(owner)), Some(owner));
    }
}

#[test]
fn all_other_numbers_have_no_connector() {
    for value in 0..=u16::MAX {
        if value != 1 && value != 2 {
            assert_eq!(connector(value), None);
        }
    }
}
