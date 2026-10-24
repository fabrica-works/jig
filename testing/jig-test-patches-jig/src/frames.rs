//! Raw fixture frames, at either kept layout, shared by step and fuzz tests.
use alloc::{boxed::Box, vec::Vec};
pub const TASK: u64 = 21;
pub const ATTEMPT: u64 = 1;
pub const TURN: u32 = 1;
pub const SPENT: u64 = 7;

#[must_use]
pub fn body(version: u16, transcript: &[u8]) -> Box<[u8]> {
    let mut out = Vec::new();
    out.extend(version.to_be_bytes());
    out.push(1); // opens_window
    out.extend(TASK.to_be_bytes());
    out.extend(ATTEMPT.to_be_bytes());
    out.extend(TURN.to_be_bytes());
    if version == 5 {
        out.extend(u32::try_from(SPENT).expect("fixture spent").to_be_bytes());
    } else {
        out.extend(SPENT.to_be_bytes());
        out.push(0); // read: none
    }
    out.extend(0_u64.to_be_bytes()); // epoch
    out.extend(u32::try_from(transcript.len()).expect("fixture transcript").to_be_bytes());
    out.extend(transcript);
    out.into_boxed_slice()
}

#[must_use]
pub fn frame(version: u16, transcript: &[u8]) -> Box<[u8]> {
    wrap(&body(version, transcript))
}
#[must_use]
pub fn wrap(body: &[u8]) -> Box<[u8]> {
    let length = u32::try_from(body.len()).expect("fixture body").checked_add(12).expect("fixture frame");
    let mut out = Vec::new();
    out.extend(length.to_be_bytes());
    out.extend(body);
    out.extend(crc32c(body).to_be_bytes());
    out.extend(length.to_be_bytes());
    out.into_boxed_slice()
}
fn crc32c(bytes: &[u8]) -> u32 {
    let mut crc = !0_u32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0_u8..8 {
            crc = if crc & 1 == 0 { crc >> 1_u32 } else { (crc >> 1_u32) ^ 0x82f6_3b78 };
        }
    }
    !crc
}
