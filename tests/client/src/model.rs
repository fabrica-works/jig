//! A platform tree changed only by patches (client/domain/view.md, 6).
use jig_view::{Made, NodeId, Patch, Tree};
#[derive(Debug)]
struct Entry {
    node: Made,
    parent: NodeId,
}

fn entries(nodes: &[Made], parent: NodeId) -> Vec<Entry> {
    let mut rows = Vec::new();
    let mut stack: Vec<(NodeId, u32)> = Vec::new();
    for (index, node) in nodes.iter().enumerate() {
        let index = u32::try_from(index).expect("small test tree");
        while stack.last().is_some_and(|(_, end)| index >= *end) {
            stack.pop();
        }
        let parent = stack.last().map_or(parent, |(id, _)| *id);
        rows.push(Entry { node: node.clone(), parent });
        stack.push((node.id, index.checked_add(node.size).expect("small subtree")));
    }
    rows
}

fn position(rows: &[Entry], id: NodeId) -> usize {
    rows.iter().position(|row| row.node.id == id).expect("patch names a present node")
}

fn sizes(rows: &mut [Entry]) {
    // Preorder makes every subtree contiguous. Close each ancestor once,
    // when its next sibling begins, rather than scanning every child.
    let mut open: Vec<usize> = Vec::new();
    for index in 0..rows.len() {
        while open.last().is_some_and(|parent| rows[*parent].node.id != rows[index].parent) {
            let parent = open.pop().expect("open ancestor");
            rows[parent].node.size = u32::try_from(index - parent).expect("small subtree");
        }
        assert!(rows[index].parent == NodeId(0) || !open.is_empty(), "preorder parent is present");
        open.push(index);
    }
    while let Some(parent) = open.pop() {
        rows[parent].node.size = u32::try_from(rows.len() - parent).expect("small subtree");
    }
}

fn insertion(rows: &[Entry], parent: NodeId, before: Option<NodeId>) -> usize {
    match before {
        Some(id) => position(rows, id),
        None => {
            if parent == NodeId(0) {
                rows.len()
            } else {
                let at = position(rows, parent);
                at.checked_add(usize::try_from(rows[at].node.size).expect("small subtree")).expect("small tree")
            }
        }
    }
}

fn apply(rows: &mut Vec<Entry>, patch: Patch) {
    match patch {
        Patch::Insert { parent, before, nodes } => {
            let at = insertion(rows, parent, before);
            drop(rows.splice(at..at, entries(&nodes, parent)));
            sizes(rows);
        }
        Patch::Remove { node } => {
            let at = position(rows, node);
            let end = at.checked_add(usize::try_from(rows[at].node.size).expect("small subtree")).expect("small tree");
            drop(rows.drain(at..end));
            sizes(rows);
        }
        Patch::Move { node, parent, before } => {
            let at = position(rows, node);
            let end = at.checked_add(usize::try_from(rows[at].node.size).expect("small subtree")).expect("small tree");
            let mut moved: Vec<_> = rows.drain(at..end).collect();
            moved[0].parent = parent;
            sizes(rows);
            let to = insertion(rows, parent, before);
            drop(rows.splice(to..to, moved));
            sizes(rows);
        }
        Patch::Text { node, text } => {
            let at = position(rows, node);
            rows[at].node.text = text;
        }
        Patch::Attribute { node, attribute } => {
            let at = position(rows, node);
            rows[at].node.attribute = attribute;
        }
        Patch::Value { node, value } => {
            let at = position(rows, node);
            rows[at].node.value = value;
        }
    }
}

fn equal(rows: &[Entry], tree: &Tree) {
    assert_eq!(rows.len(), usize::try_from(tree.len()).expect("small tree"), "platform-visible node count");
    for (row, node) in rows.iter().zip(tree.nodes().iter()) {
        let made = &row.node;
        assert_eq!((made.id, made.size, &made.value), (node.id, node.size, &node.value), "platform-visible node");
        let attribute = &made.attribute;
        assert_eq!(
            (
                attribute.role,
                &attribute.name,
                attribute.linked,
                &attribute.external,
                attribute.bound,
                &attribute.drawing
            ),
            (
                node.role,
                &node.name,
                node.link.is_some(),
                &node.external,
                node.binding.is_some() || node.link.is_some(),
                &node.drawing
            ),
            "platform-visible attributes"
        );
        assert!(attribute.states.iter().eq(node.states.iter()), "platform-visible states");
        assert!(attribute.styles.iter().eq(node.styles.iter()), "platform-visible styles");
        assert!(made.text.iter().eq(node.text.iter()), "platform-visible text runs");
    }
}

/// The platform’s tree, independent of the renderer’s retained tree.
#[derive(Debug, Default)]
pub struct Model {
    rows: Vec<Entry>,
}
impl Model {
    /// Mirrors the platform’s local field edit.
    pub fn input(&mut self, node: NodeId, text: &[u8]) {
        let at = position(&self.rows, node);
        if let Some(value) = self.rows[at].node.value.as_mut() {
            value.text = Box::from(text);
        }
    }
    /// Applies one platform patch without rebuilding from the view.
    pub fn apply(&mut self, patch: Patch) {
        apply(&mut self.rows, patch);
    }
    /// Checks the complete platform projection against the view.
    pub fn check(&self, tree: &Tree) {
        equal(&self.rows, tree);
    }
    /// Node identities in the platform projection before a changed page.
    pub fn ids(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.rows.iter().map(|row| row.node.id)
    }
    /// Returns the platform projection for an external observer.
    #[must_use]
    pub fn nodes(&self) -> Vec<Made> {
        self.rows.iter().map(|row| row.node.clone()).collect()
    }
}
