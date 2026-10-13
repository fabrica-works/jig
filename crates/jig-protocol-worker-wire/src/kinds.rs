//! Project record sizes into skein's schema (protocol/channel.md, 4.1).
use crate::{MAGIC, VERSION, v2};
use skein_channel::{Direction, Kind as ChannelKind, Role, Schema, Term, Version};
use skein_lib::List;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Hello,
    Turn,
    Call,
    Answer,
    Fact,
    Rejected,
    Exhausted,
    Assignment,
    Message,
    Relayed,
    TurnKept,
    TurnBusy,
    Acknowledge,
    Cancel,
    Grant,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct KindRule {
    pub kind: u16,
    pub direction: Direction,
    pub body: Kind,
}

/// Every kind is required, in wire number order.
#[rustfmt::skip]
pub const KINDS: &[KindRule] = &[
    KindRule { kind: 0x0100, direction: Direction::FromInitiator, body: Kind::Hello },
    KindRule { kind: 0x0101, direction: Direction::FromInitiator, body: Kind::Turn },
    KindRule { kind: 0x0102, direction: Direction::FromInitiator, body: Kind::Call },
    KindRule { kind: 0x0103, direction: Direction::FromInitiator, body: Kind::Answer },
    KindRule { kind: 0x0104, direction: Direction::FromInitiator, body: Kind::Fact },
    KindRule { kind: 0x0105, direction: Direction::FromInitiator, body: Kind::Rejected },
    KindRule { kind: 0x0106, direction: Direction::FromInitiator, body: Kind::Exhausted },
    KindRule { kind: 0x0180, direction: Direction::FromResponder, body: Kind::Assignment },
    KindRule { kind: 0x0181, direction: Direction::FromResponder, body: Kind::Message },
    KindRule { kind: 0x0182, direction: Direction::FromResponder, body: Kind::Relayed },
    KindRule { kind: 0x0183, direction: Direction::FromResponder, body: Kind::TurnKept },
    KindRule { kind: 0x0184, direction: Direction::FromResponder, body: Kind::TurnBusy },
    KindRule { kind: 0x0185, direction: Direction::FromResponder, body: Kind::Acknowledge },
    KindRule { kind: 0x0186, direction: Direction::FromResponder, body: Kind::Cancel },
    KindRule { kind: 0x0187, direction: Direction::FromResponder, body: Kind::Grant },
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Overflow {
    Body,
    Frame,
}

pub fn schema(limits: &v2::Limits) -> Result<Schema, Overflow> {
    let mut kinds = List::with_capacity(u32::try_from(KINDS.len()).expect("kind count fits"));
    for rule in KINDS {
        let largest = match rule.body {
            Kind::Hello => v2::Hello::worst_case_bytes(limits),
            Kind::Turn => v2::Turn::worst_case_bytes(limits),
            Kind::Call => v2::Call::worst_case_bytes(limits),
            Kind::Answer => v2::Answer::worst_case_bytes(limits),
            Kind::Fact => v2::Fact::worst_case_bytes(limits),
            Kind::Rejected => v2::Rejected::worst_case_bytes(limits),
            Kind::Exhausted => v2::Exhausted::worst_case_bytes(limits),
            Kind::Assignment => v2::Assignment::worst_case_bytes(limits),
            Kind::Message => v2::Inbound::worst_case_bytes(limits),
            Kind::Relayed => v2::Relayed::worst_case_bytes(limits),
            Kind::TurnKept => v2::TurnKept::worst_case_bytes(limits),
            Kind::TurnBusy => v2::TurnBusy::worst_case_bytes(limits),
            Kind::Acknowledge => v2::Acknowledge::worst_case_bytes(limits),
            Kind::Cancel => v2::Cancel::worst_case_bytes(limits),
            Kind::Grant => v2::Granted::worst_case_bytes(limits),
        };
        let largest = largest.ok_or(Overflow::Body)?;
        let Ok(largest) = u32::try_from(largest) else { return Err(Overflow::Frame) };
        kinds.push(ChannelKind { kind: rule.kind, direction: rule.direction, largest }).expect("one slot per kind");
    }
    let mut versions = List::with_capacity(1);
    versions.push(Version { version: VERSION, kinds }).expect("one version has one slot");
    Ok(Schema { magic: MAGIC, versions })
}

/// The first sending kind whose peer bound does not hold this side's largest.
#[must_use]
pub fn terms_gap(schema: &Schema, role: Role, terms: &List<Term>) -> Option<u16> {
    let sending = match role {
        Role::Initiator => Direction::FromInitiator,
        Role::Responder => Direction::FromResponder,
    };
    let table = schema.version(VERSION)?;
    for rule in KINDS {
        if rule.direction == sending {
            let own = table.kind(rule.kind)?;
            let mut accepted = false;
            for term in terms {
                if term.kind == rule.kind && term.largest >= own.largest {
                    accepted = true;
                }
            }
            if !accepted {
                return Some(rule.kind);
            }
        }
    }
    None
}
