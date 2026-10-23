//! Deployment `OpenID` provider configuration (configuration.md, 3).
use crate::{Directories, Directory, Path, Problem, Problems, What, check, read};
use alloc::boxed::Box;
use skein_json::{Document, document::Kind};
use skein_lib::{
    List, Secret, Writer,
    base64::{self, Alphabet},
};
#[derive(Debug)]
pub struct Provider {
    /// Data path to the private PKCS#8 PEM file.
    pub key: Box<[u8]>,
    pub clients: Box<[ProviderClient]>,
    pub addresses: Box<[u16]>,
}
#[derive(Debug)]
pub struct ProviderClient {
    pub id: Box<[u8]>,
    /// Data path, never the secret itself.
    pub secret: Box<[u8]>,
    pub redirect_uri: Box<[u8]>,
}
pub(super) fn parse(
    object: &mut read::Object,
    document: &Document,
    path: &Path,
    directories: &Directories,
    problems: &mut Problems,
) -> Option<Provider> {
    let at = object.member(document, b"provider")?;
    if document.token(at)?.kind == Kind::Null {
        return None;
    }
    let path = path.member(b"provider");
    let mut object = check::object(document, at, &path, problems)?;
    let key = check::path(&mut object, document, b"key", &path, Directory::Data, directories, problems);
    let mut clients = List::<ProviderClient>::with_capacity(document.len());
    if let Some(items) = check::list(&mut object, document, b"clients", document.len(), &path, problems) {
        if items.is_empty() {
            problems.push(path.member(b"clients"), What::Bound);
        }
        for (index, at) in items.iter().enumerate() {
            let path = check::index(&path.member(b"clients"), index);
            if let Some(client) = client(document, *at, &path, directories, problems) {
                for previous in &clients {
                    if previous.id == client.id {
                        problems.push(path.member(b"id"), What::Unique);
                    }
                }
                clients.push(client).expect("document bounds client count");
            }
        }
    }
    let mut addresses = List::with_capacity(document.len());
    if let Some(items) = check::list(&mut object, document, b"addresses", document.len(), &path, problems) {
        for (index, at) in items.iter().enumerate() {
            match read::integer(document, *at, u64::from(u16::MAX)) {
                Ok(number) => addresses
                    .push(u16::try_from(number).expect("bounded provider number"))
                    .expect("document bounds addresses"),
                Err(what) => problems.push(check::index(&path.member(b"addresses"), index), what),
            }
        }
    }
    object.finish(document, &path, problems);
    Some(Provider { key: key?, clients: clients.into_boxed(), addresses: addresses.into_boxed() })
}
fn client(
    document: &Document,
    at: u32,
    path: &Path,
    directories: &Directories,
    problems: &mut Problems,
) -> Option<ProviderClient> {
    let mut object = check::object(document, at, path, problems)?;
    let id = check::text(&mut object, document, b"id", path, problems);
    if let Some(id) = &id {
        if id.is_empty() || id.len() > 255 {
            problems.push(path.member(b"id"), What::Bound);
        }
        for byte in id {
            if !(0x21..=0x7e).contains(byte) {
                problems.push(path.member(b"id"), What::Form);
                break;
            }
        }
    }
    let secret = check::path(&mut object, document, b"secret", path, Directory::Data, directories, problems);
    let redirect_uri = check::text(&mut object, document, b"redirect_uri", path, problems);
    if let Some(uri) = &redirect_uri {
        if uri.len() > 2048 {
            problems.push(path.member(b"redirect_uri"), What::Bound);
        } else if !redirect(uri) {
            problems.push(path.member(b"redirect_uri"), What::Form);
        }
    }
    object.finish(document, path, problems);
    Some(ProviderClient { id: id?, secret: secret?, redirect_uri: redirect_uri? })
}
fn redirect(uri: &[u8]) -> bool {
    let (rest, https) = if let Some(rest) = uri.strip_prefix(b"https://") {
        (rest, true)
    } else if let Some(rest) = uri.strip_prefix(b"http://") {
        (rest, false)
    } else {
        return false;
    };
    for byte in rest {
        if *byte <= b' ' || *byte >= 127 || [b'#', b'\\', b'"', b'\'', b'<', b'>'].contains(byte) {
            return false;
        }
    }
    let mut end = rest.len();
    for (index, byte) in rest.iter().enumerate() {
        if *byte == b'/' || *byte == b'?' {
            end = index;
            break;
        }
    }
    let authority = rest.get(..end).expect("authority prefix");
    if !crate::llm::providers::public_authority(authority) {
        return false;
    }
    if https {
        return true;
    }
    for host in [b"127.0.0.1:".as_slice(), b"[::1]:", b"localhost:"] {
        if let Some(port) = authority.strip_prefix(host) {
            return !port.is_empty();
        }
    }
    false
}
/// Read RFC 7468 PKCS#8 armor and require a 3072-bit RSA key.
/// A failure identifies the configuration member and how to generate its file, never its contents.
pub fn provider_key(pem: &[u8]) -> Result<Secret, Problem> {
    let problem = Problem {
        path: Path::root().member(b"clients").member(b"provider").member(b"key"),
        what: What::ProviderKey { generate: "openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:3072" },
    };
    let pem = trim(pem);
    let body = pem.strip_prefix(b"-----BEGIN PRIVATE KEY-----").ok_or(problem.clone())?;
    let body = strip_line(body).ok_or(problem.clone())?;
    let body = body.strip_suffix(b"-----END PRIVATE KEY-----").ok_or(problem.clone())?;
    if !body.ends_with(b"\n") && !body.ends_with(b"\r") {
        return Err(problem);
    }
    let mut length = 0_usize;
    for byte in body {
        if !armor_space(*byte) {
            length = length.checked_add(1).expect("base64 body fits input");
        }
    }
    let mut compact = Writer::new(length);
    for byte in body {
        if !armor_space(*byte) {
            compact.put(&[*byte]).expect("measured base64 body");
        }
    }
    let compact = compact.finish();
    if compact.is_empty() {
        return Err(problem);
    }
    let Ok(max) = u32::try_from(compact.len()) else { return Err(problem) };
    match base64::decode(Alphabet::Standard, &compact, max) {
        Ok(bytes) => {
            let public = skein_tls::rs256_public(&bytes).ok_or(problem.clone())?;
            if public.modulus.len() != 384 || public.modulus.first().copied().unwrap_or(0) < 0x80 {
                return Err(problem);
            }
            Ok(Secret::new(bytes))
        }
        Err(_) => Err(problem),
    }
}
fn strip_line(body: &[u8]) -> Option<&[u8]> {
    if let Some(body) = body.strip_prefix(b"\r\n") {
        return Some(body);
    }
    if let Some(body) = body.strip_prefix(b"\r") {
        return Some(body);
    }
    body.strip_prefix(b"\n")
}
fn trim(body: &[u8]) -> &[u8] {
    let mut start = 0_usize;
    let mut end = body.len();
    for byte in body {
        if byte.is_ascii_whitespace() {
            start = start.saturating_add(1);
        } else {
            break;
        }
    }
    for byte in body.iter().rev() {
        if end > start && byte.is_ascii_whitespace() {
            end = end.saturating_sub(1);
        } else {
            break;
        }
    }
    body.get(start..end).expect("trimmed slice")
}

fn armor_space(byte: u8) -> bool {
    [b' ', b'\t', b'\r', b'\n', 0x0b, 0x0c].contains(&byte)
}
