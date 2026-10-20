//! A request's canonical original name context and tagged selectors (store.md, 5.1).
use crate::{Action, Amendment, Ask, Authority, Member, Operation, PolicyChange};
use alloc::boxed::Box;

use crate::MODELS;
pub const MODEL_NAME_BYTES: u32 = 4_096;

/// A literal ALL or a frozen mask into its enclosing record's model-name context.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Models {
    All,
    Named(u64),
}
impl Models {
    pub const ALL: Self = Self::All;
    pub const NONE: Self = Self::Named(0);
    #[must_use]
    pub const fn bits(self) -> u64 {
        match self {
            Self::All => u64::MAX,
            Self::Named(bits) => bits,
        }
    }
    #[must_use]
    pub const fn is_all(self) -> bool {
        match self {
            Self::All => true,
            Self::Named(_) => false,
        }
    }
    #[must_use]
    pub const fn from_parts(bits: u64, all: bool) -> Self {
        if all { Self::All } else { Self::Named(bits) }
    }
    /// Reordering and removal affect effective authority, never a retained ask's identity.
    #[must_use]
    pub fn remap(self, moved: &[u64; 64]) -> Self {
        match self {
            Self::All => Self::All,
            Self::Named(bits) => {
                let mut result = 0;
                for (bit, today) in moved.iter().enumerate() {
                    if bits.checked_shr(u32::try_from(bit).expect("bounded bit")).unwrap_or(0) & 1 == 1 {
                        result |= today;
                    }
                }
                Self::Named(result)
            }
        }
    }
}

/// One context per retained request: 64 names plus their owned slice headers.
#[must_use]
pub fn model_names_worst_case() -> Option<u64> {
    u64::from(MODELS).checked_mul(u64::from(MODEL_NAME_BYTES).checked_add(u64::try_from(size_of::<Box<[u8]>>()).ok()?)?)
}

#[must_use]
/// Protocols validate UTF-8; the domain checks count, lengths and canonical byte order.
pub fn valid_model_names(names: &[Box<[u8]>]) -> bool {
    if names.len() > usize::try_from(MODELS).expect("model bound") {
        return false;
    }
    for (at, name) in names.iter().enumerate() {
        if name.is_empty() || name.len() > usize::try_from(MODEL_NAME_BYTES).expect("name bound") {
            return false;
        }
        for earlier in names.iter().take(at) {
            if earlier >= name {
                return false;
            }
        }
    }
    true
}

fn authority(value: &mut Authority, moved: &[u64; 64]) {
    if let Some(models) = value.models {
        value.models = Some(models.remap(moved));
    }
}
fn amendment(value: &mut Amendment, moved: &[u64; 64]) {
    if let Some(value) = &mut value.authority {
        authority(value, moved);
    }
}
fn members(values: &mut [Member], moved: &[u64; 64]) {
    for value in values {
        authority(&mut value.authority, moved);
    }
}

/// Exhaustive traversal of every authority-bearing request, shared by canonicalization and
/// fresh-request admission. Stored asks keep their original context and are never remapped.
pub fn remap_models(ask: &mut Ask, moved: &[u64; 64]) {
    match ask {
        Ask::Create { batch, .. } => members(batch, moved),
        Ask::Amend { amendment: value, .. } => amendment(value, moved),
        Ask::Propose { action, .. } => match action {
            Action::Batch(batch) => members(batch, moved),
            Action::Amend { amendment: value, .. } => amendment(value, moved),
        },
        Ask::Compound { operations, .. } => {
            for at in 0..operations.len() {
                match operations.get_mut(at).expect("bounded operation") {
                    Operation::Amend { amendment: value, .. } => amendment(value, moved),
                    Operation::Cancel { .. }
                    | Operation::Move { .. }
                    | Operation::Release { .. }
                    | Operation::Message { .. }
                    | Operation::Stop { .. } => {}
                }
            }
        }
        Ask::ChangePolicy { change, .. } => match change {
            PolicyChange::Role(role) => authority(&mut role.authority, moved),
            PolicyChange::ProjectSpend { .. }
            | PolicyChange::Requirements { .. }
            | PolicyChange::Projections { .. }
            | PolicyChange::Permissions { .. }
            | PolicyChange::Templates { .. } => {}
        },
        Ask::MarkSeen { .. }
        | Ask::Application { .. }
        | Ask::EditNote { .. }
        | Ask::MakeService { .. }
        | Ask::Adopt { .. }
        | Ask::Withdraw { .. }
        | Ask::Stop { .. }
        | Ask::Cancel { .. }
        | Ask::Release { .. }
        | Ask::TakePerson { .. }
        | Ask::HandBackPerson { .. }
        | Ask::AnswerPerson { .. }
        | Ask::Move { .. }
        | Ask::DecideProposal { .. }
        | Ask::Say { .. }
        | Ask::AnswerQuestion { .. }
        | Ask::SetRoles { .. }
        | Ask::SetPool { .. }
        | Ask::DecideEscalation { .. }
        | Ask::AccountSignIn { .. }
        | Ask::AccountSignOut { .. }
        | Ask::AccountChange { .. } => {}
    }
}

