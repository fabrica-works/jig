//! One node per call with bounded open parents (client/domain/view.md, section 4).
//! `open` enters a parent, `close` records its subtree size, and `finish` hands
//! over a complete tree. Attributes change the innermost open node. Capacity
//! errors assert before allocating bytes: the caller's counts must cover all
//! page nodes, fallback children and fragment nodes.

use crate::tree::length;
use crate::{Binding, Drawing, DrawingKind, Key, Limits, Node, Role, Run, State, Style, TextRun, Tree, Value};
use skein_lib::{Stack, bytes::copy_of};

/// A tree under construction and its bounded parent stack.
#[derive(Debug)]
pub struct Builder {
    pub(crate) tree: Tree,
    pub(crate) limits: Limits,
    open: Stack<u32>,
    text_bytes: u32,
    bindings: u32,
}

impl Builder {
    /// Reserve storage for a tree whose counts the caller has computed.
    #[must_use]
    pub fn new(limits: &Limits) -> Builder {
        Builder {
            tree: Tree::new(limits.nodes),
            limits: *limits,
            open: Stack::with_capacity(limits.depth),
            text_bytes: 0,
            bindings: 0,
        }
    }

    /// Empty a spare tree while retaining its node allocation.
    #[must_use]
    pub fn reuse(mut tree: Tree, limits: &Limits) -> Builder {
        assert!(tree.nodes.capacity() == limits.nodes, "spare tree has the configured capacity");
        tree.nodes.clear();
        Builder { tree, limits: *limits, open: Stack::with_capacity(limits.depth), text_bytes: 0, bindings: 0 }
    }

    /// Begin a node; calls between this and `close` build its children.
    pub fn open(&mut self, role: Role) {
        assert!(self.tree.nodes.room() > 0, "tree fits its node capacity");
        assert!(self.open.len() < self.limits.depth, "tree fits its depth capacity");
        let index = self.tree.len();
        let node = Node::new(role, &self.limits);
        self.tree.nodes.push(node).expect("tree fits its node capacity");
        self.open.push(index).expect("tree fits its depth capacity");
    }

    /// End the innermost node and record its full subtree's size.
    pub fn close(&mut self) {
        let index = self.open.pop().expect("a node is open");
        let size = self.tree.len().checked_sub(index).expect("node precedes its children");
        self.tree.nodes.get_mut(index).expect("open node exists").size = size;
    }

    fn index(&self) -> u32 {
        *self.open.top().expect("an attribute has an open node")
    }
    fn node(&self) -> &Node {
        self.tree.get(self.index()).expect("open node exists")
    }
    fn node_mut(&mut self) -> &mut Node {
        let index = self.index();
        self.tree.nodes.get_mut(index).expect("open node exists")
    }

    fn charge(&mut self, old: u32, new: u32) {
        let node_bytes = self
            .node()
            .text_bytes()
            .checked_sub(old)
            .expect("old bytes are held")
            .checked_add(new)
            .expect("node bytes fit u32");
        assert!(node_bytes <= self.limits.text_per_node, "node fits its text capacity");
        let total =
            self.text_bytes.checked_sub(old).expect("old bytes are held").checked_add(new).expect("tree bytes fit u32");
        assert!(total <= self.limits.text_bytes, "tree fits its text capacity");
        self.text_bytes = total;
    }

    /// Build a text leaf with one run.
    pub fn text(&mut self, run: Run, bytes: &[u8]) {
        self.open(Role::Text);
        self.run(run, bytes);
        self.close();
    }

    /// Append a nonempty run to an open text node; empty spans take no storage.
    pub fn run(&mut self, run: Run, bytes: &[u8]) {
        assert!(self.node().role == Role::Text, "runs belong to text nodes");
        if bytes.is_empty() {
            return;
        }
        self.charge(0, length(bytes));
        let text = copy_of(bytes);
        self.node_mut().text.push(TextRun { run, text }).expect("nonempty runs fit the text capacity");
    }

    /// Give the open node a name where its text does not name it.
    pub fn name(&mut self, bytes: &[u8]) {
        let old = match &self.node().name {
            Some(name) => length(name),
            None => 0,
        };
        self.charge(old, length(bytes));
        let name = copy_of(bytes);
        self.node_mut().name = Some(name);
    }

    /// Add a semantic state once.
    pub fn state(&mut self, state: State) {
        if !self.node().states.as_slice().contains(&state) {
            self.node_mut().states.push(state).expect("nine closed states fit");
        }
    }

    /// Add a style token once, within the configured count.
    pub fn style(&mut self, style: Style) {
        if !self.node().styles.as_slice().contains(&style) {
            self.node_mut().styles.push(style).expect("styles fit their capacity");
        }
    }

    /// Identify a list item by its object.
    pub fn key(&mut self, key: Key) {
        assert!(
            self.node().role == Role::Item || self.node().role == Role::TreeItem,
            "keys belong to list or tree items"
        );
        self.node_mut().key = Some(key);
    }

    /// Set a field's value together with the domain's own-write count.
    pub fn value(&mut self, bytes: &[u8], written: u64) {
        assert!(self.node().role == Role::TextField || self.node().role == Role::TextArea, "values belong to fields");
        let old = match &self.node().value {
            Some(value) => length(&value.text),
            None => 0,
        };
        self.charge(old, length(bytes));
        let text = copy_of(bytes);
        self.node_mut().value = Some(Value { text, written });
    }

    /// Name a link's address in its owner's vocabulary.
    pub fn link(&mut self, to: Binding) {
        assert!(self.node().role == Role::Link, "addresses belong to links");
        self.node_mut().link = Some(to);
    }

    /// Bind an actable node to its owner's token.
    pub fn bind(&mut self, binding: Binding) {
        assert!(self.node().role != Role::Drawing, "a drawing is shown and never bound");
        if self.node().binding.is_none() {
            assert!(self.bindings < self.limits.bindings, "bindings fit their capacity");
            self.bindings = self.bindings.checked_add(1).expect("binding count fits u32");
        }
        self.node_mut().binding = Some(binding);
    }

    /// Open a drawing; its following child nodes are the fallback in words.
    /// Both the owner's declared byte bound and the tree's bound are checked.
    pub fn drawing(&mut self, kind: DrawingKind, name: &[u8], data: &[u8], owner_bound: u32) {
        let bytes = length(data);
        assert!(bytes <= owner_bound && bytes <= self.limits.drawing_bytes, "drawing fits its owner's byte bound");
        self.open(Role::Drawing);
        self.name(name);
        let data = copy_of(data);
        self.node_mut().drawing = Some(Drawing { kind, data });
    }

    /// Hand over a tree only after every open node has been closed.
    #[must_use]
    pub fn finish(self) -> Tree {
        assert!(self.open.is_empty(), "all tree nodes are closed");
        self.tree
    }
}
