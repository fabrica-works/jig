//! Configured tree storage (client/domain/view.md, section 10).
//! The caller derives these capacities from the domain and its own fragments.
//! `worst_case` counts owned heap, excluding allocator overhead, for the visible
//! tree, builders and their splice scratch, diff scratch and output patches.

use crate::builder::PlacedSlot;
use crate::diff::Open;
use crate::{Made, Node, Patch, Slot, State, Style, TextRun};
use skein_lib::{List, Queue, Stack};

/// Capacities checked before each builder allocation.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub nodes: u32,
    pub depth: u32,
    /// Name, runs, field value and external target together on a node.
    pub text_per_node: u32,
    /// Name, runs, field value and external target together across the tree.
    pub text_bytes: u32,
    pub drawing_bytes: u32,
    pub styles_per_node: u32,
    pub patches: u32,
    pub bindings: u32,
    pub slots: u32,
    pub markdown_depth: u32,
}

/// Heap for rendering, splicing, diff scratch and the largest patch payloads.
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    let nodes = List::<Node>::worst_case(limits.nodes)?;
    let states = List::<State>::worst_case(9)?.checked_mul(u64::from(limits.nodes))?;
    let styles = List::<Style>::worst_case(limits.styles_per_node)?.checked_mul(u64::from(limits.nodes))?;
    let runs = List::<TextRun>::worst_case(limits.text_per_node)?.checked_mul(u64::from(limits.nodes))?;
    let drawings = u64::from(limits.drawing_bytes).checked_mul(u64::from(limits.nodes))?;
    let payloads = u64::from(limits.text_bytes).checked_add(drawings)?;
    let tree = nodes.checked_add(states)?.checked_add(styles)?.checked_add(runs)?.checked_add(payloads)?;
    let trees = tree.checked_mul(5)?.checked_add(nodes.checked_mul(2)?)?;
    let stacks = Stack::<u32>::worst_case(limits.depth)?.checked_mul(3)?;
    let slots = List::<PlacedSlot>::worst_case(limits.slots)?
        .checked_mul(3)?
        .checked_add(List::<Slot>::worst_case(limits.slots)?.checked_mul(2)?)?;
    let patches = Queue::<Patch>::worst_case(limits.patches)?;
    let made = List::<Made>::worst_case(limits.nodes)?;
    let maps = List::<Option<u32>>::worst_case(limits.nodes)?.checked_mul(4)?;
    let children = List::<u32>::worst_case(limits.nodes)?;
    let ancestry = Stack::<Open>::worst_case(limits.depth)?.checked_mul(2)?;
    let markdown = crate::markdown::worst_case(limits.text_bytes, limits.markdown_depth)?;
    trees
        .checked_add(markdown)?
        .checked_add(stacks)?
        .checked_add(slots)?
        .checked_add(patches)?
        .checked_add(made)?
        .checked_add(maps)?
        .checked_add(children)?
        .checked_add(ancestry)?
        .checked_add(crate::pages::scratch(limits)?)
}

impl Limits {
    /// Whole jig composition in one enclosing page, with English's phrase
    /// envelope (256 literal bytes, at most three holes). Connector
    /// fragments and application pages use `compose`. Custom
    /// languages use `of_words` to include their checked saying capacities.
    #[must_use]
    pub fn of(client: &jig_client::Limits) -> Option<Self> {
        let phrase = client.text.max(64).checked_mul(3)?.checked_add(256)?;
        Self::with_phrase_bytes(client, phrase, 0, 0, 0)
    }

    /// The same renderer envelope using the largest checked saying across
    /// every loaded language; the root may switch language without resizing.
    #[must_use]
    pub fn of_words(client: &jig_client::Limits, words: &crate::words::Words) -> Option<Self> {
        let phrase = words.book.jig_capacity()?;
        Self::with_phrase_bytes(client, phrase, 0, 0, 0)
    }

    /// Compose shared siblings around the larger jig or application page.
    /// `slot_nodes` sums all owners' fragment bounds; `slot_depth` bounds their
    /// additional nesting. Roots enlarge text/drawing/style bounds for their
    /// own fragments when those exceed jig's checked saying envelope.
    #[must_use]
    pub fn compose(
        client: &jig_client::Limits,
        words: &crate::words::Words,
        application_nodes: u32,
        slot_nodes: u32,
        slot_depth: u32,
    ) -> Option<Self> {
        let phrase = words.book.jig_capacity()?;
        Self::with_phrase_bytes(client, phrase, application_nodes, slot_nodes, slot_depth)
    }
    fn with_phrase_bytes(
        client: &jig_client::Limits,
        phrase: u32,
        application_nodes: u32,
        slot_nodes: u32,
        slot_depth: u32,
    ) -> Option<Self> {
        let nodes = crate::page_nodes(client, 0)?
            .max(application_nodes)
            .checked_add(crate::frame_nodes(client)?)?
            .checked_add(crate::confirmation_nodes(client)?)?
            .checked_add(crate::notices_nodes(client)?)?
            .checked_add(slot_nodes)?
            .checked_add(1)?;
        // A field carries a saying as its name and the full domain value.
        // Markdown links may carry both label and external target.
        let text_per_node = phrase
            .checked_mul(2)?
            .checked_add(client.text.checked_mul(2)?)?
            .max(crate::literal_segment::max_bytes(client.text)?.checked_mul(2)?);
        Some(Self {
            nodes,
            // Page, card/input result, verdict proposal, and its deepest
            // Markdown list path. Dependency regions are siblings, not nested.
            depth: client.rows.checked_add(32)?.checked_add(slot_depth)?,
            text_per_node,
            text_bytes: nodes.checked_mul(text_per_node)?,
            drawing_bytes: 0,
            styles_per_node: 2,
            // At most one remove per old node and one structural plus three
            // attribute edits per new node (diff.rs).
            patches: nodes.checked_mul(5)?,
            // Each actionable node or part slot issues at most one meaning.
            bindings: nodes,
            slots: nodes,
            markdown_depth: 3,
        })
    }
}
