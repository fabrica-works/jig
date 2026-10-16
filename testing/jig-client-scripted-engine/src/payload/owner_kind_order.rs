//! Scalar configured order rows; the surrounding list prices its full capacity.
use super::Payload;
use crate::Bounds;
use jig_client_domain::contract::OwnerKindImplication;

impl Payload for OwnerKindImplication {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        Some(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use skein_lib::List;

    #[test]
    fn whole_kind_relations_price_reserved_rows_and_reject_one_under_rows_or_bytes() {
        let mut rows = List::with_capacity(4);
        let pair = OwnerKindImplication { connector: u16::MAX, kind: u16::MAX, implies: u16::MAX - 1 };
        rows.push(pair).unwrap();
        rows.push(pair).unwrap();
        let bytes = u64::try_from(core::mem::size_of::<List<OwnerKindImplication>>()).unwrap()
            + List::<OwnerKindImplication>::worst_case(4).unwrap();
        let mut bounds = Bounds { rows: 4, document_bytes: bytes, ..Bounds::default() };
        assert!(crate::payload::fits(&rows, &bounds));
        assert_eq!(rows.as_slice(), &[pair, pair]);
        bounds.document_bytes -= 1;
        assert!(!crate::payload::fits(&rows, &bounds));
        bounds.document_bytes = bytes;
        bounds.rows = 3;
        assert!(!crate::payload::fits(&rows, &bounds));
        let empty = List::<OwnerKindImplication>::with_capacity(4);
        assert!(!crate::payload::fits(&empty, &bounds), "reserved empty storage also exceeds receiving rows");
        bounds.rows = 4;
        assert!(crate::payload::fits(&empty, &bounds));
        assert_eq!(empty.heap(&bounds), rows.heap(&bounds), "every allocated scalar row is priced");
    }
}
