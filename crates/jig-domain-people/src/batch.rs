//! A party's batch (domain/tasks.md, sections 3 and 4).
use crate::{Authority, Grant, Parameter, PersonAddress, WakePolicy};
use alloc::boxed::Box;
/// One root requested by a party, mirroring the tasks child's New.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Member {
    pub executor: TaskExecutor,
    pub words: Box<[u8]>,
    pub parameters: Box<[Parameter]>,
    pub inputs: Box<[u64]>,
    pub contract: Contract,
    pub authority: Authority,
    pub budget: Budget,
    pub wake: WakePolicy,
    pub dependencies: Box<[Dependency]>,
    pub named: Box<[Grant]>,
    pub tracked: Option<u32>,
    pub label: u32,
    pub flags: Flags,
}
/// Who carries out a member.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TaskExecutor {
    Agent { charter: u32 },
    Procedure { connector: u16, code: u32 },
    Person(PersonAddress),
}
/// The member's result contract, mirroring tasks' contracts.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Contract {
    Report { words: u32 },
    Change { connector: u16, kind: u16, words: u32 },
    Verdict { choices: Box<[Verdict]> },
}
/// How much of the party's available budget to give a member.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Budget {
    Named,
    AsAllowed { cap: Option<u64> },
}
/// A dependency within the batch or the party's reach.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Dependency {
    Batch(u32),
    Existing(u64),
}
/// The resolved amount of a member whose budget was `AsAllowed`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Given {
    pub task: u64,
    pub amount: u64,
}
/// One allowed verdict.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Verdict {
    pub code: u32,
    pub words: u32,
    pub followups: u32,
    pub fields: Box<[FieldRule]>,
}
/// A required machine field.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FieldRule {
    pub name: Box<[u8]>,
    pub max: u32,
}
/// Task flags, mirroring the tasks child's flags.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Flags {
    pub conversation: bool,
    pub listed: bool,
    pub retention: Retention,
}
impl Flags {
    pub const PLAIN: Flags = Flags { conversation: false, listed: false, retention: Retention::Kept };
}
/// A task's retention class.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Retention {
    Kept,
    ForgottenOnceRead,
}
/// Checked owned payload, excluding the inline member.
#[must_use]
pub fn member_bytes(member: &Member) -> Option<u64> {
    let mut total = crate::authority_bytes(&member.authority)?.checked_add(u64::try_from(member.words.len()).ok()?)?;
    total = total.checked_add(
        u64::try_from(member.parameters.len()).ok()?.checked_mul(u64::try_from(size_of::<Parameter>()).ok()?)?,
    )?;
    total = total.checked_add(
        u64::try_from(member.dependencies.len()).ok()?.checked_mul(u64::try_from(size_of::<Dependency>()).ok()?)?,
    )?;
    total = total.checked_add(u64::try_from(member.inputs.len()).ok()?.checked_mul(8)?)?;
    let mut named = member.authority.clone();
    named.grants.clone_from(&member.named);
    named.delegation.kinds = Box::new([]);
    named.note_resources = Box::new([]);
    total = total.checked_add(crate::authority_bytes(&named)?)?;
    for parameter in &member.parameters {
        match parameter {
            Parameter::Bytes { value, .. } => total = total.checked_add(u64::try_from(value.len()).ok()?)?,
            Parameter::Number { .. } | Parameter::Resource { .. } => {}
        }
    }
    match &member.contract {
        Contract::Report { .. } | Contract::Change { .. } => {}
        Contract::Verdict { choices } => {
            total = total.checked_add(
                u64::try_from(choices.len()).ok()?.checked_mul(u64::try_from(size_of::<Verdict>()).ok()?)?,
            )?;
            for choice in choices {
                total = total.checked_add(
                    u64::try_from(choice.fields.len())
                        .ok()?
                        .checked_mul(u64::try_from(size_of::<FieldRule>()).ok()?)?,
                )?;
                for field in &choice.fields {
                    total = total.checked_add(u64::try_from(field.name.len()).ok()?)?;
                }
            }
        }
    }
    Some(total)
}
pub(crate) fn valid(limits: &crate::Limits, batch: &[Member]) -> bool {
    if batch.is_empty() || batch.len() > usize::try_from(limits.batch).expect("u32 fits usize") {
        return false;
    }
    for member in batch {
        if member.words.len() > usize::try_from(limits.words).expect("u32 fits usize") {
            return false;
        }
        match member_bytes(member) {
            Some(bytes) if bytes <= u64::from(limits.amendment_bytes) => {}
            Some(_) | None => return false,
        }
    }
    true
}
