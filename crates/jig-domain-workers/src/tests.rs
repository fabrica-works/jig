use super::Membership;
use alloc::boxed::Box;
fn membership() -> Membership {
    Membership::new(Box::new([Box::from(b"alpha".as_slice()), Box::from(b"beta".as_slice())]), 2, 32)
        .expect("bounded membership")
}
#[test]
fn membership_refuses_a_position_at_its_count() {
    let membership = membership();
    assert_eq!(membership.worker(2), None);
}
#[test]
fn membership_mints_the_last_position() {
    let membership = membership();
    let worker = membership.worker(1).expect("last member");
    assert_eq!(worker.number(), 1);
    assert_eq!(membership.name(worker), b"beta");
}
