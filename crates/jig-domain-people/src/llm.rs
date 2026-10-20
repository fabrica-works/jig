//! A task's LLM choice as a party names it, mirroring the tasks child's (domain/tasks.md, 3):
//! models and efforts by index in configuration's order. The root checks the choice when it
//! makes or amends the task; this child only keeps it with the request.
use alloc::boxed::Box;
use core::mem::size_of;

/// The most sub-agent models one choice names, the tasks child's `MODELS`.
pub const MODELS: u32 = 64;

/// One configured model and one of its efforts.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ModelEffort {
    /// The configured model, in configuration's order.
    pub model: u8,
    /// One of the efforts the model declares, in its order; `None` is its LLM provider's default.
    pub effort: Option<u8>,
}

/// What an agent task's runs' LLMs are: its main model and those its sub-agents may use.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct LlmChoice {
    /// Creator preference, separate from the task's last successfully claimed account.
    pub account: Option<u32>,
    pub main: ModelEffort,
    /// At most `MODELS`.
    pub subagents: Box<[ModelEffort]>,
}

/// Why the root refused a named model, the tasks child's `LlmFault`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum LlmWhy {
    /// The model is not configured.
    Unconfigured,
    /// The effort is not one the model declares, or none is named for a model that declares some.
    Effort,
    /// The model is not one of the task's charter's.
    Charter,
    /// The task's authority does not allow the model.
    Authority,
    /// The task's executor runs no model.
    Procedure,
    /// The sub-agents name the model twice.
    Repeated,
}

/// Checked deep-byte size of an optional choice: its sub-agents.
#[must_use]
pub fn llm_bytes(choice: Option<&LlmChoice>) -> Option<u64> {
    let Some(choice) = choice else {
        return Some(0);
    };
    u64::try_from(choice.subagents.len()).ok()?.checked_mul(u64::try_from(size_of::<ModelEffort>()).ok()?)
}

/// Whether an optional choice has the shape a request keeps.
pub(crate) fn valid(choice: Option<&LlmChoice>) -> bool {
    let Some(choice) = choice else {
        return true;
    };
    choice.subagents.len() <= usize::try_from(MODELS).expect("u32 fits usize")
}

/// Why an explicit account cannot fund the prospective task (domain/accounts.md, 4.3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AccountWhy {
    Unknown,
    Provider,
    Uses,
    Limited,
}
