//! Checked composition includes both slot passes (view.md, section 10).
use crate::{bindings, drawing, family, words};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub tree: jig_client_view::Limits,
    pub text: u32,
}
impl Limits {
    /// Storage declared by a world, rather than every combination admitted by
    /// the domain. Builder checks reject fixtures that exceed this profile;
    /// `of` remains the complete composition envelope.
    #[must_use]
    pub fn for_world(
        domain: &jig_test_client_domain::Limits,
        jig: &jig_client_view::words::Words,
        first: &words::Words,
        nodes: u32,
        slots: u32,
    ) -> Option<Self> {
        if nodes == 0 || slots > nodes {
            return None;
        }
        let mut limits = Self::of(domain, jig, first)?;
        if nodes > limits.tree.nodes {
            return None;
        }
        limits.tree.nodes = nodes;
        limits.tree.bindings = nodes;
        limits.tree.slots = slots;
        limits.tree.patches = nodes.checked_mul(5)?;
        limits.tree.text_bytes = nodes.checked_mul(limits.tree.text_per_node)?;
        worst_case(&limits, domain, jig, first)?;
        Some(limits)
    }
    #[must_use]
    pub fn of(
        domain: &jig_test_client_domain::Limits,
        jig: &jig_client_view::words::Words,
        first: &words::Words,
    ) -> Option<Self> {
        let cards = domain.first.rows.checked_add(1)?;
        let application_nodes = family::PART_NODES.checked_add(cards.checked_mul(3)?)?.checked_add(3)?;
        // All nodes bound the possible slots in the public jig envelope.
        // This conservative bound also covers duplicate references to one part.
        let jig_nodes = jig_client_view::Limits::of_words(&domain.jig, jig)?.nodes;
        let jig_cards = jig_client_view::card_nodes(&domain.jig)?.checked_mul(cards)?;
        let part_nodes = jig_nodes.checked_add(jig_cards)?.checked_mul(family::PART_NODES)?;
        let slot_nodes = part_nodes.checked_add(jig_cards)?.checked_add(family::NAV_NODES)?;
        let mut tree = jig_client_view::Limits::compose(&domain.jig, jig, application_nodes, slot_nodes, 8)?;
        tree.text_per_node = tree.text_per_node.max(first.capacity()?.checked_mul(2)?.checked_add(20)?);
        tree.text_bytes = tree.nodes.checked_mul(tree.text_per_node)?;
        tree.drawing_bytes = drawing::BYTES;
        let limits = Self { tree, text: domain.first.text };
        worst_case(&limits, domain, jig, first)?;
        Some(limits)
    }
}
#[must_use]
pub fn worst_case(
    limits: &Limits,
    domain: &jig_test_client_domain::Limits,
    jig: &jig_client_view::words::Words,
    first: &words::Words,
) -> Option<u64> {
    jig_client_view::worst_case(&limits.tree)?
        .checked_add(jig_client_view::jig_bindings_worst_case(limits.tree.bindings, &domain.jig)?)?
        .checked_add(bindings::worst_case(limits.tree.bindings)?)?
        .checked_add(jig_client_view::phrases::worst_case(jig.book.limits())?)?
        .checked_add(jig_client_view::phrases::worst_case(first.book.limits())?)?
        .checked_add(u64::from(jig.book.capacity(jig_client_view::phrases::jig::JigPhrase::LoadingPart.id())?))?
        .checked_add(u64::from(first.capacity()?))
}
