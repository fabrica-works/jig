//! Shared keys favor number boundaries and common prefixes (protocol/store.md, 4.1).
use crate::keys::Own;
use jig_domain_core::{self as core, CoreKey, ProjectionKey};
use jig_domain_people::{self as people, Proposer};
use skein_lib::Rng;

pub const KEY_LIMIT: u32 = 1024;
pub fn number(rng: &mut Rng) -> u64 {
    match rng.below(5) {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => u64::from(u32::MAX),
        _ => u64::MAX,
    }
}
pub fn narrow(rng: &mut Rng) -> u32 {
    match rng.below(4) {
        0 => 0,
        1 => 1,
        2 => 2,
        _ => u32::MAX,
    }
}
pub fn own(rng: &mut Rng) -> Own {
    match rng.below(3) {
        0 => Own::Format,
        1 => Own::Transcript { task: number(rng) },
        _ => Own::Attempt { task: number(rng), attempt: number(rng) },
    }
}
pub fn core_key(rng: &mut Rng) -> CoreKey {
    match rng.below(8) {
        0 => CoreKey::Deployment,
        1 => CoreKey::Projection(match rng.below(4) {
            0 => ProjectionKey::Goal(number(rng)),
            1 => ProjectionKey::Task { goal: number(rng), task: number(rng) },
            2 => ProjectionKey::Lifecycle { goal: number(rng), task: number(rng), position: number(rng) },
            _ => ProjectionKey::Revision { goal: number(rng), task: number(rng), revision: number(rng) },
        }),
        2 => CoreKey::RunProof(number(rng)),
        3 => CoreKey::Call(core::CallKey {
            task: number(rng),
            attempt: number(rng),
            completion: narrow(rng),
            position: narrow(rng),
        }),
        4 => CoreKey::Turn { task: number(rng), attempt: number(rng), turn: narrow(rng) },
        5 => CoreKey::Terminal { task: number(rng), attempt: number(rng) },
        6 => CoreKey::ProposalDecision {
            proposer: match rng.below(2) {
                0 => Proposer::Task(number(rng)),
                _ => Proposer::Party(number(rng)),
            },
            proposal: number(rng),
        },
        _ => CoreKey::EscalationDecision { task: number(rng), revision: number(rng) },
    }
}
pub fn people_key(rng: &mut Rng) -> people::Key {
    match rng.below(7) {
        0 => people::Key::Person(number(rng)),
        1 => people::Key::ReadResults { party: number(rng) },
        2 => people::Key::ReadChat { party: number(rng), task: number(rng) },
        3 => people::Key::SignIn(number(rng)),
        4 => people::Key::Roles(narrow(rng)),
        5 => people::Key::Policy(narrow(rng)),
        _ => {
            let mut key = [0; 16];
            for byte in &mut key {
                *byte = match rng.below(5) {
                    0 => 0,
                    1 => 1,
                    2 => 2,
                    3 => 254,
                    _ => 255,
                };
            }
            people::Key::Answer(people::RequestKey { person: number(rng), key })
        }
    }
}
pub fn key(rng: &mut Rng) -> core::Key {
    match rng.below(2) {
        0 => core::Key::Core(core_key(rng)),
        _ => core::Key::People(people_key(rng)),
    }
}
