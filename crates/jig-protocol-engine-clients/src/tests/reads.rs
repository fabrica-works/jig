use super::super::*;
use super::fixtures as f;
use jig_domain_core::reads;
#[test]
fn a_read_is_answered_only_once_held_is_released() {
    let cases = f::held_reads();
    for (label, value, expected_read, answer, expected_answer) in cases {
        let mut exchange = f::read_exchange();
        let (owner, reply, read) = exchange.read(&value);
        assert_eq!(read, expected_read, "{label}");
        assert!(exchange.read_unanswered(owner), "{label}");
        let name = reply.into_token();
        exchange.release_read(ReplyTo::new(name), answer.clone());
        assert_eq!(exchange.read_answer(owner), expected_answer, "{label}");
        exchange.release_read(ReplyTo::new(name), reads::Answer::None(reads::Why::Unknown));
        assert_eq!(exchange.read_answer(owner), expected_answer, "{label}");
        assert!(exchange.component.pending.exchanges.is_empty(), "{label}");
        assert!(exchange.out.is_empty(), "{label}");
    }
}
#[test]
fn a_cursor_round_trips_to_the_same_position() {
    for (label, position) in f::cursors() {
        let encoded = crate::translate::cursor::encode(&position, 1024).expect(label);
        assert_eq!(crate::translate::cursor::decode(&encoded), Some(position.clone()), "{label}");
        let size = u32::try_from(encoded.len()).expect("small cursor");
        assert_eq!(crate::translate::cursor::encode(&position, size), Some(encoded.clone()), "{label}");
        assert_eq!(
            crate::translate::cursor::encode(&position, size.checked_sub(1).expect("cursor header")),
            None,
            "{label}"
        );
        for cut in 0..encoded.len() {
            assert_eq!(crate::translate::cursor::decode(&encoded[..cut]), None, "{label}, cut {cut}");
        }
        let mut trailing = skein_lib::Writer::new(encoded.len().checked_add(1).expect("small cursor"));
        trailing.put(&encoded).expect("cursor");
        trailing.put(&[0]).expect("extra byte");
        assert_eq!(crate::translate::cursor::decode(&trailing.finish()), None, "{label}, trailing");
    }
}
#[test]
fn a_cursor_that_does_not_decode_is_answered_unknown() {
    let limits = f::wire();
    for (label, bytes) in
        [("unknown format", &[255][..]), ("unknown variant", &[1, 255][..]), ("truncated scope", &[1, 1, 10][..])]
    {
        let mut exchange = f::read_exchange();
        let after = w::Cursor::new(&limits, w::CursorParts { value: bytes.into() }).expect(label);
        let value = w::Read::Goals(
            w::ReadGoals::new(&limits, w::ReadGoalsParts { project: 3, after: Some(after) }).expect(label),
        );
        let owner = exchange.post_to(b"/jig/v1/read", f::owned_read(w::Owner::Jig, 7, f::encoded_read(&value)));
        exchange.authenticate();
        assert_eq!(exchange.read_answer(owner), f::read_none(w::ReadNone::Unknown), "{label}");
        assert!(exchange.out.is_empty(), "{label}");
        assert!(exchange.component.pending.exchanges.is_empty(), "{label}");
    }
}
#[test]
fn a_read_signed_out_is_answered_signed_out() {
    for label in ["core answer", "ended pending sign-in", "absent digest"] {
        let mut exchange = f::read_exchange();
        let owner = if label == "absent digest" {
            let owner =
                exchange.post_to(b"/jig/v1/read", f::owned_read(w::Owner::Jig, 7, f::encoded_read(&w::Read::Frame)));
            let Some(Request::Secrets(SecretsRequest::Get { owner: who })) = exchange.out.pop() else {
                panic!("digest get")
            };
            exchange.component.down(
                &exchange.env,
                Event::Secrets(SecretsAnswer::Got { owner: who, value: None }),
                &mut exchange.out,
            );
            owner
        } else {
            let (owner, reply, _) = exchange.read(&w::Read::Frame);
            if label == "core answer" {
                exchange.release_read(reply, reads::Answer::None(reads::Why::SignedOut));
            } else {
                exchange.component.down(&exchange.env, Event::SignInEnded { sign_in: f::NUMBER }, &mut exchange.out);
                let Some(Request::Secrets(SecretsRequest::Erase { owner: secret_owner })) = exchange.out.pop() else {
                    panic!("digest erase")
                };
                assert_eq!(secret_owner, SecretOwner::SignIn(f::NUMBER), "{label}");
            }
            owner
        };
        assert_eq!(exchange.read_answer(owner), f::read_none(w::ReadNone::SignedOut), "{label}");
        assert!(exchange.component.pending.exchanges.is_empty(), "{label}");
        assert!(exchange.out.is_empty(), "{label}");
    }
}
#[test]
fn an_application_read_is_routed_and_answered_under_its_family() {
    for label in ["document", "unknown"] {
        let mut exchange = f::read_exchange();
        let (owner, reply) = exchange.application_read();
        assert!(exchange.read_unanswered(owner), "{label}");
        let name = reply.into_token();
        exchange.release_application_read(ReplyTo::new(name), 8, None);
        assert!(exchange.read_unanswered(owner), "{label}");
        let record = if label == "document" { Some(Box::from([255, 0, 2])) } else { None };
        exchange.release_application_read(ReplyTo::new(name), 9, record);
        let expected = if label == "document" {
            let family = w::Family::new(&f::wire(), w::FamilyParts { number: 9 }).expect(label);
            w::ReadAnswer::Value(
                w::Owned::new(
                    &f::wire(),
                    w::OwnedParts { owner: w::Owner::Family(family), version: 2, record: Box::from([255, 0, 2]) },
                )
                .expect(label),
            )
        } else {
            f::read_none(w::ReadNone::Unknown)
        };
        assert_eq!(exchange.read_answer(owner), expected, "{label}");
        exchange.release_application_read(ReplyTo::new(name), 9, None);
        assert_eq!(exchange.read_answer(owner), expected, "{label}, late");
        assert!(exchange.component.pending.exchanges.is_empty(), "{label}");
    }
}
