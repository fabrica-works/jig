//! A moved record's file (protocol/llm.md, 5.6): one account, opaque and versioned.
use alloc::boxed::Box;
use jig_domain_accounts::{DISPLAY, SUBJECT};
use skein_lib::{Reader, Secret, Writer, bytes};
use skein_oauth::{Limits, SavedToken};

pub const MAGIC: [u8; 4] = *b"JIGM";
pub const VERSION: u16 = 1;
/// The application's name, in bytes (nonempty).
pub const APPLICATION: u32 = 64;
/// The issuer URL, in bytes (nonempty).
pub const ISSUER: u32 = 2_048;
/// The refreshing client id, in bytes (nonempty).
pub const CLIENT_ID: u32 = 255;
// Magic, version and six u32 length prefixes.
const FIXED: u64 = 30;

/// A moved record's parts; holds a secret, so no Debug.
#[expect(missing_debug_implementations, reason = "carries a refresh token")]
pub struct Moved {
    pub application: Box<[u8]>,
    pub issuer: Box<[u8]>,
    pub client_id: Box<[u8]>,
    pub subject: Box<[u8]>,
    pub display: Box<[u8]>,
    pub record: SavedToken,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Unread {
    Magic,
    Version,
    Malformed,
    TooLarge,
    Trailing,
    Record,
}

/// The largest file under these limits, including its fixed parts.
#[must_use]
pub fn bound(limits: &Limits) -> u64 {
    let mut size = FIXED;
    for cap in [APPLICATION, ISSUER, CLIENT_ID, SUBJECT, DISPLAY, limits.record_bytes] {
        size = size.checked_add(u64::from(cap)).expect("six u32 bounds fit u64");
    }
    size
}

/// Encodes bounded parts and a record skein accepts, allocating the final size.
pub fn encode(moved: &Moved, limits: &Limits) -> Result<Secret, Unread> {
    let fields: [&[u8]; 5] = [&moved.application, &moved.issuer, &moved.client_id, &moved.subject, &moved.display];
    let mut size = FIXED;
    for (field, cap, empty) in [
        (moved.application.as_ref(), APPLICATION, false),
        (moved.issuer.as_ref(), ISSUER, false),
        (moved.client_id.as_ref(), CLIENT_ID, false),
        (moved.subject.as_ref(), SUBJECT, false),
        (moved.display.as_ref(), DISPLAY, true),
    ] {
        let len = u32::try_from(field.len()).or(Err(Unread::TooLarge))?;
        check(len, cap, empty)?;
        size = size.checked_add(u64::from(len)).ok_or(Unread::TooLarge)?;
    }
    let record = Secret::new(skein_oauth::encode_record(&moved.record, limits).or(Err(Unread::Record))?);
    let len = u32::try_from(record.len()).or(Err(Unread::TooLarge))?;
    check(len, limits.record_bytes, false)?;
    size = size.checked_add(u64::from(len)).ok_or(Unread::TooLarge)?;
    let mut out = Writer::new(usize::try_from(size).or(Err(Unread::TooLarge))?);
    out.put(&MAGIC).expect("file measured");
    out.put(&VERSION.to_be_bytes()).expect("file measured");
    for field in fields {
        write_part(&mut out, field);
    }
    write_part(&mut out, record.expose());
    Ok(Secret::new(out.finish()))
}

/// Decodes a whole file, checking its size before reading or allocating parts.
pub fn decode(input: &[u8], limits: &Limits) -> Result<Moved, Unread> {
    if u64::try_from(input.len()).or(Err(Unread::TooLarge))? > bound(limits) {
        return Err(Unread::TooLarge);
    }
    // Reader's cursor is limited to u32::MAX bytes; the whole file can be
    // larger when record_bytes is u32::MAX. Read only the fixed header here.
    let mut reader = Reader::new(input.get(..input.len().min(6)).ok_or(Unread::Malformed)?);
    if reader.bytes(4).ok_or(Unread::Malformed)? != MAGIC {
        return Err(Unread::Magic);
    }
    if reader.u16().ok_or(Unread::Malformed)? != VERSION {
        return Err(Unread::Version);
    }
    let rest = input.get(6..).ok_or(Unread::Malformed)?;
    let (application, rest) = read_part(rest, APPLICATION, false)?;
    let (issuer, rest) = read_part(rest, ISSUER, false)?;
    let (client_id, rest) = read_part(rest, CLIENT_ID, false)?;
    let (subject, rest) = read_part(rest, SUBJECT, false)?;
    let (display, rest) = read_part(rest, DISPLAY, true)?;
    let (record, rest) = read_part(rest, limits.record_bytes, false)?;
    if !rest.is_empty() {
        return Err(Unread::Trailing);
    }
    let record = skein_oauth::decode_record(record, limits).or(Err(Unread::Record))?;
    Ok(Moved {
        application: bytes::copy_of(application),
        issuer: bytes::copy_of(issuer),
        client_id: bytes::copy_of(client_id),
        subject: bytes::copy_of(subject),
        display: bytes::copy_of(display),
        record,
    })
}

fn check(len: u32, cap: u32, empty: bool) -> Result<(), Unread> {
    if len > cap {
        return Err(Unread::TooLarge);
    }
    if len == 0 && !empty {
        return Err(Unread::Malformed);
    }
    Ok(())
}

fn read_part(input: &[u8], cap: u32, empty: bool) -> Result<(&[u8], &[u8]), Unread> {
    let mut reader = Reader::new(input.get(..4).ok_or(Unread::Malformed)?);
    let len = reader.u32().ok_or(Unread::Malformed)?;
    check(len, cap, empty)?;
    let rest = input.get(4..).ok_or(Unread::Malformed)?;
    rest.split_at_checked(usize::try_from(len).or(Err(Unread::TooLarge))?).ok_or(Unread::Malformed)
}

fn write_part(out: &mut Writer, field: &[u8]) {
    out.put(&u32::try_from(field.len()).expect("bounded part").to_be_bytes()).expect("file measured");
    out.put(field).expect("file measured");
}
