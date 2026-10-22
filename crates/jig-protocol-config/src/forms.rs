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
