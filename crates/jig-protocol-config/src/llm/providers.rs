//! LLM providers: each one's number, its name for showing, the configured models it serves and
//! the sources it offers (configuration.md, 3 and 7; accounts.md, 3.4). No account, key or record.
use crate::{Directories, Path, Problems, What, check, read};
use alloc::boxed::Box;
use jig_domain_core::Model;
use skein_json::Document;
use skein_lib::List;

/// One configured LLM provider.
#[derive(Debug)]
pub struct Provider {
    /// Its name for showing.
    pub name: Box<[u8]>,
    pub number: u32,
    pub signed_in_plan: bool,
    pub manage_usage: Option<Box<[u8]>>,
    /// Configured models' names; each configured model is in one provider.
    pub models: Box<[Box<[u8]>]>,
    /// The sign-in it offers, with its public registration.
    pub sign_in: Option<Registration>,
    /// Whether it offers a key.
    pub keys: bool,
    /// The programs it offers, such as `claude_code`; parsed only.
    pub programs: Box<[Box<[u8]>]>,
}

/// A provider's public registration for its sign-in: a client secret is refused.
#[derive(Debug)]
pub struct Registration {
    pub issuer: Box<[u8]>,
    pub client: Box<[u8]>,
    pub scopes: Box<[Box<[u8]>]>,
    pub name_hint: Option<Box<[u8]>>,
    pub authorization: Box<[u8]>,
    pub token: Box<[u8]>,
    pub revocation: Option<Box<[u8]>>,
    pub keys: Box<[u8]>,
    pub endpoint_transport: super::Transport,
    pub must_grant: Box<[Box<[u8]>]>,
    pub resource: Option<Box<[u8]>>,
    pub redirect_path: Box<[u8]>,
    pub redirect_port: Option<u16>,
}

/// The sources a provider offers, each optional: `{}` offers none.
#[derive(Debug)]
struct Sources {
    sign_in: Option<Registration>,
    keys: bool,
    programs: Box<[Box<[u8]>]>,
}

/// The longest program name (accounts.md, 3.4).
const PROGRAM: usize = 32;

/// Every configured model belongs to exactly one provider, even when none are named.
pub(super) fn parse(
    object: &mut read::Object,
    document: &Document,
    path: &Path,
    models: &[Model],
    directories: &Directories,
    problems: &mut Problems,
) -> List<Provider> {
    let mut providers = List::<Provider>::with_capacity(document.len());
    let Some(at) = object.member(document, b"providers") else {
        for (index, _) in models.iter().enumerate() {
            problems.push(check::index(&path.member(b"models"), index).member(b"name"), What::NoProvider);
        }
        return providers;
    };
    let items = match read::list(document, at, document.len()) {
        Ok(items) => items,
        Err(what) => {
            problems.push(path.member(b"providers"), what);
            return providers;
        }
    };
    let mut claimed = List::<bool>::with_capacity(u32::try_from(models.len()).expect("models bounded by u32"));
    for _ in models {
        claimed.push(false).expect("one flag per model");
    }
    for (index, at) in items.iter().enumerate() {
        let path = check::index(&path.member(b"providers"), index);
        if let Some(value) = provider(document, *at, &path, models, &mut claimed, directories, problems) {
            for previous in &providers {
                if previous.name == value.name {
                    problems.push(path.member(b"name"), What::Unique);
                }
                if previous.number == value.number {
                    problems.push(path.member(b"number"), What::Unique);
                }
            }
            providers.push(value).expect("input count bounded");
        }
    }
    for (index, claimed) in claimed.iter().enumerate() {
        if !claimed {
            problems.push(check::index(&path.member(b"models"), index).member(b"name"), What::NoProvider);
        }
    }
    providers
}

