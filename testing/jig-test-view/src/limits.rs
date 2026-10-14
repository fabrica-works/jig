//! Checked composition includes both slot passes (view.md, section 10).
use crate::{bindings, drawing, family, words};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub tree: jig_view::Limits,
    pub text: u32,
}
impl Limits {
    /// Storage declared by a world, rather than every combination admitted by
    /// the domain. Builder checks reject fixtures that exceed this profile;
    /// `of` remains the complete composition envelope.
    #[must_use]
    pub fn for_world(
        domain: &jig_test_domain_client::Limits,
        jig: &jig_view::words::Words,
        first: &words::Words,
        nodes: u32,
        slots: u32,
    ) -> Option<Self> {
        if nodes == 0 || slots > nodes {
            return None;
        }
        let owner = crate::connector_values::requirements(&domain.first.connector, jig)?;
        let fragment_nodes =
            family::PART_NODES.max(owner.nodes.checked_add(16)?).max(domain.first.rows.checked_add(2)?);
        let cards = domain.first.rows.checked_add(1)?;
        let application_nodes = fragment_nodes.checked_add(cards.checked_mul(3)?)?.checked_add(3)?;
        let mut tree = jig_view::Limits::of_words(&domain.jig, jig)?;
        let jig_nodes = tree.nodes;
        let jig_cards = u64::from(jig_view::card_nodes(&domain.jig)?).checked_mul(u64::from(cards))?;
        // The complete node ceiling is checked without constructing its storage.
        // Only the world's declared counts need text and patch capacities in u32.
        let slot_nodes = u64::from(jig_nodes)
            .checked_add(jig_cards)?
            .checked_mul(u64::from(fragment_nodes))?
            .checked_add(jig_cards)?
            .checked_add(u64::from(family::NAV_NODES))?;
        let jig_page = jig_view::page_nodes(&domain.jig, 0)?;
        let full_nodes = u64::from(jig_nodes.checked_sub(jig_page)?)
            .checked_add(u64::from(jig_page.max(application_nodes)))?
            .checked_add(slot_nodes)?;
        if u64::from(nodes) > full_nodes {
            return None;
        }
        tree.text_per_node = tree
            .text_per_node
            .max(first.capacity()?.checked_mul(2)?.checked_add(20)?)
            .max(owner.text_per_node)
            .max(jig_view::literal_segment::max_bytes(domain.first.text)?);
        tree.nodes = nodes;
        tree.depth = tree.depth.checked_add(owner.depth.checked_add(4)?.max(8))?;
        tree.bindings = nodes;
        tree.slots = slots;
        tree.patches = nodes.checked_mul(5)?;
        tree.text_bytes = nodes.checked_mul(tree.text_per_node)?;
        tree.drawing_bytes = drawing::BYTES;
        let limits = Self { tree, text: domain.first.text };
        worst_case(&limits, domain, jig, first)?;
        Some(limits)
    }
    #[must_use]
    pub fn of(
        domain: &jig_test_domain_client::Limits,
        jig: &jig_view::words::Words,
        first: &words::Words,
    ) -> Option<Self> {
        let owner = crate::connector_values::requirements(&domain.first.connector, jig)?;
        let fragment_nodes =
            family::PART_NODES.max(owner.nodes.checked_add(16)?).max(domain.first.rows.checked_add(2)?);
        let cards = domain.first.rows.checked_add(1)?;
        let application_nodes = fragment_nodes.checked_add(cards.checked_mul(3)?)?.checked_add(3)?;
        // All nodes bound the possible slots in the public jig envelope.
        // This conservative bound also covers duplicate references to one part.
        let jig_nodes = jig_view::Limits::of_words(&domain.jig, jig)?.nodes;
        let jig_cards = jig_view::card_nodes(&domain.jig)?.checked_mul(cards)?;
        let part_nodes = jig_nodes.checked_add(jig_cards)?.checked_mul(fragment_nodes)?;
        let slot_nodes = part_nodes.checked_add(jig_cards)?.checked_add(family::NAV_NODES)?;
        let mut tree = jig_view::Limits::compose(
            &domain.jig,
            jig,
            &jig_domain_client::pages::Arrangement::none(),
            application_nodes,
            slot_nodes,
            owner.depth.checked_add(4)?.max(8),
        )?;
        tree.text_per_node = tree
            .text_per_node
            .max(first.capacity()?.checked_mul(2)?.checked_add(20)?)
            .max(owner.text_per_node)
            .max(jig_view::literal_segment::max_bytes(domain.first.text)?);
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
    domain: &jig_test_domain_client::Limits,
    jig: &jig_view::words::Words,
    first: &words::Words,
) -> Option<u64> {
    jig_view::worst_case(&limits.tree)?
        .checked_add(jig_view::jig_bindings_worst_case(limits.tree.bindings, &domain.jig)?)?
        .checked_add(bindings::worst_case(limits.tree.bindings)?)?
        .checked_add(jig_view::phrases::worst_case(jig.book.limits())?)?
        .checked_add(jig_view::phrases::worst_case(first.book.limits())?)?
        .checked_add(u64::from(jig.book.capacity(jig_view::phrases::jig::JigPhrase::LoadingPart.id())?))?
        .checked_add(u64::from(first.capacity()?))
}
