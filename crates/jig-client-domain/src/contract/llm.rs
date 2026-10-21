//! A task's LLM choice and the models a party may choose, named as configuration names them
//! (client/domain/contract.md, 3.2 and 5.1; domain/tasks.md, 3). The engine maps the names.
use super::Text;
use alloc::boxed::Box;
use skein_lib::List;

/// One model and its effort, each by its configured name.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ModelEffort {
    pub model: Text,
    /// One of the efforts the model declares; `None` is its LLM provider's default.
    pub effort: Option<Text>,
}

/// What an agent task's runs' LLMs are: its main model and those its sub-agents may use.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LlmChoice {
    /// Creator preference, separate from the task's last successfully claimed account.
    pub account: Option<u32>,
    pub main: ModelEffort,
    pub subagents: List<ModelEffort>,
}

/// A model the party may choose in a project, with what configuration gives it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MeModel {
    pub name: Text,
    /// The LLM provider that serves it, when configuration names one.
    pub provider: Option<u32>,
    /// Its context window in tokens.
    pub window: u32,
    /// Its longest completion in tokens.
    pub output: u32,
    /// The efforts it declares, in its order.
    pub efforts: List<Text>,
    /// Its configured prices.
    pub prices: ModelPrices,
    /// What it is good at, as configured, if given.
    pub description: Option<Text>,
}

/// Amounts per `unit` tokens in the deployment's spend unit. Without a separate cache-write
/// price, cache writes are priced at input (client/domain/contract.md, 5.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ModelPrices {
    pub input: u64,
    pub cached: u64,
    pub cache_write: Option<u64>,
    pub output: u64,
    pub unit: u32,
}

/// Why a named model was refused (domain/tasks.md, 4 and 7).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

/// Why an explicit account cannot fund the prospective task (domain/accounts.md, 4.3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AccountWhy {
    Unknown,
    Provider,
    Uses,
    Limited,
}

/// The task's current next-run wait. Deadlines are wall times; None is unknown, never zero.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TaskRunWait {
    Account { account: Option<u32>, source: Option<u32>, why: TaskWaitWhy },
    Credential,
    Host,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TaskWaitWhy {
    Source(TaskSourceState),
    Refreshing,
    Host,
    Empty,
    Invalid(AccountWhy),
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TaskSourceState {
    Asking,
    Granted,
    Later { until: Option<skein_lib::Wall> },
    Limited { until: Option<skein_lib::Wall> },
    Refused,
    NeedsSignIn,
    Misconfigured,
    Unloaded { at: super::OwnerAccountPlace, why: super::OwnerAccountUnread },
    Cancelled,
    Closing,
}

/// Configured public source offers; no private account identity is carried.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[expect(clippy::struct_excessive_bools, reason = "independent configured source offers and cut facts")]
pub struct ProviderOffer {
    /// Stable configured LLM provider number.
    pub provider: u32,
    /// Actual startup key ceiling for this provider's key offer.
    pub key_bytes: u32,
    /// Configured display name, carrying its own text loss.
    pub name: super::DocumentText,
    /// Whether a sign-in source is offered.
    pub sign_in: bool,
    /// Whether a key source is offered.
    pub key: bool,
    /// Whole configured program names within the list bound.
    pub programs: List<Text>,
    /// Whether additional program names were omitted.
    pub programs_cut: bool,
    /// Whether configuration offers a source needing no credential.
    pub none: bool,
    /// Whether configured sign-ins use plan terms.
    pub signed_in_plan: bool,
    /// Whole optional public HTTPS manage target, at most 2,048 bytes.
    pub manage_usage: Option<Box<[u8]>>,
}
/// Availability does not reserve a run's account, host, slots or budget.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Readiness {
    Usable,
    Unavailable,
    Unknown,
}
/// A provider's readiness for one authorized payer, independent of private account rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PayerReadiness {
    /// Stable configured LLM provider number.
    pub provider: u32,
    /// The authenticated party or a project it may read.
    pub payer: super::OwnerAccountUse,
    /// Current observation from the engine selection and host checks.
    pub readiness: Readiness,
    /// Unknown until the engine has program check facts; pending sign-ins are false.
    pub settled: Option<bool>,
}
