//! Program inventories cross bytes intact and share the document's native byte budget.
use super::{c, fixtures, w};
use crate::translate::programs as decode;
use skein_lib::{List, Writer};
fn login(name: &[u8], state: Option<w::OwnerProgramState>) -> w::OwnerProgramLogin {
    w::OwnerProgramLogin::new(
        &w::CEILINGS,
        w::OwnerProgramLoginParts { name: name.into(), state, sign_out: None, cleanup: false, cut: false },
    )
    .expect("literal login")
}
fn piece(row: &w::OwnerProgramLogin) -> Box<[u8]> {
    let mut out = Writer::new(usize::try_from(row.measure()).expect("measure").checked_add(4).expect("piece prefix"));
    out.put(&row.measure().to_be_bytes()).expect("prefix");
    row.encode(&mut out).expect("measured login");
    out.finish()
}
#[test]
fn program_inventory_preserves_every_login_state() {
    let failed = w::OwnerProgramFailed::new(
        &w::CEILINGS,
        w::OwnerProgramFailedParts { reason: w::OwnerProgramFailure::Deadline, detail: b"bounded".as_slice().into() },
    )
    .expect("failure");
    let method = w::OwnerProgramOtherwise::new(
        &w::CEILINGS,
        w::OwnerProgramOtherwiseParts { method: b"api-key".as_slice().into() },
    )
    .expect("method");
    for (wire, native) in [
        (None, None),
        (Some(w::OwnerProgramState::Usable), Some(c::OwnerProgramState::Usable)),
        (Some(w::OwnerProgramState::NotSignedIn), Some(c::OwnerProgramState::NotSignedIn)),
        (
            Some(w::OwnerProgramState::SignedInOtherwise(method)),
            Some(c::OwnerProgramState::SignedInOtherwise { method: b"api-key".as_slice().into() }),
        ),
        (
            Some(w::OwnerProgramState::Failed(failed)),
            Some(c::OwnerProgramState::Failed {
                reason: c::OwnerProgramFailure::Deadline,
                detail: b"bounded".as_slice().into(),
            }),
        ),
    ] {
        let mut room = 4096;
        let rows =
            decode::logins(&piece(&login(b"personal", wire)), &fixtures::limits(), &mut room).expect("inventory");
        assert_eq!(rows.get(0).expect("one row").state, native);
    }
}
#[test]
fn malformed_program_names_are_refused() {
    let mut room = 4096;
    let result = decode::logins(&piece(&login(b"Bad Name", None)), &fixtures::limits(), &mut room);
    assert_eq!(result.err(), Some(w::Problem { path: w::Path::OwnedRecord, reason: skein_codec::Reason::Malformed }));
}
#[test]
fn malformed_payment_methods_are_refused() {
    let method = w::OwnerProgramOtherwise::new(
        &w::CEILINGS,
        w::OwnerProgramOtherwiseParts { method: b"bad method".as_slice().into() },
    )
    .expect("wire bytes");
    let mut room = 4096;
    let result = decode::logins(
        &piece(&login(b"personal", Some(w::OwnerProgramState::SignedInOtherwise(method)))),
        &fixtures::limits(),
        &mut room,
    );
    assert_eq!(result.err(), Some(w::Problem { path: w::Path::OwnedRecord, reason: skein_codec::Reason::Malformed }));
}
#[test]
fn native_program_budget_accepts_its_exact_bound() {
    let mut room = List::<c::OwnerProgramLogin>::worst_case(1).expect("one row").checked_add(1).expect("name");
    let rows = decode::logins(&piece(&login(b"p", Some(w::OwnerProgramState::Usable))), &fixtures::limits(), &mut room)
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(room, 0);
}
#[test]
fn native_program_budget_refuses_one_byte_less() {
    let mut room = List::<c::OwnerProgramLogin>::worst_case(1).expect("one row");
    let result =
        decode::logins(&piece(&login(b"p", Some(w::OwnerProgramState::Usable))), &fixtures::limits(), &mut room);
    assert_eq!(result.err(), Some(w::Problem { path: w::Path::OwnedRecord, reason: skein_codec::Reason::Bound }));
}

#[test]
fn program_lists_over_the_native_budget_are_refused() {
    let one = piece(&login(b"p", Some(w::OwnerProgramState::Usable)));
    let mut bytes = Writer::new(one.len().checked_mul(2).expect("two pieces"));
    bytes.put(&one).expect("first");
    bytes.put(&one).expect("second");
    let mut room = List::<c::OwnerProgramLogin>::worst_case(1).expect("one row").checked_add(1).expect("name");
    let result = decode::logins(&bytes.finish(), &fixtures::limits(), &mut room);
    assert_eq!(result.err(), Some(w::Problem { path: w::Path::OwnedRecord, reason: skein_codec::Reason::Bound }));
}
#[test]
fn configured_host_names_stay_whole_below_the_text_limit() {
    let row = w::OwnerProgramHost::new(
        &w::CEILINGS,
        w::OwnerProgramHostParts {
            name: b"configured-worker".as_slice().into(),
            channel: 17,
            connected: true,
            awaiting_logins: true,
            programs: Box::new([]),
            cut: false,
        },
    )
    .expect("host");
    let mut bytes = Writer::new(usize::try_from(row.measure()).expect("row").checked_add(4).expect("prefix"));
    bytes.put(&row.measure().to_be_bytes()).expect("prefix");
    row.encode(&mut bytes).expect("row");
    let mut limits = fixtures::limits();
    limits.client.text = 1;
    limits.client.turn_text = 1;
    let mut room = 4096;
    let rows = decode::hosts(&bytes.finish(), &limits, &mut room).expect("whole name");
    assert_eq!(rows.get(0).expect("host").name.as_ref(), b"configured-worker");
}

#[test]
fn a_host_rule_survives_wire_receipt() {
    let person = w::OwnerHostPerson::new(&w::CEILINGS, w::OwnerHostPersonParts { party: 7 }).expect("person");
    for (rule, expected) in [
        (w::OwnerHostRule::Open, c::HostRule::Open),
        (w::OwnerHostRule::Person(person), c::HostRule::Person(7)),
        (w::OwnerHostRule::Several, c::HostRule::Several),
        (w::OwnerHostRule::Unknown, c::HostRule::Unknown),
    ] {
        let row = w::OwnerHost::new(
            &w::CEILINGS,
            w::OwnerHostParts {
                name: None,
                skein_bools: [false, false, true, false],
                rule,
                programs: Box::new([]),
                channel: 17,
                kind: w::OwnerHostKind::Worker,
                slots: 1,
                used: 0,
                free: 1,
                grace_until: None,
                grace_remaining: None,
                runs: Box::new([]),
            },
        )
        .expect("host");
        let mut bytes = Writer::new(usize::try_from(row.measure()).expect("row").checked_add(4).expect("prefix"));
        bytes.put(&row.measure().to_be_bytes()).expect("prefix");
        row.encode(&mut bytes).expect("row");
        let mut room = 4096;
        let limits = fixtures::limits();
        let rows =
            crate::translate::documents::owner_host_piece(&bytes.finish(), &limits, limits.client.rows, &mut room)
                .expect("host receipt");
        assert_eq!(rows.get(0).expect("host").rule, expected);
    }
}
