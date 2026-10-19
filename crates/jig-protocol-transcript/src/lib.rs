//! Whole Smith turn bytes become the client's own bounded vocabulary. No state,
//! endpoint policy or opaque provider bytes survive this translation. `decode`
//! refuses unsupported or broken records; a receiving limit cuts the displayed
//! turn, marked by `Turn::cut`. See client/domain/client.md, 10, and contract.md, 10.
//!
//! Calls are numbered by their first call/result block's position within the turn.
//! Their inputs remain bytes as written. Results also fill the matching call's
//! outcome, including results outside the display window. This schema has no jig
//! object references or inbox read position: neither is inferred from tool text.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::boxed::Box;
use jig_client_domain::contract::{Block, Call, Message, Speaker, Text, Turn, TurnLimits, Usage};
use skein_lib::{Decimal, List, Reader};
use smith_transcript::v6 as smith;

mod summary;
#[cfg(test)]
mod tests;

/// Admission bounds for Smith bytes and receiving bounds for the displayed turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub smith: smith::Limits,
    pub bytes: u32,
    pub turn: TurnLimits,
}

impl Limits {
    #[must_use]
    pub const fn new(bytes: u32, turn: TurnLimits) -> Self {
        Self { smith: smith::CEILINGS, bytes, turn }
    }
}

/// Why one activity turn cannot be shown; its activity line remains in place.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Undecodable {
    /// The record has an unsupported version.
    Version { found: u16 },
    /// Broken fields or trailing bytes.
    Malformed,
    /// The wire turn exceeds admission bounds.
    TooLarge,
    /// A wire enumeration names an unknown variant.
    Unknown,
}

/// Decodes a whole record; receiving bounds truncate without losing usage/spend.
pub fn decode(limits: &Limits, bytes: &[u8]) -> Result<Turn, Undecodable> {
    if bytes.len() > usize::try_from(limits.bytes).expect("u32 fits usize") {
        return Err(Undecodable::TooLarge);
    }
    let version = Reader::new(bytes).u16().ok_or(Undecodable::Malformed)?;
    if u64::from(version) != smith::Turn::FORMAT_VERSION {
        return Err(Undecodable::Version { found: version });
    }
    let source = match smith::Turn::decode(&limits.smith, &mut Reader::new(bytes)) {
        Ok(turn) => turn,
        Err(problem) => return Err(classify(problem)),
    };
    summary::validate(&limits.smith, &source)?;
    Ok(translate(limits.turn, &source))
}

// Smith's codec reason type is intentionally not a client dependency. Obtain
// allocation-free reference reasons through Smith's public limits check/decoder.
fn classify(problem: smith::Problem) -> Undecodable {
    let mut beyond = smith::CEILINGS;
    beyond.turn_endpoint = beyond.turn_endpoint.checked_add(1).expect("small ceiling");
    let bound = smith::check_limits(&beyond).expect_err("limits exceed schema");
    let tag = smith::Role::decode(&smith::CEILINGS, &mut Reader::new(&[255])).expect_err("unknown role tag");
    if problem.reason == bound.reason {
        Undecodable::TooLarge
    } else if problem.reason == tag.reason {
        Undecodable::Unknown
    } else {
        Undecodable::Malformed
    }
}

fn translate(limits: TurnLimits, source: &smith::Turn) -> Turn {
    let mut budget = Budget { left: limits.text, cut: false };
    let mut messages = List::with_capacity(limits.messages.min(source.messages().len()));
    for message in source.messages() {
        if messages.len() == messages.capacity() {
            budget.cut = true;
            break;
        }
        let speaker = match message.role() {
            smith::Role::User => Speaker::Person,
            smith::Role::Assistant => Speaker::Assistant,
        };
        let mut blocks = List::with_capacity(limits.blocks.min(message.blocks().len()));
        for block in message.blocks() {
            if blocks.len() == blocks.capacity() {
                budget.cut = true;
                break;
            }
            let block = translate_block(source, block, &mut budget);
            blocks.push(block).expect("receiving block capacity checked");
        }
        messages.push(Message { speaker, blocks }).expect("receiving message capacity checked");
    }
    let usage = source.usage();
    Turn {
        messages,
        // The contract's counts are plain: one the provider did not report shows
        // as zero, as smith reported it before its counts became optional.
        used: Usage {
            input: usage.input().unwrap_or(0),
            output: usage.output().unwrap_or(0),
            cached: usage.cache_read().unwrap_or(0),
            cache_written: usage.cache_write().unwrap_or(0),
        },
        spent: source.spent(),
        read_through: None,
        cut: budget.cut,
    }
}

