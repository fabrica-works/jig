use super::fixtures::{self, PAYLOAD, WORKER_A};
use crate::opening::{Verdict, same};
use jig_protocol_worker_wire::Reason;
use skein_io::digest::Digest;
#[test]
fn a_wrong_secret_and_an_unknown_name_refuse_alike() {
    let component = fixtures::component();
    for name in [WORKER_A, b"unknown"] {
        let bytes = fixtures::credential(name, b"wrong", PAYLOAD);
        assert_eq!(component.judge(&bytes), Verdict::Refuse(Reason::Unauthorized));
    }
}
#[test]
fn a_wrong_secret_with_another_payload_is_unauthorized() {
    let component = fixtures::component();
    let bytes = fixtures::credential(WORKER_A, b"wrong", 4);
    assert_eq!(component.judge(&bytes), Verdict::Refuse(Reason::Unauthorized));
}
#[test]
fn digests_differing_in_the_last_word_are_not_the_same() {
    assert!(!same(&Digest([1, 2, 3, 4]), &Digest([1, 2, 3, 5])), "the last word participates");
}
#[test]
fn an_undecodable_credential_is_unauthorized() {
    let component = fixtures::component();
    assert_eq!(component.judge(b"bad"), Verdict::Refuse(Reason::Unauthorized));
}

#[test]
fn worker_names_are_bounded_independently_of_channels() {
    let limits = crate::Limits { names: 1, ..fixtures::LIMITS };
    let result = crate::Component::new(&limits, fixtures::workers(), PAYLOAD);
    assert_eq!(result.err(), Some(crate::Unusable::Workers));
}

#[test]
fn a_configured_worker_authenticates_with_its_secret() {
    let component = fixtures::component();
    let bytes = fixtures::credential(WORKER_A, fixtures::SECRET_A, PAYLOAD);
    assert_eq!(component.judge(&bytes), Verdict::Accept { worker: 0 });
}