fn provider(
    document: &Document,
    at: u32,
    path: &Path,
    models: &[Model],
    claimed: &mut List<bool>,
    directories: &Directories,
    problems: &mut Problems,
) -> Option<Provider> {
    let mut object = check::object(document, at, path, problems)?;
    let name = public_text(&mut object, document, b"name", jig_domain_accounts::PROVIDER_NAME, path, problems);
    let signed_at = object.member(document, b"signed_in_plan");
    let signed_in_plan = match signed_at {
        Some(at) => match read::boolean(document, at) {
            Ok(value) => value,
            Err(what) => {
                problems.push(path.member(b"signed_in_plan"), what);
                false
            }
        },
        None => false,
    };
    let manage_usage = match object.member(document, b"manage_usage") {
        Some(at) => match read::text(document, at, jig_domain_accounts::MANAGE_USAGE) {
            Ok(value) if public_address(&value) => Some(value),
            Ok(_) => {
                problems.push(path.member(b"manage_usage"), What::Form);
                None
            }
            Err(what) => {
                problems.push(path.member(b"manage_usage"), what);
                None
            }
        },
        None => None,
    };
    let number = check::count(&mut object, document, b"number", path, problems);
    let mut served = None;
    if let Some(items) = check::list(&mut object, document, b"models", document.len(), path, problems) {
        served = served_models(document, &items, &path.member(b"models"), models, claimed, problems);
    }
    let mut offered = None;
    if let Some(at) = object.required(document, b"sources", path, problems) {
        offered = sources(document, at, &path.member(b"sources"), directories, problems);
    }
    object.finish(document, path, problems);
    let offered = offered?;
    if offered.sign_in.is_some() && signed_at.is_none() {
        problems.push(path.member(b"signed_in_plan"), What::Missing);
    }
    if signed_in_plan && offered.sign_in.is_none() {
        problems.push(path.member(b"signed_in_plan"), What::Form);
    }
    if (signed_in_plan || !offered.programs.is_empty()) && manage_usage.is_none() {
        problems.push(path.member(b"manage_usage"), What::Missing);
    }
    Some(Provider {
        name: name?,
        number: number?,
        signed_in_plan,
        manage_usage,
        models: served?,
        sign_in: offered.sign_in,
        keys: offered.keys,
        programs: offered.programs,
    })
}

/// Each name a configured model, claimed by this provider alone.
fn served_models(
    document: &Document,
    items: &List<u32>,
    path: &Path,
    models: &[Model],
    claimed: &mut List<bool>,
    problems: &mut Problems,
) -> Option<Box<[Box<[u8]>]>> {
    let mut names = List::<Box<[u8]>>::with_capacity(items.len());
    let mut failed = false;
    for (index, at) in items.iter().enumerate() {
        let path = check::index(path, index);
        let name = match read::text(document, *at, 4_096) {
            Ok(name) => name,
            Err(what) => {
                problems.push(path, what);
                failed = true;
                continue;
            }
        };
        let mut found = None;
        for (model, value) in models.iter().enumerate() {
            if value.name == name {
                found = Some(u32::try_from(model).expect("models bounded by u32"));
                break;
            }
        }
        match found {
            Some(model) => {
                let flag = claimed.get_mut(model).expect("one flag per model");
                if *flag {
                    problems.push(path, What::Unique);
                    failed = true;
                }
                *flag = true;
            }
            None => {
                problems.push(path, What::Refers);
                failed = true;
            }
        }
        names.push(name).expect("input count bounded");
    }
    if failed {
        return None;
    }
    Some(names.into_boxed())
}

/// The sources offered: `sign_in`, `keys` and `programs`, each optional.
fn sources(
    document: &Document,
    at: u32,
    path: &Path,
    directories: &Directories,
    problems: &mut Problems,
) -> Option<Sources> {
    let mut object = check::object(document, at, path, problems)?;
    let mut failed = false;
    let mut sign_in = None;
    if let Some(at) = object.member(document, b"sign_in") {
        sign_in = registration(document, at, &path.member(b"sign_in"), directories, problems);
        if sign_in.is_none() {
            failed = true;
        }
    }
    let mut keys = false;
    if let Some(at) = object.member(document, b"keys") {
        match read::boolean(document, at) {
            Ok(value) => keys = value,
            Err(what) => {
                problems.push(path.member(b"keys"), what);
                failed = true;
            }
        }
    }
    let mut programs = Some(List::<Box<[u8]>>::with_capacity(0));
    if let Some(at) = object.member(document, b"programs") {
        programs = program_names(document, at, &path.member(b"programs"), problems);
    }
    object.finish(document, path, problems);
    let programs = programs?;
    if failed {
        return None;
    }
    Some(Sources { sign_in, keys, programs: programs.into_boxed() })
}

