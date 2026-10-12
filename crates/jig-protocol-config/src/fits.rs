//! Whole-process memory admission (configuration.md, 6; programming-model.md, 6.3).
use crate::What;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Share {
    pub name: &'static [u8],
    pub bytes: u64,
}
pub fn fits(memory: u64, shares: &[Share]) -> Result<u64, What> {
    let mut total = 0_u64;
    for share in shares {
        total = total.checked_add(share.bytes).ok_or(What::Bound)?;
    }
    match total.checked_sub(memory) {
        Some(over) if over > 0 => Err(What::Memory { over }),
        Some(_) | None => Ok(total),
    }
}
