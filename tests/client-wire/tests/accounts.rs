//! Accounts watch records preserve every terminal outcome and optional wall.
use jig_client_protocol_wire::v10 as w;
use skein_lib::{Reader, Writer};

#[test]
fn whole_accounts_round_trip_with_every_terminal_outcome() {
    let limits = jig_client_test_records::limits();
    for end in jig_client_test_records::account_end_records() {
        let record = jig_client_test_records::accounts_record(Some(end));
        let mut out = Writer::new(usize::try_from(record.measure()).expect("measured record fits"));
        record.encode(&mut out).expect("measured accounts record");
        let bytes = out.finish();
        assert_eq!(w::OwnerAccounts::decode(&limits, &mut Reader::new(&bytes)), Ok(record));
    }
    let record = jig_client_test_records::accounts_record(None);
    let mut out = Writer::new(usize::try_from(record.measure()).expect("measured record fits"));
    record.encode(&mut out).expect("measured accounts record");
    assert_eq!(w::OwnerAccounts::decode(&limits, &mut Reader::new(&out.finish())), Ok(record));
}
