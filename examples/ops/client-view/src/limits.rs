//! Keep: checked child sums and both slot passes (view.md, 10).
//! Replace: application family and connector node bounds.
use crate::{bindings, drawing, words::Words};
/// Bounds for the composed tree and owner bindings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub tree: jig_client_view::Limits,
}
impl Limits {
    #[must_use]
    pub fn of(
        domain: &jig_ops_client_domain::Limits,
        jig: &jig_client_view::words::Words,
        observability: &Words,
        infrastructure: &Words,
    ) -> Option<Self> {
        let rows = domain.observability.rows.max(domain.infrastructure.rows);
        // Each nested row contributes at most eighty fact, heading, form or fallback nodes.
        // The fixed part reserve includes both deferred Deployment diagnostics.
        let part = rows
            .checked_add(1)?
            .checked_mul(rows.checked_add(1)?)?
            .checked_mul(80)?
            .checked_add(34)?
            .checked_add(jig_client_view::refusal_nodes(&domain.jig)?)?;
        let application_nodes = part.checked_mul(rows.checked_add(1)?)?;
        // A full connector document is reached through a chip phase, runtime,
        // effect, hold, approval, project or deployment slot. Nested rows bound
        // result/verdict/proposal and chip dependencies; object counts bound
        // repeated cards and their dependency chips. Each resource slot adds a
        // label and one literal node per separately supplied bounded segment.
        let documents = domain
            .jig
            .rows
            .checked_add(1)?
            .checked_pow(3)?
            .checked_mul(domain.jig.objects.checked_add(1)?.checked_pow(2)?)?
            .checked_mul(8)?;
        let jig_nodes = jig_client_view::Limits::of_words(&domain.jig, jig)?.nodes;
        let jig_cards = jig_client_view::card_nodes(&domain.jig)?.checked_mul(rows.checked_add(1)?)?;
        let slot_nodes = documents
            .checked_mul(part)?
            .checked_add(jig_nodes.checked_mul(rows.checked_add(3)?)?)?
            .checked_add(jig_cards)?
            .checked_add(8)?;
        let mut tree = jig_client_view::Limits::compose(&domain.jig, jig, application_nodes, slot_nodes, 8)?;
        tree.text_per_node = tree
            .text_per_node
            .max(observability.capacity()?.max(infrastructure.capacity()?).checked_mul(2)?.checked_add(32)?)
            .max(jig_client_view::literal_segment::max_bytes(
                domain.observability.text.max(domain.infrastructure.text),
            )?);
        tree.text_bytes = tree.nodes.checked_mul(tree.text_per_node)?;
        tree.drawing_bytes = drawing::BYTES;
        let limits = Self { tree };
        worst_case(&limits, domain, jig, observability, infrastructure)?;
        Some(limits)
    }
}
#[must_use]
pub fn worst_case(
    limits: &Limits,
    domain: &jig_ops_client_domain::Limits,
    jig: &jig_client_view::words::Words,
    observability: &Words,
    infrastructure: &Words,
) -> Option<u64> {
    jig_client_view::worst_case(&limits.tree)?
        .checked_add(jig_client_view::jig_bindings_worst_case(limits.tree.bindings, &domain.jig)?)?
        .checked_add(bindings::worst_case(limits.tree.bindings)?.checked_mul(2)?)?
        .checked_add(jig_client_view::phrases::worst_case(jig.book.limits())?)?
        .checked_add(jig_client_view::phrases::worst_case(observability.book.limits())?)?
        .checked_add(jig_client_view::phrases::worst_case(infrastructure.book.limits())?)?
        .checked_add(u64::from(
            observability
                .capacity()?
                .max(infrastructure.capacity()?)
                .max(jig.book.capacity(jig_client_view::phrases::jig::JigPhrase::LoadingPart.id())?),
        ))
}
