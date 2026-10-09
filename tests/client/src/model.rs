//! A platform tree changed only by patches (client/domain/view.md, 6).
use jig_client_view::{Made, NodeId, Patch, Tree};
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
    for index in (0..rows.len()).rev() {
        let id = rows[index].node.id;
        let mut size = 1_u32;
        for child in rows.iter() {
            if child.parent == id {
                size = size.checked_add(child.node.size).expect("small tree");
            }
        }
        rows[index].node.size = size;
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
        }
        Patch::Remove { node } => {
            let at = position(rows, node);
            let end = at.checked_add(usize::try_from(rows[at].node.size).expect("small subtree")).expect("small tree");
            drop(rows.drain(at..end));
        }
        Patch::Move { node, parent, before } => {
            let at = position(rows, node);
            let end = at.checked_add(usize::try_from(rows[at].node.size).expect("small subtree")).expect("small tree");
            let mut moved: Vec<_> = rows.drain(at..end).collect();
            moved[0].parent = parent;
            sizes(rows);
            let to = insertion(rows, parent, before);
            drop(rows.splice(to..to, moved));
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
    sizes(rows);
}

fn equal(rows: &[Entry], tree: &Tree) {
    let model: Vec<_> = rows.iter().map(|row| row.node.clone()).collect();
    let wanted: Vec<_> = tree.nodes().iter().map(Made::of).collect();
    assert_eq!(model, wanted, "patches reproduce the platform-visible tree");
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
    /// Returns the platform projection for an external observer.
    #[must_use]
    pub fn nodes(&self) -> Vec<Made> {
        self.rows.iter().map(|row| row.node.clone()).collect()
    }
}
