use crate::finishes::{Finishes, Next};
use jig_domain_accounts::{Event, SignInRedirectWhy};
use skein_lib::{Secret, Token, bytes};
use skein_oauth_accounts as driver;

#[test]
fn a_held_address_is_read_only_to_hand_it_to_its_sign_in() {
    let mut finishes = Finishes::new(1);
    let Next::Get { exchange, carried } = finishes.redirected(7, 90) else { panic!("read the held address") };
    assert_eq!(carried, 90);
    assert!(finishes.pending());
    let value = b"http://127.0.0.1:1455/callback?code=fake&state=fake";
    let Some(Next::Driver(request)) = finishes.read(exchange, Some(Secret::new(bytes::copy_of(value)))) else {
        panic!("driver request")
    };
    assert!(request == driver::Request::Redirected { account: 7, uri: bytes::copy_of(value) });
    assert!(!finishes.pending());
    assert!(finishes.read(exchange, None).is_none());
}
#[test]
fn a_gone_address_refuses_the_finish_without_asking_the_driver() {
    let mut finishes = Finishes::new(1);
    let Next::Get { exchange, .. } = finishes.redirected(7, 90) else { panic!("read") };
    let Some(Next::Child(event)) = finishes.read(exchange, None) else { panic!("refused, no driver request") };
    assert_eq!(event, Event::FinishRefused { source: 7, why: SignInRedirectWhy::NotWaiting });
    assert!(!finishes.pending());
}
#[test]
fn finish_reads_are_bounded_and_aborting_discards_their_tokens() {
    let mut finishes = Finishes::new(1);
    let Next::Get { exchange, .. } = finishes.redirected(7, 90) else { panic!("read") };
    let Next::Child(event) = finishes.redirected(8, 91) else { panic!("no read past the bound") };
    assert_eq!(event, Event::FinishRefused { source: 8, why: SignInRedirectWhy::NotWaiting });
    finishes.abort();
    assert!(!finishes.pending());
    assert!(finishes.read(exchange, None).is_none());
    assert!(finishes.read(Token::new(99), None).is_none());
}
