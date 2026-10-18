//! Whether a root has someone to receive its escalations (domain/authority.md, 6; domain/people.md, 5.1).

use crate::{Domain, ProposalKind};

impl Domain {
    /// A party's role decides escalations itself, or its policy names an escalation role.
    /// Unknown projects and roles hold nothing (domain/people.md, 4 and 5.1).
    #[must_use]
    pub fn escalations_held(&self, project: u32, role: u32) -> bool {
        let Some(policy) = self.policy(project) else { return false };
        let Some(role) = self.role(project, role) else { return false };
        role.decides.allows(ProposalKind::Escalation) || policy.escalation_role.is_some()
    }
}
