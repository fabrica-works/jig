//! A versioned turn with lengths and CRC-32C (protocol/store.md, 8.1).
use crate::codec;
use alloc::boxed::Box;
use jig_domain_core as core;
use skein_lib::{Reader, Writer};
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Unread {
    Short,
    Lengths,
    Checksum,
    Record,
}
#[must_use]
pub fn encode(turn: &core::TurnRecord, limits: &codec::Limits) -> Option<Box<[u8]>> {
    let body = codec::core::turn(turn, &codec::core_limits(limits)).ok()?;
    let length = u32::try_from(body.len()).ok()?.checked_add(12)?;
    let mut writer = Writer::new(usize::try_from(length).ok()?);
    writer.put(&length.to_be_bytes()).ok()?;
    writer.put(&body).ok()?;
    writer.put(&crc32c(&body).to_be_bytes()).ok()?;
    writer.put(&length.to_be_bytes()).ok()?;
    Some(writer.finish())
}
/// Decode exactly one whole frame, refusing trailing bytes as well.
pub fn decode(bytes: &[u8], limits: &codec::Limits) -> Result<core::TurnRecord, Unread> {
    if u32::try_from(bytes.len()).is_err() {
        return Err(Unread::Lengths);
    }
    let mut reader = Reader::new(bytes);
    let length = reader.u32().ok_or(Unread::Short)?;
    let body_length = length.checked_sub(12).ok_or(Unread::Lengths)?;
    let body = reader.bytes(body_length).ok_or(Unread::Short)?;
    let checksum = reader.u32().ok_or(Unread::Short)?;
    let trailing = reader.u32().ok_or(Unread::Short)?;
    if trailing != length || !reader.is_empty() {
        return Err(Unread::Lengths);
    }
    if crc32c(body) != checksum {
        return Err(Unread::Checksum);
    }
    match codec::core::read_turn(body, &codec::core_limits(limits)) {
        Ok(turn) => Ok(turn),
        Err(_) => Err(Unread::Record),
    }
}
#[must_use]
pub fn crc32c(bytes: &[u8]) -> u32 {
    let mut crc = !0_u32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0_u8..8 {
            crc = if crc & 1 == 0 { crc >> 1_u32 } else { (crc >> 1_u32) ^ 0x82f6_3b78 };
        }
    }
    !crc
}
