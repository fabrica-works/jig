//! Index-based strict readers (configuration.md, 1). No borrowed state is retained.
use crate::{Path, Problems, What};
use alloc::boxed::Box;
use skein_json::{Document, document::Kind};
use skein_lib::{Duration, List, bytes};

/// The first token after one value, including a container's closing token.
pub fn after(document: &Document, at: u32) -> Result<u32, What> {
    let mut depth = 0_u32;
    for index in at..document.len() {
        match document.token(index).ok_or(What::Kind)?.kind {
            Kind::ObjectStart | Kind::ArrayStart => depth = depth.checked_add(1).ok_or(What::Bound)?,
            Kind::ObjectEnd | Kind::ArrayEnd => depth = depth.checked_sub(1).ok_or(What::Kind)?,
            Kind::Key | Kind::String | Kind::Number | Kind::True | Kind::False | Kind::Null | Kind::Long => {}
        }
        if depth == 0 {
            return index.checked_add(1).ok_or(What::Bound);
        }
    }
    Err(What::Kind)
}

#[derive(Debug)]
struct Member {
    key: u32,
    value: u32,
    read: bool,
}
/// One object's member indices and which the caller consumed.
#[derive(Debug)]
pub struct Object {
    members: List<Member>,
}
impl Object {
    pub fn new(document: &Document, at: u32) -> Result<Self, What> {
        if document.token(at).ok_or(What::Kind)?.kind != Kind::ObjectStart {
            return Err(What::Kind);
        }
        let end = after(document, at)?.checked_sub(1).ok_or(What::Kind)?;
        let mut members = List::with_capacity(end.checked_sub(at).ok_or(What::Kind)?);
        let mut next = at.checked_add(1).ok_or(What::Bound)?;
        for _ in at..end {
            if next == end {
                return Ok(Self { members });
            }
            if document.token(next).ok_or(What::Kind)?.kind != Kind::Key {
                return Err(What::Kind);
            }
            let value = next.checked_add(1).ok_or(What::Bound)?;
            members.push(Member { key: next, value, read: false }).or(Err(What::Bound))?;
            next = after(document, value)?;
        }
        Err(What::Kind)
    }
    /// Read every occurrence of this name, retaining the first for value decoding.
    pub fn member(&mut self, document: &Document, name: &[u8]) -> Option<u32> {
        let mut first = None;
        for index in 0..self.members.len() {
            let member = self.members.get_mut(index)?;
            let key = document.token(member.key)?;
            if document.text(key)? == name {
                member.read = true;
                if first.is_none() {
                    first = Some(member.value);
                }
            }
        }
        first
    }
    pub fn required(&mut self, document: &Document, name: &[u8], path: &Path, problems: &mut Problems) -> Option<u32> {
        let at = self.member(document, name);
        if at.is_none() {
            problems.push(path.member(name), What::Missing);
        }
        at
    }
    pub fn finish(self, document: &Document, path: &Path, problems: &mut Problems) {
        for (index, member) in self.members.iter().enumerate() {
            let name = document.text(document.token(member.key).expect("object key index")).expect("object key text");
            if !member.read {
                problems.push(path.member(name), What::Unknown);
            }
            for previous in self.members.iter().take(index) {
                let key = document.token(previous.key).expect("object key index");
                if document.text(key) == Some(name) {
                    problems.push(path.member(name), What::Duplicate);
                    break;
                }
            }
        }
    }
}

pub fn text(document: &Document, at: u32, max: u32) -> Result<Box<[u8]>, What> {
    let token = document.token(at).ok_or(What::Kind)?;
    if token.kind != Kind::String {
        return Err(What::Kind);
    }
    if token.len > max {
        return Err(What::Bound);
    }
    Ok(bytes::copy_of(document.text(token).ok_or(What::Kind)?))
}
pub fn integer(document: &Document, at: u32, max: u64) -> Result<u64, What> {
    let token = document.token(at).ok_or(What::Kind)?;
    if token.kind != Kind::Number {
        return Err(What::Kind);
    }
    let mut value = 0_u64;
    for digit in document.text(token).ok_or(What::Kind)? {
        if !digit.is_ascii_digit() {
            return Err(What::Bound);
        }
        value = value.checked_mul(10).ok_or(What::Bound)?;
        value = value.checked_add(u64::from(digit.checked_sub(b'0').expect("decimal digit"))).ok_or(What::Bound)?;
    }
    if value > max {
        return Err(What::Bound);
    }
    Ok(value)
}
pub fn boolean(document: &Document, at: u32) -> Result<bool, What> {
    match document.token(at).ok_or(What::Kind)?.kind {
        Kind::True => Ok(true),
        Kind::False => Ok(false),
        Kind::ObjectStart
        | Kind::ObjectEnd
        | Kind::ArrayStart
        | Kind::ArrayEnd
        | Kind::Key
        | Kind::String
        | Kind::Number
        | Kind::Null
        | Kind::Long => Err(What::Kind),
    }
}
pub fn duration(document: &Document, at: u32) -> Result<Duration, What> {
    let millis = integer(document, at, u64::MAX.checked_div(1_000_000).expect("nonzero unit"))?;
    Ok(Duration::from_millis(millis))
}
pub fn list(document: &Document, at: u32, max: u32) -> Result<List<u32>, What> {
    if document.token(at).ok_or(What::Kind)?.kind != Kind::ArrayStart {
        return Err(What::Kind);
    }
    let end = after(document, at)?.checked_sub(1).ok_or(What::Kind)?;
    let mut items = List::with_capacity(max);
    let mut next = at.checked_add(1).ok_or(What::Bound)?;
    for _ in at..end {
        if next == end {
            return Ok(items);
        }
        items.push(next).or(Err(What::Bound))?;
        next = after(document, next)?;
    }
    Err(What::Kind)
}
