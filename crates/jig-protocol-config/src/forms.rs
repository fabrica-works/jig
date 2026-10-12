//! Private-file byte forms (configuration.md, 10); startup owns reading them.
use crate::What;
use alloc::boxed::Box;
use skein_io::digest::Digest;
use skein_lib::{Secret, bytes};
pub fn digest(input: &[u8]) -> Result<Digest, What> {
    let digits = input.strip_suffix(b"\n").unwrap_or(input);
    if digits.len() != 64 {
        return Err(What::Digest);
    }
    let mut words = [0_u64; 4];
    for (word, group) in words.iter_mut().zip(digits.chunks_exact(16)) {
        for digit in group {
            let value = match digit {
                b'0'..=b'9' => digit.checked_sub(b'0'),
                b'a'..=b'f' => {
                    Some(digit.checked_sub(b'a').expect("hex digit checked").checked_add(10).expect("hex value fits"))
                }
                _ => None,
            }
            .ok_or(What::Digest)?;
            *word = word.checked_mul(16).ok_or(What::Digest)?.checked_add(u64::from(value)).ok_or(What::Digest)?;
        }
    }
    Ok(Digest(words))
}
#[must_use]
pub fn secret(input: Box<[u8]>) -> Secret {
    match input.strip_suffix(b"\n") {
        Some(bytes) => Secret::new(bytes::copy_of(bytes)),
        None => Secret::new(input),
    }
}

/// A handed-in token's strict private-file form (configuration.md, 10).
#[derive(Debug)]
pub struct HandedIn {
    pub token: Secret,
    pub lapses: skein_lib::Wall,
    pub account_id: Option<Box<[u8]>>,
}
pub fn handed_in(bytes: &[u8], limits: &crate::DocumentLimits) -> Result<HandedIn, crate::Problems> {
    let mut problems = crate::Problems::empty();
    let document = match crate::document(bytes, limits) {
        Ok(document) => document,
        Err(problem) => {
            problems.push(problem.path, problem.what);
            return Err(problems);
        }
    };
    let path = crate::Path::root();
    let Some(mut object) = crate::check::object(&document, 0, &path, &mut problems) else {
        return Err(problems);
    };
    let mut token = None;
    if let Some(at) = object.required(&document, b"access_token", &path, &mut problems) {
        match crate::read::text(&document, at, limits.bytes) {
            Ok(value) => token = Some(value),
            Err(what) => problems.push(path.member(b"access_token"), what),
        }
    }
    let expires_at = crate::check::integer(&mut object, &document, b"expires_at", &path, &mut problems);
    let mut lapses = None;
    if let Some(seconds) = expires_at {
        match seconds.checked_mul(1_000_000_000) {
            Some(nanos) => lapses = Some(skein_lib::Wall::from_nanos(nanos)),
            None => problems.push(path.member(b"expires_at"), What::Bound),
        }
    }
    let mut account_id = None;
    if let Some(at) = object.member(&document, b"account_id") {
        match crate::read::text(&document, at, limits.bytes) {
            Ok(value) => account_id = Some(value),
            Err(what) => problems.push(path.member(b"account_id"), what),
        }
    }
    object.finish(&document, &path, &mut problems);
    if !problems.is_empty() {
        return Err(problems);
    }
    Ok(HandedIn {
        token: Secret::new(token.expect("required token checked")),
        lapses: lapses.expect("required wall time checked"),
        account_id,
    })
}