fn translate_block(source: &smith::Turn, block: &smith::Block, budget: &mut Budget) -> Block {
    match block {
        smith::Block::Text(text) | smith::Block::Refusal(text) => Block::Text(budget.text(text.text())),
        smith::Block::Opaque(opaque) => Block::Opaque {
            dialect: budget.text(Decimal::of(u64::from(opaque.dialect())).as_bytes()),
            size: u64::try_from(opaque.bytes().len()).expect("schema byte ceiling fits u64"),
        },
        smith::Block::Call(call) => {
            let tool = budget.text(call.name());
            let input = budget.text(call.input());
            let mut outcome = None;
            let mut failed = false;
            for message in source.messages() {
                for candidate in message.blocks() {
                    match candidate {
                        smith::Block::Result(result) => {
                            if result.id() == call.id() && outcome.is_none() {
                                let answer = result_text(result.returned(), budget);
                                failed = answer.failed;
                                outcome = Some(answer.text);
                            }
                        }
                        smith::Block::Text(_)
                        | smith::Block::Refusal(_)
                        | smith::Block::Call(_)
                        | smith::Block::Opaque(_) => {}
                    }
                }
            }
            Block::Call(Call {
                number: call_number(source, call.id()),
                tool,
                input,
                outcome,
                failed,
                objects: List::with_capacity(0),
                parts: List::with_capacity(0),
            })
        }
        smith::Block::Result(result) => {
            let answer = result_text(result.returned(), budget);
            Block::Result { call: call_number(source, result.id()), summary: answer.text, failed: answer.failed }
        }
    }
}

fn call_number(source: &smith::Turn, id: &[u8]) -> u64 {
    let mut position = 0_u64;
    for message in source.messages() {
        for block in message.blocks() {
            position = position.checked_add(1).expect("schema ceilings bound blocks");
            let candidate = match block {
                smith::Block::Call(call) => Some(call.id()),
                smith::Block::Result(result) => Some(result.id()),
                smith::Block::Text(_) | smith::Block::Refusal(_) | smith::Block::Opaque(_) => None,
            };
            if candidate == Some(id) {
                return position;
            }
        }
    }
    unreachable!("the translated call or result occurs in its source turn")
}

#[derive(Debug)]
struct Budget {
    left: u32,
    cut: bool,
}
impl Budget {
    fn text(&mut self, bytes: &[u8]) -> Text {
        let maximum = usize::try_from(self.left).expect("u32 fits usize");
        let length = bytes.len().min(maximum);
        self.cut |= length < bytes.len();
        self.left =
            self.left.checked_sub(u32::try_from(length).expect("bounded length")).expect("length within budget");
        Text::new(
            Box::from(bytes.get(..length).expect("prefix within bytes")),
            u32::try_from(length).expect("bounded length"),
        )
        .expect("text allocated at receiving length")
    }
}

#[derive(Debug)]
struct ResultText {
    text: Text,
    failed: bool,
}
fn result_text(returned: &smith::Returned, budget: &mut Budget) -> ResultText {
    match returned {
        smith::Returned::Said(said) => ResultText { text: budget.text(said.text()), failed: said.error() },
        smith::Returned::Outcome(_)
        | smith::Returned::Invalid(_)
        | smith::Returned::NotRun(_)
        | smith::Returned::Withdrawn => {
            let summary = summary::summarize(returned);
            ResultText { text: budget.text(&summary.bytes), failed: summary.failed }
        }
    }
}
