//! The core's live settings (tag 8) and revision-ordered history (tag 9).
use jig_domain_core::settings::{Key, Setting};
use jig_domain_people::Proposer;
use skein_kv::{KeyReader, KeyWriter};
pub(super) fn write(writer: &mut KeyWriter, key: &Key) {
    match key {
        Key::History(revision) => {
            writer.tag(9);
            writer.u64(u64::from(*revision));
        }
        Key::Default(setting) => {
            writer.tag(8);
            writer.tag(0);
            setting_write(writer, *setting);
        }
        Key::Changed(setting) => {
            writer.tag(8);
            writer.tag(1);
            setting_write(writer, *setting);
        }
        Key::Held(revision) => {
            writer.tag(8);
            writer.tag(2);
            writer.u64(u64::from(*revision));
        }
        Key::Charter { charter, revision } => {
            writer.tag(8);
            writer.tag(3);
            writer.u64(u64::from(*charter));
            writer.u64(u64::from(*revision));
        }
        Key::Proposal { proposer, proposal } => {
            writer.tag(8);
            writer.tag(4);
            match proposer {
                Proposer::Task(number) => {
                    writer.tag(0);
                    writer.u64(*number);
                }
                Proposer::Party(number) => {
                    writer.tag(1);
                    writer.u64(*number);
                }
            }
            writer.u64(u64::from(*proposal));
        }
        Key::Extent => {
            writer.tag(8);
            writer.tag(5);
        }
    }
}
fn setting_write(writer: &mut KeyWriter, setting: Setting) {
    let (kind, number) = match setting {
        Setting::Charter(n) => (0, n),
        Setting::Template(n) => (1, n),
        Setting::Policy(n) => (2, n),
        Setting::Rules => (3, 0),
    };
    writer.tag(kind);
    writer.u64(0);
    writer.u64(u64::from(number));
}
pub(super) fn read(reader: &mut KeyReader<'_>, history: bool) -> Option<Key> {
    if history {
        return Some(Key::History(u32::try_from(reader.u64()?).ok()?));
    }
    Some(match reader.tag()? {
        0 => Key::Default(setting_read(reader)?),
        1 => Key::Changed(setting_read(reader)?),
        2 => Key::Held(u32::try_from(reader.u64()?).ok()?),
        3 => Key::Charter { charter: u32::try_from(reader.u64()?).ok()?, revision: u32::try_from(reader.u64()?).ok()? },
        4 => {
            let proposer = match reader.tag()? {
                0 => Proposer::Task(reader.u64()?),
                1 => Proposer::Party(reader.u64()?),
                _ => return None,
            };
            Key::Proposal { proposer, proposal: u32::try_from(reader.u64()?).ok()? }
        }
        5 => Key::Extent,
        _ => return None,
    })
}
fn setting_read(reader: &mut KeyReader<'_>) -> Option<Setting> {
    let kind = reader.tag()?;
    if reader.u64()? != 0 {
        return None;
    }
    let number = u32::try_from(reader.u64()?).ok()?;
    Some(match kind {
        0 => Setting::Charter(number),
        1 => Setting::Template(number),
        2 => Setting::Policy(number),
        3 if number == 0 => Setting::Rules,
        _ => return None,
    })
}
