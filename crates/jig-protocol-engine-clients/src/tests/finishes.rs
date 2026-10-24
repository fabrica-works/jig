//! A pasted address is held, bounded and erased once (store.md, section 9).
use super::fixtures::{self as f, Exchange};
use crate::{Event, Message, Request, SecretOwner, SecretsAnswer, SecretsRequest};
use jig_client_protocol_wire::v19 as w;
use jig_domain_core as core;
use jig_domain_people as p;
use skein_lib::{Secret, Token, Writer};
const SOURCE: u32 = 29;
// Matches this world's configured driver redirect bound.
const REDIRECT_BYTES: usize = 4096;
const HELD_NUMBER: u64 = (1 << 63) + (1 << 62);
// Room for a message carrying an address at its bound.
fn roomy() -> Exchange {
    let mut env = f::env();
    env.limits.message_bytes = 8192;
    Exchange::with_env(env)
}
fn finish(exchange: &mut Exchange, bytes: &[u8]) -> Token {
    let limits = f::wire();
    let change = w::AccountChange::FinishSignIn(
        w::AccountChangeFinishSignIn::new(
            &limits,
            w::AccountChangeFinishSignInParts { source: SOURCE, digest: skein_lib::sha256::digest(bytes) },
        )
        .unwrap(),
    );
    let ask = w::Ask::AccountChange(
        w::AskAccountChange::new(&limits, w::AskAccountChangeParts { holder: w::AccountHolder::Own, change }).unwrap(),
    );
    let keyed = w::Keyed::new(
        &limits,
        w::KeyedParts {
            key: [5; 16],
            owner: w::Owner::Jig,
            version: 19,
            request: f::encode_ask(&ask),
            value: Some(Secret::new(Box::from(bytes))),
        },
    )
    .unwrap();
    let mut writer = Writer::new(usize::try_from(keyed.measure()).unwrap());
    keyed.encode(&mut writer).unwrap();
    let owner = exchange.send(Message::Request(writer.finish()));
    exchange.authenticate();
    owner
}
#[test]
fn a_finish_at_its_address_bound_is_held() {
    let mut exchange = roomy();
    let bytes = skein_lib::bytes::zeroed(REDIRECT_BYTES);
    finish(&mut exchange, &bytes);
    let Some(Request::Secrets(SecretsRequest::Hold { value, .. })) = exchange.out.pop() else { panic!("hold address") };
    assert_eq!(value.expose(), &*bytes);
    assert!(exchange.out.is_empty());
}
#[test]
fn a_finish_past_its_address_bound_holds_nothing() {
    let mut exchange = roomy();
    let bytes = skein_lib::bytes::zeroed(REDIRECT_BYTES + 1);
    let owner = finish(&mut exchange, &bytes);
    let w::Final::Refused(row) = exchange.final_answer(owner) else { panic!("limit") };
    let w::Refusal::Limit(_) = row.refusal() else { panic!("limit refusal") };
    assert!(exchange.out.is_empty());
}
#[test]
fn an_unused_finish_erases_its_held_address_once() {
    let mut exchange = roomy();
    let address = b"http://127.0.0.1:1455/auth/callback?code=c&state=s";
    let owner = finish(&mut exchange, address);
    let Some(Request::Secrets(SecretsRequest::Hold { exchange: token, .. })) = exchange.out.pop() else {
        panic!("hold")
    };
    exchange.component.down(
        &exchange.env,
        Event::Secrets(SecretsAnswer::Carried { exchange: token, number: Some(HELD_NUMBER) }),
        &mut exchange.out,
    );
    let Some(Request::Core(event)) = exchange.out.pop() else { panic!("core input") };
    let core::Event::Key { reply_to, ask, carried, .. } = *event else { panic!("carried input") };
    assert_eq!(
        *ask,
        p::Ask::AccountChange {
            holder: p::AccountHolder::Own,
            change: p::AccountChange::FinishSignIn { source: SOURCE, digest: skein_lib::sha256::digest(address) }
        }
    );
    assert_eq!(carried.number, HELD_NUMBER);
    for _ in 0_u8..2 {
        exchange.component.down(
            &exchange.env,
            Event::Application(crate::FromApplication::Unused { number: HELD_NUMBER }),
            &mut exchange.out,
        );
    }
    let Some(Request::Secrets(SecretsRequest::Erase { owner: SecretOwner::Carried(HELD_NUMBER) })) = exchange.out.pop()
    else {
        panic!("erase once")
    };
    assert!(exchange.out.is_empty());
    exchange.reply(reply_to, p::Reply::Refused(p::Refusal::SignInRedirect { why: p::SignInRedirectWhy::NotWaiting }));
    let w::Final::Refused(row) = exchange.final_answer(owner) else { panic!("refused") };
    let w::Refusal::SignInRedirect(row) = row.refusal() else { panic!("redirect refusal") };
    assert_eq!(row.why(), &w::SignInRedirectWhy::NotWaiting);
    assert!(exchange.out.is_empty());
}
