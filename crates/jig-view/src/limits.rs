//! jig's view limits and page scratch (client/domain/view.md, section 10).

use skein_ui::Limits;

/// Whole jig composition in one enclosing page, with English's phrase
/// envelope (256 literal bytes, at most three holes). Connector
/// fragments and application pages use `compose`. Custom
/// languages use `of_words` to include their checked saying capacities.
#[must_use]
pub fn of(client: &jig_domain_client::Limits, arrangement: &jig_domain_client::pages::Arrangement) -> Option<Limits> {
    let phrase = client.text.max(64).checked_mul(3)?.checked_add(256)?;
    with_phrase_bytes(client, phrase, arrangement, 0, 0, 0)
}

/// The same renderer envelope using the largest checked saying across
/// every loaded language; the root may switch language without resizing.
#[must_use]
pub fn of_words(client: &jig_domain_client::Limits, words: &crate::words::Words) -> Option<Limits> {
    let phrase = crate::phrases::jig_capacity(&words.book)?;
    with_phrase_bytes(client, phrase, &jig_domain_client::pages::Arrangement::none(), 0, 0, 0)
}

/// Compose shared siblings around the larger jig or application page.
/// `slot_nodes` sums all owners' fragment bounds; `slot_depth` bounds their
/// additional nesting. Roots enlarge text/drawing/style bounds for their
/// own fragments when those exceed jig's checked saying envelope.
#[must_use]
pub fn compose(
    client: &jig_domain_client::Limits,
    words: &crate::words::Words,
    arrangement: &jig_domain_client::pages::Arrangement,
    application_nodes: u32,
    slot_nodes: u32,
    slot_depth: u32,
) -> Option<Limits> {
    let phrase = crate::phrases::jig_capacity(&words.book)?;
    with_phrase_bytes(client, phrase, arrangement, application_nodes, slot_nodes, slot_depth)
}
fn with_phrase_bytes(
    client: &jig_domain_client::Limits,
    phrase: u32,
    arrangement: &jig_domain_client::pages::Arrangement,
    application_nodes: u32,
    slot_nodes: u32,
    slot_depth: u32,
) -> Option<Limits> {
    let mut limits = envelope(client, phrase, arrangement, application_nodes, slot_nodes, slot_depth)?;
    limits.text_bytes = limits.nodes.checked_mul(limits.text_per_node)?;
    limits.patches = limits.nodes.checked_mul(5)?;
    Some(limits)
}

/// Node and attribute envelope before storage products. A bounded world
/// supplies its own node, binding, slot, text and patch capacities.
#[must_use]
pub fn envelope(
    client: &jig_domain_client::Limits,
    phrase: u32,
    arrangement: &jig_domain_client::pages::Arrangement,
    application_nodes: u32,
    slot_nodes: u32,
    slot_depth: u32,
) -> Option<Limits> {
    let nodes = crate::arranged_nodes(client, 0, arrangement)?
        .max(application_nodes)
        .checked_add(crate::banner_nodes(client)?)?
        .checked_add(crate::navigation_nodes(client)?)?
        .checked_add(crate::confirmation_nodes(client)?)?
        .checked_add(crate::notices_nodes(client)?)?
        .checked_add(slot_nodes)?
        .checked_add(3)?;
    // A field carries a saying as its name and the full domain value.
    // Markdown links may carry both label and external target.
    let text_per_node = phrase
        .checked_mul(2)?
        .checked_add(client.text.checked_mul(2)?)?
        .max(crate::literal_segment::max_bytes(client.text)?.checked_mul(2)?);
    Some(Limits {
        nodes,
        // Page, card/input result, verdict proposal, and its deepest
        // Markdown list path. Dependency regions are siblings, not nested.
        depth: client.rows.checked_add(33)?.checked_add(slot_depth)?,
        text_per_node,
        text_bytes: 0,
        drawing_bytes: 0,
        styles_per_node: 2,
        // At most one remove per old node and one structural plus three
        // attribute edits per new node (diff.rs).
        patches: 0,
        // Each actionable node or part slot issues at most one meaning.
        bindings: nodes,
        slots: nodes,
        markdown_depth: 3,
        scene_marks: 0,
        scene_bytes: 0,
    })
}

/// Heap for the UI containers and jig's page-building scratch.
#[must_use]
pub fn jig_worst_case(limits: &Limits) -> Option<u64> {
    skein_ui::worst_case(limits)?.checked_add(crate::pages::scratch(limits)?)
}
