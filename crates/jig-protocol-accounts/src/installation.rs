//! The installation's identifier where the provider takes it (protocol/llm.md,
//! 5.4 "The registration's parts"; domain/providers.md, 2.2): a `urn:uuid:`
//! sent as `ext_agent_host_id` on every sign-in, new registration or not.
use alloc::boxed::Box;
use skein_lib::Writer;
use skein_oauth as oauth;

/// The authorization request's parameter that carries the identifier.
pub const NAME: &[u8] = b"ext_agent_host_id";
/// `urn:uuid:` and a UUID's 36 characters.
const VALUE_BYTES: usize = 45;
/// The parameter's name and value bytes, which skein's `parameter_bytes`
/// limit counts; every identifier formats to the same length.
pub const PARAMETER_BYTES: u32 = 62;

/// The identifier as a version 4 UUID's URN: its 16 drawn bytes with the
/// version and variant bits set, in lowercase hex.
#[must_use]
pub fn urn(id: &[u8; 16]) -> Box<[u8]> {
    let mut value = Writer::new(VALUE_BYTES);
    value.put(b"urn:uuid:").expect("sized for the URN");
    for (index, byte) in id.iter().enumerate() {
        // A dash before bytes 4, 6, 8 and 10; byte 6 holds the version,
        // byte 8 the variant.
        let (byte, dash) = match index {
            4 | 10 => (*byte, true),
            6 => ((byte & 0x0f) | 0x40, true),
            8 => ((byte & 0x3f) | 0x80, true),
            _ => (*byte, false),
        };
        if dash {
            value.put(b"-").expect("sized for the URN");
        }
        value.put(&[digit(byte / 16), digit(byte % 16)]).expect("sized for the URN");
    }
    value.finish()
}

fn digit(nibble: u8) -> u8 {
    *b"0123456789abcdef".get(usize::from(nibble)).expect("a nibble names a digit")
}

/// A sign-in registration's terms that carry only the identifier, sent on
/// every sign-in.
#[must_use]
pub fn terms(id: &[u8; 16]) -> oauth::Terms {
    oauth::Terms {
        parameters: Box::new([oauth::Parameter { name: NAME.into(), value: urn(id), sent: oauth::Sent::EverySignIn }]),
        openid: None,
        issued_client_id: false,
        varying_port: false,
        resource: None,
        required_scopes: None,
        revocation_endpoint: None,
        sign_in_again: Box::default(),
    }
}
