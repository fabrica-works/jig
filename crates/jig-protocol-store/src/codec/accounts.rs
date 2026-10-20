//! The accounts family's rows (protocol/store.md, 4.4): the installation's
//! identifier, and the accounts child's counter, accounts and sources (item 9).
use crate::accounts_v1 as wire;
use alloc::boxed::Box;
use jig_domain_accounts as domain;
use skein_lib::{List, Reader, Wall, Writer};

/// The installation's identifier, drawn at the first start (configuration.md, 8).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Installation {
    pub id: [u8; 16],
}

impl Installation {
    /// Encode this row with the accounts family's version.
    #[must_use]
    pub fn encode(&self) -> Box<[u8]> {
        let row = wire::Installation::new(&wire::CEILINGS, wire::InstallationParts { id: self.id })
            .expect("a fixed identifier has no bound to break");
        let mut writer = Writer::new(usize::try_from(row.measure()).expect("encoded row fits usize"));
        row.encode(&mut writer).expect("measured room");
        writer.finish()
    }

    /// Decode one row, refusing another version or trailing bytes.
    pub fn decode(bytes: &[u8]) -> Result<Self, wire::Problem> {
        let wire::InstallationParts { id } =
            wire::Installation::decode(&wire::CEILINGS, &mut Reader::new(bytes))?.into_parts();
        Ok(Self { id })
    }
}

/// Encode one of the accounts child's records (store.md 4.4 item 9) within the
/// schema's ceilings, which are the child's own bounds.
#[expect(clippy::manual_map, reason = "step code translates optional fields through exhaustive matches")]
pub fn encode(row: &domain::Stored) -> Result<Box<[u8]>, wire::Problem> {
    let limits = &wire::CEILINGS;
    let value = match row {
        domain::Stored::Issued { number, holder, name } => {
            let holder = match holder {
                domain::Holder::Deployment => wire::Holder::Deployment,
                domain::Holder::Party(party) => {
                    wire::Holder::Party(wire::Party::new(limits, wire::PartyParts { party: *party })?)
                }
            };
            wire::Value::Issued(wire::Issued::new(
                limits,
                wire::IssuedParts { number: *number, holder, name: name.clone() },
            )?)
        }
        domain::Stored::Next { number } => {
            wire::Value::Next(wire::Next::new(limits, wire::NextParts { number: *number })?)
        }
        domain::Stored::Account { number, provider, holder, place, uses, subject, display } => {
            let holder = match holder {
                domain::Holder::Deployment => wire::Holder::Deployment,
                domain::Holder::Party(party) => {
                    wire::Holder::Party(wire::Party::new(limits, wire::PartyParts { party: *party })?)
                }
            };
            wire::Value::Account(wire::Account::new(
                limits,
                wire::AccountParts {
                    number: *number,
                    provider: *provider,
                    holder,
                    place: *place,
                    uses: encode_uses(uses.as_deref())?,
                    subject: subject.clone(),
                    display: display.clone(),
                },
            )?)
        }
        domain::Stored::Source { account, number, place, kind, name, held } => {
            let kind = match kind {
                domain::Kind::SignIn => wire::Kind::SignIn,
                domain::Kind::Key => wire::Kind::Key,
                domain::Kind::Program => wire::Kind::Program,
                domain::Kind::None => wire::Kind::None,
            };
            let held = match held {
                None => None,
                Some(domain::Held::Limited { reset }) => {
                    let reset = match reset {
                        Some(reset) => Some(reset.as_nanos()),
                        None => None,
                    };
                    Some(wire::Held::Limited(wire::Limited::new(limits, wire::LimitedParts { reset })?))
                }
                Some(domain::Held::Refused) => Some(wire::Held::Refused),
            };
            wire::Value::Source(wire::Source::new(
                limits,
                wire::SourceParts { account: *account, number: *number, place: *place, kind, name: name.clone(), held },
            )?)
        }
    };
    let row = wire::Row::new(limits, wire::RowParts { value })?;
    let mut writer = Writer::new(usize::try_from(row.measure()).expect("encoded row fits usize"));
    row.encode(&mut writer).expect("measured room");
    Ok(writer.finish())
}

/// Decode one of the accounts child's records, refusing another version or
/// trailing bytes. The child checks the record's rules as it restores it.
#[expect(clippy::manual_map, reason = "step code translates optional fields through exhaustive matches")]
pub fn decode(bytes: &[u8]) -> Result<domain::Stored, wire::Problem> {
    let wire::RowParts { value } = wire::Row::decode(&wire::CEILINGS, &mut Reader::new(bytes))?.into_parts();
    Ok(match value {
        wire::Value::Issued(issued) => {
            let wire::IssuedParts { number, holder, name } = issued.into_parts();
            let holder = match holder {
                wire::Holder::Deployment => domain::Holder::Deployment,
                wire::Holder::Party(party) => domain::Holder::Party(party.into_parts().party),
            };
            domain::Stored::Issued { number, holder, name }
        }
        wire::Value::Next(next) => {
            let wire::NextParts { number } = next.into_parts();
            domain::Stored::Next { number }
        }
        wire::Value::Account(account) => {
            let wire::AccountParts { number, provider, holder, place, uses, subject, display } = account.into_parts();
            let holder = match holder {
                wire::Holder::Deployment => domain::Holder::Deployment,
                wire::Holder::Party(party) => domain::Holder::Party(party.into_parts().party),
            };
            domain::Stored::Account { number, provider, holder, place, uses: decode_uses(uses), subject, display }
        }
        wire::Value::Source(source) => {
            let wire::SourceParts { account, number, place, kind, name, held } = source.into_parts();
            let kind = match kind {
                wire::Kind::SignIn => domain::Kind::SignIn,
                wire::Kind::Key => domain::Kind::Key,
                wire::Kind::Program => domain::Kind::Program,
                wire::Kind::None => domain::Kind::None,
            };
            let held = match held {
                None => None,
                Some(wire::Held::Limited(limited)) => {
                    let reset = match limited.into_parts().reset {
                        Some(reset) => Some(Wall::from_nanos(reset)),
                        None => None,
                    };
                    Some(domain::Held::Limited { reset })
                }
                Some(wire::Held::Refused) => Some(domain::Held::Refused),
            };
            domain::Stored::Source { account, number, place, kind, name, held }
        }
    })
}

fn encode_uses(uses: Option<&[domain::Use]>) -> Result<Option<List<wire::Use>>, wire::Problem> {
    let Some(uses) = uses else { return Ok(None) };
    let mut result = List::with_capacity(domain::USES);
    for payer in uses {
        let payer = match payer {
            domain::Use::Party(party) => {
                wire::Use::Party(wire::Party::new(&wire::CEILINGS, wire::PartyParts { party: *party })?)
            }
            domain::Use::Project(project) => {
                wire::Use::Project(wire::Project::new(&wire::CEILINGS, wire::ProjectParts { project: *project })?)
            }
        };
        if result.push(payer).is_err() {
            return Err(wire::Problem { path: wire::Path::AccountUses, reason: skein_codec::Reason::Bound });
        }
    }
    Ok(Some(result))
}
fn decode_uses(uses: Option<List<wire::Use>>) -> Option<Box<[domain::Use]>> {
    let uses = uses?;
    let mut result = List::with_capacity(domain::USES);
    for payer in uses.into_boxed() {
        let payer = match payer {
            wire::Use::Party(party) => domain::Use::Party(party.into_parts().party),
            wire::Use::Project(project) => domain::Use::Project(project.into_parts().project),
        };
        result.push(payer).expect("schema bounds uses");
    }
    Some(result.into_boxed())
}
