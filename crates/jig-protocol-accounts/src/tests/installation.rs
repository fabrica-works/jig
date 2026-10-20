use crate::installation;
use skein_oauth as oauth;

#[test]
fn the_identifier_reads_as_a_version_4_uuid_urn() {
    let id = [0xff, 0xee, 0xdd, 0xcc, 0xbb, 0xaa, 0x99, 0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0x00];
    assert_eq!(installation::urn(&id).as_ref(), b"urn:uuid:ffeeddcc-bbaa-4988-b766-554433221100");
    assert_eq!(installation::urn(&[0; 16]).as_ref(), b"urn:uuid:00000000-0000-4000-8000-000000000000");
}
#[test]
fn the_terms_send_the_identifier_on_every_sign_in() {
    let terms = installation::terms(&[0; 16]);
    let [parameter] = terms.parameters.as_ref() else {
        panic!("one parameter");
    };
    assert_eq!(parameter.name.as_ref(), b"ext_agent_host_id");
    assert_eq!(parameter.value, installation::urn(&[0; 16]));
    assert_eq!(parameter.sent, oauth::Sent::EverySignIn);
    let counted = parameter.name.len() + parameter.value.len();
    assert_eq!(u32::try_from(counted), Ok(installation::PARAMETER_BYTES));
    assert_eq!((terms.openid, terms.issued_client_id, terms.varying_port), (None, false, false));
}
