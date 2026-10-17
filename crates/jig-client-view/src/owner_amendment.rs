//! Standalone amendment settings (client/domain/objects.md, 4 and 11;
//! view.md, 7 and 10). Context comes from the enclosing owner document.
use crate::owner_amendment_words::AmendmentPhrase as P;
use crate::words::Words;
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role, Run};

use crate::Limits;
use jig_client::contract::{
    DocumentText, OwnerAmendmentState, OwnerAuthorityDocument, OwnerPattern, OwnerPatternEnd, ParameterDocument,
    PersonAddress,
};

/// Storage for a whole standalone state, including all nested fragments.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmendmentFragment {
    pub nodes: u32,
    pub text_bytes: u32,
    pub text_per_node: u32,
    pub depth: u32,
    pub bindings: u32,
    pub slots: u32,
}
fn text_fits(value: &DocumentText, limits: &Limits) -> Option<()> {
    if value.bytes.fits(limits.text) { Some(()) } else { None }
}
fn pattern_fits(value: &OwnerPattern, limits: &Limits) -> Option<()> {
    if value.segments.capacity() > limits.rows {
        return None;
    }
    for segment in &value.segments {
        if u32::try_from(segment.len()).ok()? > limits.text {
            return None;
        }
    }
    let last = match &value.last {
        OwnerPatternEnd::Exact(bytes) | OwnerPatternEnd::Open(bytes) => bytes,
    };
    if u32::try_from(last.len()).ok()? > limits.text {
        return None;
    }
    Some(())
}
pub(crate) fn authority_fits(value: &OwnerAuthorityDocument, limits: &Limits) -> Option<()> {
    let value = &value.authority;
    if value.grants.capacity() > limits.rows
        || value.delegation.kinds.capacity() > limits.rows
        || value.note_resources.capacity() > limits.rows
    {
        return None;
    }
    for grant in &value.grants {
        pattern_fits(&grant.pattern, limits)?;
    }
    for resource in &value.note_resources {
        pattern_fits(&resource.pattern, limits)?;
    }
    Some(())
}
/// Validate every nested text and allocated list capacity before reserving a whole fragment.
#[must_use]
pub fn required(value: &OwnerAmendmentState, limits: &Limits, words: &Words) -> Option<AmendmentFragment> {
    if let Some(instructions) = &value.instructions {
        text_fits(instructions, limits)?;
    }
    let spec = &value.spec.specification;
    text_fits(&spec.words, limits)?;
    if value.dependencies.capacity() > limits.rows
        || spec.parameters.capacity() > limits.rows
        || spec.inputs.capacity() > limits.rows
    {
        return None;
    }
    for parameter in &spec.parameters {
        match parameter {
            ParameterDocument::Bytes { value, .. } => text_fits(value, limits)?,
            ParameterDocument::Number { .. } | ParameterDocument::Resource { .. } => {}
        }
    }
    authority_fits(&value.authority, limits)?;
    worst_case(limits, words)
}
/// Finite whole-node envelope shared by standalone and enclosing page renderers.
#[must_use]
pub fn max_nodes(limits: &Limits) -> Option<u32> {
    20_u32
        .checked_add(skein_ui::markdown::max_nodes(limits.text)?)?
        .checked_add(crate::specification::nodes(limits)?)?
        .checked_add(crate::owner_authority::document_nodes(limits.rows)?)?
        .checked_add(limits.rows)
}
/// Configured envelope; it does not attest that an absent field was supplied.
/// The authority fragment includes escaped opaque pattern bytes; other fragment
/// bounds include Markdown, per-text loss, specification arrays and wake facts.
#[must_use]
pub fn worst_case(limits: &Limits, words: &Words) -> Option<AmendmentFragment> {
    let nodes = max_nodes(limits)?;
    let phrase_bytes = crate::phrases::jig_capacity(&words.book)?;
    let text_per_node = crate::literal_segment::max_bytes(limits.text)?.max(phrase_bytes).checked_mul(2)?;
    Some(AmendmentFragment {
        nodes,
        text_bytes: nodes.checked_mul(text_per_node)?,
        text_per_node,
        depth: 8,
        bindings: 0,
        slots: 0,
    })
}
fn saying(b: &mut Builder, words: &Words, phrase: P, holes: &[Hole]) {
    let value = words.book.say(words.language, phrase.id(), holes).expect("checked amendment sayings and typed holes");
    b.text(Run::Plain, &value.text);
}
/// Render only supplied owner facts, borrowing the normal checked language book.
/// No request, revision, actor, project, clock, origin or subscription is inferred.
pub fn render(b: &mut Builder, value: &OwnerAmendmentState, words: &Words) {
    b.open(Role::Region);
    let name = words.book.say(words.language, P::State.id(), &[]).expect("checked amendment region name");
    b.name(&name.text);
    match value.addressed {
        Some(PersonAddress::Party(party)) => saying(b, words, P::Party, &[Hole::Number(party)]),
        Some(PersonAddress::Role(role)) => saying(b, words, P::Role, &[Hole::Number(u64::from(role))]),
        None => saying(b, words, P::AddressAbsent, &[]),
    }
    match value.taken_by {
        Some(party) => saying(b, words, P::TakenBy, &[Hole::Number(party)]),
        None => saying(b, words, P::TakerAbsent, &[]),
    }
    saying(b, words, P::Instructions, &[]);
    crate::instructions::document(b, value.instructions.as_ref(), words);
    saying(b, words, P::Specification, &[]);
    crate::specification::document(b, &value.spec, words);
    saying(b, words, P::Authority, &[]);
    crate::owner_authority::document_words(b, &value.authority, words);
    crate::owner_wake::policy(b, &value.wake, &words.book, words.language);
    for task in &value.dependencies {
        saying(b, words, P::Dependencies, &[Hole::Number(*task)]);
    }
    if value.cut {
        saying(b, words, P::DependenciesCut, &[]);
    }
    b.close();
}
