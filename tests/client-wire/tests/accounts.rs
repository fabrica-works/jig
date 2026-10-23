//! Accounts watch records preserve every terminal outcome and optional wall.
use jig_client_protocol_wire::v18 as w;
use skein_lib::{Reader, Writer};

#[test]
fn whole_accounts_round_trip_with_every_terminal_outcome() {
    let limits = jig_client_test_records::limits();
    let ends = std::iter::once(None).chain(jig_client_test_records::account_end_records().into_iter().map(Some));
    for end in ends {
        let record = jig_client_test_records::accounts_record(end);
        let mut out = Writer::new(usize::try_from(record.measure()).expect("measured record fits"));
        record.encode(&mut out).expect("measured accounts record");
        let bytes = out.finish();
        assert_eq!(w::OwnerAccounts::decode(&limits, &mut Reader::new(&bytes)), Ok(record));
    }
}