/// A public registration: a client secret makes it confidential, which is refused
/// (configuration.md, 7).
fn registration(
    document: &Document,
    at: u32,
    path: &Path,
    directories: &Directories,
    problems: &mut Problems,
) -> Option<Registration> {
    let mut object = check::object(document, at, path, problems)?;
    let issuer = present(check::text(&mut object, document, b"issuer", path, problems), b"issuer", path, problems);
    let client = present(check::text(&mut object, document, b"client", path, problems), b"client", path, problems);
    let mut scopes = None;
    if let Some(items) = check::list(&mut object, document, b"scopes", document.len(), path, problems) {
        scopes = scope_tokens(document, &items, &path.member(b"scopes"), problems);
    }
    let name_hint = super::optional_text(&mut object, document, b"name_hint", path, problems);
    let authorization =
        present(check::text(&mut object, document, b"authorization", path, problems), b"authorization", path, problems);
    let token = present(check::text(&mut object, document, b"token", path, problems), b"token", path, problems);
    let revocation = super::optional_text(&mut object, document, b"revocation", path, problems);
    let keys = present(check::text(&mut object, document, b"keys", path, problems), b"keys", path, problems);
    let endpoint_transport = super::transport(&mut object, document, path, b"token", None, directories, problems);
    let mut must_grant = None;
    if let Some(items) = check::list(&mut object, document, b"must_grant", document.len(), path, problems) {
        must_grant = scope_tokens(document, &items, &path.member(b"must_grant"), problems);
    }
    if let (Some(scopes), Some(required)) = (&scopes, &must_grant) {
        for (index, scope) in required.iter().enumerate() {
            if !scopes.contains(scope) {
                problems.push(check::index(&path.member(b"must_grant"), index), What::Refers);
            }
        }
    }
    let resource = super::optional_text(&mut object, document, b"resource", path, problems);
    let redirect_path = check::text(&mut object, document, b"redirect_path", path, problems);
    if let Some(value) = &redirect_path
        && (value.is_empty() || value.starts_with(b"/") || value.contains(&b'?') || value.contains(&b'#'))
    {
        problems.push(path.member(b"redirect_path"), What::Form);
    }
    let mut redirect_port = None;
    if let Some(at) = object.member(document, b"redirect_port") {
        match read::integer(document, at, u64::from(u16::MAX)) {
            Ok(value) if value > 0 => redirect_port = Some(u16::try_from(value).expect("port bounded")),
            Ok(_) => problems.push(path.member(b"redirect_port"), What::Form),
            Err(what) => problems.push(path.member(b"redirect_port"), what),
        }
    }
    let secret = object.member(document, b"secret");
    if secret.is_some() {
        problems.push(path.member(b"secret"), What::Confidential);
    }
    object.finish(document, path, problems);
    if secret.is_some() {
        return None;
    }
    Some(Registration {
        issuer: issuer?,
        client: client?,
        scopes: scopes?,
        name_hint,
        authorization: authorization?,
        token: token?,
        revocation,
        keys: keys?,
        endpoint_transport: endpoint_transport?,
        must_grant: must_grant?,
        resource,
        redirect_path: redirect_path?,
        redirect_port,
    })
}

/// RFC 6749, 3.3: a scope token is one or more of `%x21 / %x23-5B / %x5D-7E`.
fn scope_tokens(
    document: &Document,
    items: &List<u32>,
    path: &Path,
    problems: &mut Problems,
) -> Option<Box<[Box<[u8]>]>> {
    let mut scopes = List::<Box<[u8]>>::with_capacity(items.len());
    let mut failed = false;
    for (index, at) in items.iter().enumerate() {
        let path = check::index(path, index);
        match read::text(document, *at, 4_096) {
            Ok(scope) => {
                let mut token = !scope.is_empty();
                for byte in &scope {
                    match byte {
                        0x21 | 0x23..=0x5B | 0x5D..=0x7E => {}
                        _ => token = false,
                    }
                }
                if token {
                    scopes.push(scope).expect("input count bounded");
                } else {
                    problems.push(path, What::Form);
                    failed = true;
                }
            }
            Err(what) => {
                problems.push(path, what);
                failed = true;
            }
        }
    }
    if failed {
        return None;
    }
    Some(scopes.into_boxed())
}

