use super::super::*;
use super::fixtures as f;
use f::Exchange;
#[test]
fn a_made_service_answers_its_credential_once_with_its_digest_kept() {
    let mut exchange = Exchange::new();
    let (owner, to) = exchange.make_service();
    exchange.service_made(to);
    exchange.service_got(None);
    exchange.keep_service();
    assert!(exchange.component.connections.get(&owner).expect("owner").response.is_none());
    exchange.component.down(
        &exchange.env,
        Event::Secrets(SecretsAnswer::Kept { owner: SecretOwner::Service(f::SERVICE) }),
        &mut exchange.out,
    );
    assert_eq!(exchange.service_credential(owner), f::credential());
    exchange.component.down(
        &exchange.env,
        Event::Secrets(SecretsAnswer::Kept { owner: SecretOwner::Service(f::SERVICE) }),
        &mut exchange.out,
    );
    assert_eq!(exchange.service_credential(owner), f::credential());
    assert!(exchange.component.pending.exchanges.is_empty());
    assert!(exchange.out.is_empty());
}
#[test]
fn a_replayed_make_service_answers_its_party_without_a_credential() {
    for concurrent in [false, true] {
        let mut exchange = Exchange::new();
        let (owner, to) = exchange.make_service();
        exchange.service_made(to);
        if concurrent {
            exchange.service_got(None);
            exchange.keep_service();
            let (other, to) = exchange.make_service();
            exchange.service_made(to);
            exchange.service_got(None);
            assert!(exchange.out.is_empty());
            exchange.component.down(
                &exchange.env,
                Event::Secrets(SecretsAnswer::Kept { owner: SecretOwner::Service(f::SERVICE) }),
                &mut exchange.out,
            );
            assert_eq!(exchange.service_credential(owner), f::credential());
            assert!(exchange.service_credential(other).expose().is_empty());
        } else {
            exchange.service_got(Some(f::service_kept()));
            assert!(exchange.service_credential(owner).expose().is_empty());
        }
        assert!(exchange.out.is_empty());
        assert!(exchange.component.pending.exchanges.is_empty());
    }
}
#[test]
fn a_service_signs_in_with_its_credential_and_exchanges_with_its_sign_in() {
    let mut exchange = Exchange::new();
    let (owner, to) = exchange.sign_in_service();
    exchange.reply(
        to,
        people::Reply::SignedIn { person: f::SERVICE, sign_in: f::NUMBER, expires: skein_lib::Wall::from_nanos(99) },
    );
    let Some(Request::Secrets(SecretsRequest::Keep { owner: who, .. })) = exchange.out.pop() else {
        panic!("sign-in keep")
    };
    assert_eq!(who, SecretOwner::SignIn(f::NUMBER));
    exchange.component.down(&exchange.env, Event::Secrets(SecretsAnswer::Kept { owner: who }), &mut exchange.out);
    let answer = exchange.signed_in_answer(owner);
    assert_eq!(answer.party(), f::SERVICE);
    assert_eq!(answer.token().expose()[..8], f::NUMBER.to_be_bytes());
    exchange.token = answer.token().expose().into();
    let (request_owner, to) = exchange.ask();
    exchange.reply(to, people::Reply::Outcome(people::Outcome::Said { task: 8, message: 9 }));
    assert_eq!(exchange.final_answer(request_owner), f::started());
    assert!(exchange.out.is_empty());
}
#[test]
fn a_deployment_sign_in_naming_no_service_is_refused() {
    let mut exchange = Exchange::new();
    let (owner, to) = exchange.sign_in_service();
    exchange.reply(to, people::Reply::SignedOut);
    let (_, bytes) = exchange.response(owner);
    assert_eq!(
        w::SignInAnswer::decode(&f::wire(), &mut skein_lib::Reader::new(bytes)).expect("refusal"),
        w::SignInAnswer::Refused
    );
    assert!(exchange.out.is_empty());
    assert!(exchange.component.pending.exchanges.is_empty());
}
