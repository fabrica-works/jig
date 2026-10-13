use jig_client_protocol_world::referee::{Broken, Rules};
use jig_protocol_client_wire::v7 as wire;
use skein_lib::{Reader, Token, Writer};

#[test]
fn a_record_altered_on_the_wire_is_caught() {
    let value = wire::LinkSignIn::new(&wire::CEILINGS, wire::LinkSignInParts { secret: Box::new([7; 32]) })
        .expect("fixture record");
    let mut writer = Writer::new(usize::try_from(value.measure()).expect("fixture size"));
    value.encode(&mut writer).expect("measured record");
    let source = writer.finish();
    let mut actual = source.clone();
    *actual.last_mut().expect("nonempty record") = 8;
    assert!(wire::LinkSignIn::decode(&wire::CEILINGS, &mut Reader::new(&actual)).is_ok());
    assert_eq!(Rules::default().record(b"/jig/v1/sign-in/link", false, &source, &actual), Err(Broken::Record));
}

#[test]
fn a_token_leaked_into_a_domain_event_is_caught() {
    let secret = [73; 40];
    let event = jig_domain_core::Event::RunText {
        retry: 0,
        run: Token::new(1),
        attempt: Token::new(1),
        turn: 1,
        offset: 0,
        text: Box::new(secret),
        call: None,
    };
    let mut rules = Rules::default();
    rules.secret(&secret);
    assert_eq!(rules.domain(&event), Err(Broken::Secret));
}