/// Program names, unique, each 1 to 32 of `a-z`, `0-9`, `-` and `_`: `claude_code` names Claude
/// Code (smith's programs.md, 3).
fn program_names(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<List<Box<[u8]>>> {
    let items = match read::list(document, at, document.len()) {
        Ok(items) => items,
        Err(what) => {
            problems.push(path.clone(), what);
            return None;
        }
    };
    let mut names = List::<Box<[u8]>>::with_capacity(items.len());
    let mut failed = false;
    for (index, at) in items.iter().enumerate() {
        let path = check::index(path, index);
        match read::text(document, *at, 4_096) {
            Ok(name) => {
                let mut form = (1..=PROGRAM).contains(&name.len());
                for byte in &name {
                    match byte {
                        b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' => {}
                        _ => form = false,
                    }
                }
                if !form {
                    problems.push(path, What::Form);
                    failed = true;
                } else if names.as_slice().contains(&name) {
                    problems.push(path, What::Unique);
                    failed = true;
                } else {
                    names.push(name).expect("input count bounded");
                }
            }
            Err(what) => {
                problems.push(path, what);
                failed = true;
            }
        }
    }
    if failed {
        return None;
    }
    Some(names)
}

/// A required text that is not empty.
fn present(value: Option<Box<[u8]>>, name: &[u8], path: &Path, problems: &mut Problems) -> Option<Box<[u8]>> {
    let value = value?;
    if value.is_empty() {
        problems.push(path.member(name), What::Form);
        return None;
    }
    Some(value)
}

/// Text is nonempty, UTF-8, and free of control characters; diagnostics name its exact field.
#[expect(clippy::disallowed_methods, reason = "configuration validates public UTF-8 text (configuration.md, 3)")]
fn public_text(
    object: &mut read::Object,
    document: &Document,
    name: &[u8],
    max: u32,
    path: &Path,
    problems: &mut Problems,
) -> Option<Box<[u8]>> {
    let at = object.required(document, name, path, problems)?;
    let value = match read::text(document, at, max) {
        Ok(value) => value,
        Err(what) => {
            problems.push(path.member(name), what);
            return None;
        }
    };
    if value.is_empty() || core::str::from_utf8(&value).is_err() {
        problems.push(path.member(name), What::Form);
        return None;
    }
    // Match complete UTF-8 encodings of controls and invisible display formatting.
    let mut before = 0_u8;
    let mut earlier = 0_u8;
    for byte in &value {
        let format = (before == 0xd8 && *byte == 0x9c)
            || (before == 0xc2 && *byte == 0xad)
            || (earlier == 0xe2 && before == 0x80 && ((0x8b..=0x8f).contains(byte) || (0xaa..=0xae).contains(byte)))
            || (earlier == 0xe2 && before == 0x81 && (0xa0..=0xaf).contains(byte))
            || (earlier == 0xe1 && before == 0xa0 && *byte == 0x8e)
            || (earlier == 0xef && before == 0xbb && *byte == 0xbf)
            || (earlier == 0xef && before == 0xbf && (0xb9..=0xbb).contains(byte));
        if *byte < b' ' || *byte == 0x7f || (before == 0xc2 && (0x80..=0x9f).contains(byte)) || format {
            problems.push(path.member(name), What::Form);
            return None;
        }
        earlier = before;
        before = *byte;
    }
    Some(value)
}

/// Public HTTPS address with an ASCII authority and no credentials, query or fragment.
fn public_address(value: &[u8]) -> bool {
    let Some(rest) = value.strip_prefix(b"https://") else {
        return false;
    };
    for byte in rest {
        if *byte <= b' ' || *byte >= 127 || [b'@', b'?', b'#', b'\\', b'"', b'\'', b'<', b'>'].contains(byte) {
            return false;
        }
    }
    let end = match skein_lib::bytes::find(rest, b"/") {
        Some(end) => end,
        None => rest.len(),
    };
    public_authority(rest.get(..end).expect("address authority prefix"))
}

#[derive(Clone, Copy)]
enum LabelByte {
    Label,
    Dash,
    Dot,
    Other,
}

#[expect(clippy::disallowed_methods, reason = "configuration parses an address's IPv6 literal (configuration.md, 3)")]
fn public_authority(authority: &[u8]) -> bool {
    if let Some(rest) = authority.strip_prefix(b"[") {
        let Some(end) = skein_lib::bytes::find(rest, b"]") else {
            return false;
        };
        let host = rest.get(..end).expect("IPv6 host prefix");
        let Ok(host) = core::str::from_utf8(host) else {
            return false;
        };
        if host.parse::<core::net::Ipv6Addr>().is_err() {
            return false;
        }
        return public_port(rest.get(end.saturating_add(1)..).expect("suffix after bracket"));
    }
    let end = match skein_lib::bytes::find(authority, b":") {
        Some(end) => end,
        None => authority.len(),
    };
    let host = authority.get(..end).expect("host prefix");
    let mut label = 0_u32;
    let mut dash = false;
    for byte in host {
        let class = if byte.is_ascii_alphanumeric() {
            LabelByte::Label
        } else if *byte == b'-' {
            LabelByte::Dash
        } else if *byte == b'.' {
            LabelByte::Dot
        } else {
            LabelByte::Other
        };
        match class {
            LabelByte::Label => {
                label = label.saturating_add(1);
                dash = false;
            }
            LabelByte::Dash => {
                if label == 0 {
                    return false;
                }
                label = label.saturating_add(1);
                dash = true;
            }
            LabelByte::Dot => {
                if label == 0 || dash {
                    return false;
                }
                label = 0;
            }
            LabelByte::Other => return false,
        }
        if label > 63 {
            return false;
        }
    }
    label > 0 && !dash && public_port(authority.get(end..).expect("port suffix"))
}
fn public_port(suffix: &[u8]) -> bool {
    if suffix.is_empty() {
        return true;
    }
    let Some(port) = suffix.strip_prefix(b":") else {
        return false;
    };
    let mut value = 0_u32;
    for byte in port {
        if !byte.is_ascii_digit() {
            return false;
        }
        let digit = u32::from(byte.checked_sub(b'0').expect("ASCII digit"));
        let Some(next) = value.checked_mul(10) else {
            return false;
        };
        let Some(next) = next.checked_add(digit) else {
            return false;
        };
        value = next;
    }
    (1..=65_535).contains(&value)
}
