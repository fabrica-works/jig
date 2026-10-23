//! Whole project owner facts; client/domain/contract.md, 5.1 and 5.3.
//! Lists retain their source order and allocated capacities. Neither a name,
//! a period nor permission mappings are inferred from these values.
use super::{
    Chip, DocumentText, Funder, HeldRole, Offered, OwnerAuthorityDocument, OwnerGrant, OwnerNumbers, OwnerPart,
    PartyKind, PolicyPermission, PolicyRequirement, PoolLedger, Recurring,
};
use skein_lib::List;

/// The current project, independently supplied from the earlier Project carrier.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnerProject {
    pub acts: List<Offered>,
    pub number: u32,
    pub name: DocumentText,
    pub resources: List<OwnerProjectResource>,
    pub policy: OwnerProjectPolicy,
    pub members: List<OwnerProjectMember>,
    pub current_period: Option<PoolLedger>,
    pub pools: List<PoolLedger>,
    pub configured_period: u64,
    pub configured_pools: List<PoolLedger>,
    pub configured_pools_cut: bool,
    pub spend: List<OwnerProjectSpend>,
    pub recurring: List<Recurring>,
    pub cut: bool,
}
/// One resource's whole connector part and independently supplied role.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnerProjectResource {
    pub part: OwnerPart,
    pub role: OwnerProjectResourceRole,
}
/// The four roles supplied by the resource owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OwnerProjectResourceRole {
    Owned,
    Participant,
    Unavailable,
    Context,
}
/// Policy facts, with a loss flag independent of each authority document.
/// Missing mappings remain absent; reserved lists and loss remain independent.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnerProjectPolicy {
    pub templates: List<super::TemplateOverride>,
    pub ceiling: OwnerAuthorityDocument,
    pub period_spend: u64,
    pub escalation_role: Option<u32>,
    pub projections: List<OwnerGrant>,
    pub roles: List<OwnerProjectPolicyRole>,
    pub requirements: List<PolicyRequirement>,
    pub permissions: Option<List<PolicyPermission>>,
    pub permissions_cut: bool,
    pub cut: bool,
}
/// A role's whole authority, allotment and full raw request/decision bits.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnerProjectPolicyRole {
    pub number: u32,
    pub authority: OwnerAuthorityDocument,
    pub period_spend: u64,
    pub requests: u16,
    pub decides: u8,
}
/// The same authentic membership facts for people and services.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnerProjectMember {
    pub party: u64,
    pub kind: PartyKind,
    pub role: HeldRole,
    pub login: DocumentText,
    pub name: DocumentText,
}
/// One expense row in the order its accounting owner supplied.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnerProjectSpend {
    pub chip: Chip,
    pub funder: Funder,
    pub numbers: OwnerNumbers,
}
