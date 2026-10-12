//! Scope patterns preserve each escaped segment (protocol/store.md, 4.1).
use alloc::boxed::Box;
use jig_domain_notes::{Key, Last, Pattern, Scope};
use skein_kv::{KeyReader, KeyWriter};
use skein_lib::List;
pub(super) fn write(writer: &mut KeyWriter, key: &Key) {
    match key {
        Key::Entry { name } => {
            writer.tag(0);
            writer.u64(*name);
        }
        Key::Line { scope: value, name } => {
            writer.tag(1);
            scope(writer, value);
            writer.u64(*name);
        }
    }
}
pub(super) fn scope(writer: &mut KeyWriter, scope: &Scope) {
    match scope {
        Scope::Deployment => {
            writer.tag(0);
        }
        Scope::Project { project } => {
            writer.tag(1);
            writer.u64(u64::from(*project));
        }
        Scope::Goal { project, goal } => {
            writer.tag(2);
            writer.u64(u64::from(*project));
            writer.u64(*goal);
        }
        Scope::Resources { project, connector, pattern } => {
            writer.tag(3);
            writer.u64(u64::from(*project));
            writer.u64(u64::from(*connector));
            for segment in &pattern.segments {
                writer.tag(1);
                writer.bytes(segment);
            }
            writer.tag(0);
            match &pattern.last {
                Last::Exact(bytes) => {
                    writer.tag(0);
                    writer.bytes(bytes);
                }
                Last::Open(bytes) => {
                    writer.tag(1);
                    writer.bytes(bytes);
                }
            }
        }
    }
}
pub(super) fn read(reader: &mut KeyReader<'_>, bytes: usize) -> Option<Key> {
    Some(match reader.tag()? {
        0 => Key::Entry { name: reader.u64()? },
        1 => Key::Line { scope: read_scope(reader, bytes)?, name: reader.u64()? },
        _ => return None,
    })
}
fn read_scope(reader: &mut KeyReader<'_>, bytes: usize) -> Option<Scope> {
    Some(match reader.tag()? {
        0 => Scope::Deployment,
        1 => Scope::Project { project: u32::try_from(reader.u64()?).ok()? },
        2 => Scope::Goal { project: u32::try_from(reader.u64()?).ok()?, goal: reader.u64()? },
        3 => Scope::Resources {
            project: u32::try_from(reader.u64()?).ok()?,
            connector: u16::try_from(reader.u64()?).ok()?,
            pattern: read_pattern(reader, bytes)?,
        },
        _ => return None,
    })
}
fn read_pattern(reader: &mut KeyReader<'_>, bytes: usize) -> Option<Pattern> {
    // Every segment consumes a marker and a terminator; input length bounds the walk and storage.
    let capacity = u32::try_from(bytes).ok()?;
    let mut segments: List<Box<[u8]>> = List::with_capacity(capacity);
    for _ in 0..=capacity {
        match reader.tag()? {
            0 => {
                let last = match reader.tag()? {
                    0 => Last::Exact(reader.bytes()?),
                    1 => Last::Open(reader.bytes()?),
                    _ => return None,
                };
                return Some(Pattern { segments: segments.into_boxed(), last });
            }
            1 => {
                let segment = reader.bytes()?;
                segments.push(segment).ok()?;
            }
            _ => return None,
        }
    }
    None
}
