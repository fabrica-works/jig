//! Exact policy edit review fragments; client/domain/view.md, 7.
//! Numeric flags and literal segments remain supplied facts, not reconstructed policy.
use crate::{
    Builder, Role, Run,
    phrases::{Hole, jig::JigPhrase},
    words::Words,
};
use jig_client::contract::{OwnerPattern, OwnerPatternEnd, PolicyEdit, PolicyGuard};

/// Allocation required for one complete review fragment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PolicyFragment {
    pub nodes: u32,
    pub text_bytes: u32,
    pub text_per_node: u32,
    pub depth: u32,
}
impl PolicyFragment {
    fn leaf(&mut self, bytes: u32) -> Option<()> {
        self.nodes = self.nodes.checked_add(1)?;
        self.text_bytes = self.text_bytes.checked_add(bytes)?;
        self.text_per_node = self.text_per_node.max(bytes);
        Some(())
    }
    fn phrase(&mut self, words: &Words, phrase: JigPhrase) -> Option<()> {
        self.leaf(words.book.capacity(phrase.id())?)
    }
    fn pattern(&mut self, pattern: &OwnerPattern, words: &Words) -> Option<()> {
        for segment in &pattern.segments {
            self.leaf(crate::literal_segment::display_bytes(segment)?)?;
        }
        let (phrase, bytes) = match &pattern.last {
            OwnerPatternEnd::Exact(bytes) => (crate::owner_authority_words::AuthorityPhrase::Exact, bytes),
            OwnerPatternEnd::Open(bytes) => (crate::owner_authority_words::AuthorityPhrase::Open, bytes),
        };
        self.leaf(words.book.capacity(phrase.id())?)?;
        self.leaf(crate::literal_segment::display_bytes(bytes)?)
    }
}
/// Counts all checked labels, whole authority and escaped literal segment leaves.
#[must_use]
pub fn required(value: &PolicyEdit, words: &Words) -> Option<PolicyFragment> {
    let mut result = PolicyFragment { nodes: 1, text_bytes: 0, text_per_node: 0, depth: 3 };
    match value {
        PolicyEdit::ProjectSpend { .. } => result.phrase(words, JigPhrase::PolicyPeriodSpend)?,
        PolicyEdit::Role(role) => {
            for phrase in [
                JigPhrase::PolicyEditRole,
                JigPhrase::PolicyPeriodSpend,
                JigPhrase::PolicyRequests,
                JigPhrase::PolicyDecides,
            ] {
                result.phrase(words, phrase)?;
            }
            let authority = crate::owner_authority::required_words(&role.authority, words)?;
            result.nodes = result.nodes.checked_add(authority.nodes)?;
            result.text_bytes = result.text_bytes.checked_add(authority.text_bytes)?;
            result.text_per_node = result.text_per_node.max(authority.text_per_node);
        }
        PolicyEdit::Requirements { requirements } => {
            result.phrase(words, JigPhrase::PolicyRequirements)?;
            for requirement in requirements {
                result.phrase(words, JigPhrase::PolicyEffect)?;
                result.pattern(&requirement.pattern, words)?;
                result.phrase(words, JigPhrase::PolicyJudge)?;
                result.phrase(
                    words,
                    match requirement.guard {
                        PolicyGuard::Guarded => JigPhrase::PolicyGuarded,
                        PolicyGuard::Observed { .. } => JigPhrase::PolicyObserved,
                    },
                )?;
                result.phrase(
                    words,
                    if requirement.must_be_guarded {
                        JigPhrase::PolicyMustGuard
                    } else {
                        JigPhrase::PolicyGuardNotRequired
                    },
                )?;
            }
        }
        PolicyEdit::Projections { grants } => {
            result.phrase(words, JigPhrase::PolicyProjections)?;
            for grant in grants {
                result.phrase(words, JigPhrase::PolicyEffect)?;
                result.pattern(&grant.pattern, words)?;
            }
        }
        PolicyEdit::Permissions { mappings } => {
            result.phrase(words, JigPhrase::PolicyPermissions)?;
            for _ in mappings {
                result.phrase(words, JigPhrase::PolicyPermission)?;
            }
        }
    }
    Some(result)
}
fn pattern(b: &mut Builder, value: &OwnerPattern, words: &Words) {
    for segment in &value.segments {
        crate::literal_segment::render(b, segment);
    }
    let (phrase, bytes) = match &value.last {
        OwnerPatternEnd::Exact(bytes) => (crate::owner_authority_words::AuthorityPhrase::Exact, bytes),
        OwnerPatternEnd::Open(bytes) => (crate::owner_authority_words::AuthorityPhrase::Open, bytes),
    };
    let label = words.book.say(words.language, phrase.id(), &[]).expect("checked terminal label").text;
    b.text(Run::Plain, &label);
    crate::literal_segment::render(b, bytes);
}
fn effect(b: &mut Builder, connector: u16, kind: u16, words: &Words) {
    words.text(b, JigPhrase::PolicyEffect, &[Hole::Number(u64::from(connector)), Hole::Number(u64::from(kind))]);
}
/// Display one whole edit without inventing role names or parsing resource names.
pub fn render(b: &mut Builder, value: &PolicyEdit, words: &Words) {
    b.open(Role::Region);
    match value {
        PolicyEdit::ProjectSpend { period_spend } => {
            words.text(b, JigPhrase::PolicyPeriodSpend, &[Hole::Amount(*period_spend)]);
        }
        PolicyEdit::Role(role) => {
            words.text(b, JigPhrase::PolicyEditRole, &[Hole::Number(u64::from(role.number))]);
            crate::owner_authority::render_words(b, &role.authority, words);
            words.text(b, JigPhrase::PolicyPeriodSpend, &[Hole::Amount(role.period_spend)]);
            words.text(b, JigPhrase::PolicyRequests, &[Hole::Number(u64::from(role.requests))]);
            words.text(b, JigPhrase::PolicyDecides, &[Hole::Number(u64::from(role.decides))]);
        }
        PolicyEdit::Requirements { requirements } => {
            words.text(b, JigPhrase::PolicyRequirements, &[]);
            for requirement in requirements {
                effect(b, requirement.connector, requirement.kind, words);
                pattern(b, &requirement.pattern, words);
                words.text(
                    b,
                    JigPhrase::PolicyJudge,
                    &[
                        Hole::Number(u64::from(requirement.judge.connector)),
                        Hole::Number(u64::from(requirement.judge.requirement)),
                        Hole::Number(u64::from(requirement.judge.parameters)),
                    ],
                );
                match requirement.guard {
                    PolicyGuard::Guarded => words.text(b, JigPhrase::PolicyGuarded, &[]),
                    PolicyGuard::Observed { freshness } => {
                        words.text(b, JigPhrase::PolicyObserved, &[Hole::Number(freshness.as_nanos())]);
                    }
                }
                words.text(
                    b,
                    if requirement.must_be_guarded {
                        JigPhrase::PolicyMustGuard
                    } else {
                        JigPhrase::PolicyGuardNotRequired
                    },
                    &[],
                );
            }
        }
        PolicyEdit::Projections { grants } => {
            words.text(b, JigPhrase::PolicyProjections, &[]);
            for grant in grants {
                effect(b, grant.connector, grant.kind, words);
                pattern(b, &grant.pattern, words);
            }
        }
        PolicyEdit::Permissions { mappings } => {
            words.text(b, JigPhrase::PolicyPermissions, &[]);
            for mapping in mappings {
                words.text(
                    b,
                    JigPhrase::PolicyPermission,
                    &[
                        Hole::Number(u64::from(mapping.connector)),
                        Hole::Number(u64::from(mapping.permission)),
                        Hole::Number(u64::from(mapping.role)),
                    ],
                );
            }
        }
    }
    b.close();
}
