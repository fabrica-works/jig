//! The printed link and the shell's own exchange outcomes (protocol/client.md, 8).
use jig_protocol_client_wire::v7 as w;
use skein_lib::{
    Reader, Secret, Token, Writer,
    base64::{self, Alphabet},
};
#[derive(Debug, PartialEq, Eq)]
pub enum Unlinked {
    NotHttp,
    NoSecret,
    Address,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Signing {
    SignedIn { exchange: Token, party: u64 },
    Again { exchange: Token },
    Refused { exchange: Token },
    Lost { exchange: Token },
    SignedOut { exchange: Token },
}
pub(crate) fn parse(link: &[u8]) -> Result<(String, Box<[u8]>), Unlinked> {
    let text = std::str::from_utf8(link).map_err(|_| Unlinked::Address)?;
    let rest = text.strip_prefix("http://").ok_or(Unlinked::NotHttp)?;
    let (host, path) = rest.split_once('/').ok_or(Unlinked::NoSecret)?;
    let _: std::net::SocketAddr = host.parse().map_err(|_| Unlinked::Address)?;
    let secret = path.strip_prefix("jig/v1/sign-in/link?secret=").ok_or(Unlinked::NoSecret)?;
    let secret = base64::decode(Alphabet::Url, secret.as_bytes(), 32).map_err(|_| Unlinked::NoSecret)?;
    if secret.len() != 32 {
        return Err(Unlinked::NoSecret);
    }
    let value = w::LinkSignIn::new(&w::CEILINGS, w::LinkSignInParts { secret }).expect("32-byte link secret");
    let mut writer = Writer::new(usize::try_from(value.measure()).expect("short record"));
    value.encode(&mut writer).expect("measured record");
    Ok((host.to_owned(), writer.finish()))
}
pub(crate) fn answer(exchange: Token, status: u16, body: &[u8], sign_out: bool, token: &mut Option<Secret>) -> Signing {
    if status != 200 {
        return Signing::Lost { exchange };
    }
    let mut reader = Reader::new(body);
    if sign_out {
        match w::SignOutAnswer::decode(&w::CEILINGS, &mut reader) {
            Ok(w::SignOutAnswer::SignedOut) if reader.remaining() == 0 => {
                *token = None;
                Signing::SignedOut { exchange }
            }
            Ok(w::SignOutAnswer::NotReady) if reader.remaining() == 0 => Signing::Again { exchange },
            Ok(_) | Err(_) => Signing::Lost { exchange },
        }
    } else {
        match w::SignInAnswer::decode(&w::CEILINGS, &mut reader) {
            Ok(w::SignInAnswer::SignedIn(value)) if reader.remaining() == 0 && value.token().len() == 40 => {
                let fields = value.into_parts();
                *token = Some(fields.token);
                Signing::SignedIn { exchange, party: fields.party }
            }
            Ok(w::SignInAnswer::NotReady) if reader.remaining() == 0 => Signing::Again { exchange },
            Ok(w::SignInAnswer::Refused) if reader.remaining() == 0 => Signing::Refused { exchange },
            Ok(_) | Err(_) => Signing::Lost { exchange },
        }
    }
}