fn authority_bits(value: &Authority) -> u64 {
    match value.models {
        Some(Models::Named(bits)) => bits,
        Some(Models::All) | None => 0,
    }
}
fn amendment_bits(value: &Amendment) -> u64 {
    match &value.authority {
        Some(value) => authority_bits(value),
        None => 0,
    }
}
fn members_bits(values: &[Member]) -> u64 {
    let mut bits = 0;
    for value in values {
        bits |= authority_bits(&value.authority);
    }
    bits
}
/// The original context contains exactly the names referenced by frozen selectors.
#[must_use]
pub fn valid_model_context(ask: &Ask, names: &[Box<[u8]>]) -> bool {
    if !valid_model_names(names) {
        return false;
    }
    let bits = match ask {
        Ask::Create { batch, .. } => members_bits(batch),
        Ask::Amend { amendment, .. } => amendment_bits(amendment),
        Ask::Propose { action, .. } => match action {
            Action::Batch(batch) => members_bits(batch),
            Action::Amend { amendment, .. } => amendment_bits(amendment),
        },
        Ask::Compound { operations, .. } => {
            let mut bits = 0;
            for operation in operations {
                match operation {
                    Operation::Amend { amendment, .. } => bits |= amendment_bits(amendment),
                    Operation::Cancel { .. }
                    | Operation::Move { .. }
                    | Operation::Release { .. }
                    | Operation::Message { .. }
                    | Operation::Stop { .. } => {}
                }
            }
            bits
        }
        Ask::ChangePolicy { change, .. } => match change {
            PolicyChange::Role(role) => authority_bits(&role.authority),
            PolicyChange::ProjectSpend { .. }
            | PolicyChange::Requirements { .. }
            | PolicyChange::Projections { .. }
            | PolicyChange::Permissions { .. }
            | PolicyChange::Templates { .. } => 0,
        },
        Ask::MarkSeen { .. }
        | Ask::Application { .. }
        | Ask::EditNote { .. }
        | Ask::MakeService { .. }
        | Ask::Adopt { .. }
        | Ask::Withdraw { .. }
        | Ask::Stop { .. }
        | Ask::Cancel { .. }
        | Ask::Release { .. }
        | Ask::TakePerson { .. }
        | Ask::HandBackPerson { .. }
        | Ask::AnswerPerson { .. }
        | Ask::Move { .. }
        | Ask::DecideProposal { .. }
        | Ask::Say { .. }
        | Ask::AnswerQuestion { .. }
        | Ask::SetRoles { .. }
        | Ask::SetPool { .. }
        | Ask::DecideEscalation { .. }
        | Ask::AccountSignIn { .. }
        | Ask::AccountSignOut { .. }
        | Ask::AccountChange { .. } => 0,
    };
    let mask = u64::MAX
        .checked_shr(u32::try_from(64_usize.checked_sub(names.len()).expect("bounded context")).expect("bounded names"))
        .unwrap_or(0);
    bits == mask
}
