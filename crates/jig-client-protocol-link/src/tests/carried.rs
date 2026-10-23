//! A request carrying a value is sent once and never held behind the store
//! (protocol/client.md, 5 and 10): it leaves at once, and the link answers it
//! *unsent* when it never left and *lost* when its sign-in ends under it.
use super::*;
use fixtures::{ATTEMPT, Harness, limits};
use skein_lib::Secret;
const CARRIED_KEY: skein_lib::RequestKey = [4; 16];
const VALUE: [u8; 5] = *b"value";
fn carry(h: &mut Harness, attempt: Token) {
    let ask = Ask::Family { scope: c::Scope::Own, record: fixtures::family_read() };
    let value = Some(Secret::new(Box::new(VALUE)));
    h.link.step(Time::ZERO, Event::Send { attempt, key: CARRIED_KEY, ask, value }, &mut h.out);
}
fn posted(h: &Harness) -> Option<w::Keyed> {
    for request in &h.out {
        if let Request::Exchange { message: Message::Request(body), .. } = request {
            let record = w::Keyed::decode(&limits().wire, &mut Reader::new(body)).expect("keyed request");
            if record.key() == &CARRIED_KEY {
                return Some(record);
            }
        }
    }
    None
}
#[test]
fn a_carried_request_leaves_while_the_table_waits_on_a_write() {
    let mut h = Harness::new(&limits());
    h.save(fixtures::saved_table(fixtures::message()));
    h.send_kept(Token::new(5));
    carry(&mut h, ATTEMPT);
    assert_eq!(h.exchanges(), 1, "the carried request alone leaves");
    let record = posted(&h).expect("the carried request left");
    assert_eq!(record.value(), &Some(Secret::new(Box::new(VALUE))));
    h.write_failed();
    assert!(h.has(&Request::NotKept { attempt: Token::new(5) }));
    assert_eq!(h.answered(ATTEMPT), None, "a failed write never answers it");
}
#[test]
fn a_carried_attempt_is_answered_lost_when_its_sign_in_ends() {
    let mut h = Harness::new(&limits());
    h.send(Token::new(5), fixtures::KEY, fixtures::message());
    carry(&mut h, ATTEMPT);
    h.link.step(Time::ZERO, Event::SignIn { with: With::Transport }, &mut h.out);
    assert_eq!(h.answered(ATTEMPT), Some((RequestEnvelope::Lost, None)));
    assert_eq!(h.answered(Token::new(5)), None, "the table's attempt ends unanswered");
}
#[test]
fn a_carried_request_the_link_cannot_post_is_unsent() {
    let mut h = Harness::new(&limits());
    for number in 0..4 {
        h.send(Token::new(number), [u8::try_from(number).expect("small"); 16], fixtures::message());
    }
    carry(&mut h, ATTEMPT);
    assert_eq!(h.answered(ATTEMPT), Some((RequestEnvelope::Again, None)), "never left, so unsent");
    assert!(posted(&h).is_none());
    h.send(Token::new(9), fixtures::KEY, fixtures::message());
    assert_eq!(h.answered(Token::new(9)), Some((RequestEnvelope::Lost, None)), "the table's attempt is lost");
}

fn move_out(h: &mut Harness, attempt: Token) {
    h.link.step(
        Time::ZERO,
        Event::Send {
            attempt,
            key: CARRIED_KEY,
            ask: Ask::Jig(Box::new(c::Ask::AccountChange {
                holder: c::AccountHolder::Own,
                change: c::AccountChange::MoveOut { account: u32::MAX },
            })),
            value: None,
        },
        &mut h.out,
    );
}
#[test]
fn a_move_out_leaves_while_the_table_waits_on_a_write() {
    let mut h = Harness::new(&limits());
    h.save(fixtures::saved_table(fixtures::message()));
    h.send_kept(Token::new(5));
    move_out(&mut h, ATTEMPT);
    assert_eq!(h.exchanges(), 1);
    assert_eq!(posted(&h).expect("move out left").value(), &None);
    h.write_failed();
    assert_eq!(h.answered(ATTEMPT), None);
}
#[test]
fn a_move_out_the_link_cannot_post_is_unsent() {
    let mut h = Harness::new(&limits());
    for number in 0..limits().exchanges {
        h.send(Token::new(u64::from(number)), [u8::try_from(number).unwrap(); 16], fixtures::message());
    }
    move_out(&mut h, ATTEMPT);
    assert_eq!(h.answered(ATTEMPT), Some((RequestEnvelope::Again, None)));
    assert!(posted(&h).is_none());
}
